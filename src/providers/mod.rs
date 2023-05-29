use {
    crate::env::ProviderConfig,
    axum::response::Response,
    axum_tungstenite::WebSocketUpgrade,
    rand::{distributions::WeightedIndex, prelude::Distribution, rngs::OsRng},
    std::{fmt::Debug, hash::Hash, sync::Arc},
    tracing::info,
};

mod binance;
mod infura;
mod omnia;
mod pokt;
mod publicnode;
mod weights;
mod zksync;

use {
    crate::{error::RpcResult, handlers::RpcQueryParams},
    async_trait::async_trait,
    std::{collections::HashMap, fmt::Display},
};
pub use {
    binance::BinanceProvider,
    infura::{InfuraProvider, InfuraWsProvider},
    omnia::OmniatechProvider,
    pokt::PoktProvider,
    publicnode::PublicnodeProvider,
    zksync::ZKSyncProvider,
};

#[derive(Default)]
pub struct ProviderRepository {
    providers: HashMap<ProviderKind, Arc<dyn RpcProvider>>,
    ws_providers: HashMap<ProviderKind, Arc<dyn RpcWsProvider>>,
    // TODO: create newtype for ChainId
    weight_resolver: HashMap<String, Vec<(ProviderKind, Weight)>>,
    ws_weight_resolver: HashMap<String, Vec<(ProviderKind, Weight)>>,

    prometheus_client: prometheus_http_query::Client,
}

impl ProviderRepository {
    pub fn get_provider_for_chain_id(&self, chain_id: &str) -> Option<Arc<dyn RpcProvider>> {
        let Some(providers) = self.weight_resolver.get(chain_id) else {return None};

        if providers.is_empty() {
            return None;
        }

        let weights: Vec<_> = providers.iter().map(|(_, weight)| weight.value()).collect();
        let dist = WeightedIndex::new(weights).unwrap();
        let provider = &providers[dist.sample(&mut OsRng)].0;

        self.providers.get(provider).cloned()
    }

    pub fn get_ws_provider_for_chain_id(&self, chain_id: &str) -> Option<Arc<dyn RpcWsProvider>> {
        let Some(providers) = self.ws_weight_resolver.get(chain_id) else {return None};

        if providers.is_empty() {
            return None;
        }

        let weights: Vec<_> = providers.iter().map(|(_, weight)| weight.value()).collect();
        let dist = WeightedIndex::new(weights).unwrap();
        let provider = &providers[dist.sample(&mut OsRng)].0;

        self.ws_providers.get(provider).cloned()
    }

    pub fn add_ws_provider<
        T: RpcProviderFactory<C> + RpcWsProvider + 'static,
        C: ProviderConfig,
    >(
        &mut self,
        provider_config: C,
    ) {
        let ws_provider = T::new(&provider_config);
        let arc_ws_provider = Arc::new(ws_provider);

        self.ws_providers
            .insert(provider_config.provider_kind(), arc_ws_provider);

        let provider_kind = provider_config.provider_kind();
        let supported_ws_chains = provider_config.supported_chains();

        supported_ws_chains
            .into_iter()
            .for_each(|(chain_id, (_, weight))| {
                self.ws_weight_resolver
                    .entry(chain_id.clone())
                    .or_insert_with(Vec::new)
                    .push((provider_kind, weight));
            });
    }

    pub fn add_provider<T: RpcProviderFactory<C> + RpcProvider + 'static, C: ProviderConfig>(
        &mut self,
        provider_config: C,
    ) {
        let provider = T::new(&provider_config);
        let arc_provider = Arc::new(provider);

        self.providers
            .insert(provider_config.provider_kind(), arc_provider);

        let provider_kind = provider_config.provider_kind();
        let supported_chains = provider_config.supported_chains();

        supported_chains
            .into_iter()
            .for_each(|(chain_id, (_, weight))| {
                self.weight_resolver
                    .entry(chain_id.clone())
                    .or_insert_with(Vec::new)
                    .push((provider_kind, weight));
            });
    }

    pub async fn update_weights(&self) {
        info!("Updating weights");
        self.weight_resolver.iter().for_each(
            (|(_, providers)| {
                providers.iter().for_each(|(_, weight)| {
                    weight.0.store(
                        rand::random::<u32>() % 25,
                        std::sync::atomic::Ordering::SeqCst,
                    );
                });
            }),
        );
        let data = self
            .prometheus_client
            .query("round(increase(provider_status_code_counter[1m]))")
            .get()
            .await
            .unwrap();
        // self.map.iter().for_each(|(_, providers)| {
        //     providers.iter().for_each(|(_, weight)| {
        //         weight.0.store(3, std::sync::atomic::Ordering::SeqCst);
        //     });
        // });
        // self.weight_resolver.
    }
}

// TODO: Find better name
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderKind {
    Infura,
    Pokt,
    Binance,
    ZKSync,
    Publicnode,
    Omniatech,
}

impl Display for ProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            ProviderKind::Infura => "Infura",
            ProviderKind::Pokt => "Pokt",
            ProviderKind::Binance => "Binance",
            ProviderKind::ZKSync => "zkSync",
            ProviderKind::Publicnode => "Publicnode",
            ProviderKind::Omniatech => "Omniatech",
        })
    }
}

#[async_trait]
pub trait RpcProvider: Provider {
    async fn proxy(
        &self,
        method: hyper::http::Method,
        xpath: axum::extract::MatchedPath,
        query_params: RpcQueryParams,
        headers: hyper::http::HeaderMap,
        body: hyper::body::Bytes,
    ) -> RpcResult<Response>;
}

pub trait RpcProviderFactory<T: ProviderConfig>: Provider {
    fn new(provider_config: &T) -> Self;
}

#[async_trait]
pub trait RpcWsProvider: Provider {
    async fn proxy(
        &self,
        ws: WebSocketUpgrade,
        query_params: RpcQueryParams,
    ) -> RpcResult<Response>;
}

#[derive(Debug)]
pub struct Weight(pub std::sync::atomic::AtomicU32);

impl Weight {
    pub fn value(&self) -> u32 {
        self.0.load(std::sync::atomic::Ordering::SeqCst)
    }
}

// TODO: This is should not be Clone ever.
// Cloning it makes it possible that updates to the weight are not reflected in
// the map
impl Clone for Weight {
    fn clone(&self) -> Self {
        let atomic =
            std::sync::atomic::AtomicU32::new(self.0.load(std::sync::atomic::Ordering::SeqCst));
        Self(atomic)
    }
}

#[derive(Debug)]
pub struct SupportedChain {
    pub chain_id: String,
    pub weight: Weight,
}

pub trait Provider: Send + Sync + Debug {
    fn supports_caip_chainid(&self, chain_id: &str) -> bool;

    fn supported_caip_chains(&self) -> Vec<String>;

    fn provider_kind(&self) -> ProviderKind;
}
