use {
    axum::response::Response,
    axum_tungstenite::WebSocketUpgrade,
    rand::{distributions::WeightedIndex, prelude::Distribution, rngs::OsRng},
};

mod binance;
mod infura;
mod pokt;
mod zksync;

use {
    crate::{error::RpcResult, handlers::RpcQueryParams},
    async_trait::async_trait,
    std::{collections::HashMap, fmt::Display, sync::Arc},
};
pub use {
    binance::BinanceProvider,
    infura::{InfuraProvider, InfuraWsProvider},
    pokt::PoktProvider,
    zksync::ZKSyncProvider,
};

#[derive(Default, Clone)]
pub struct ProviderRepository {
    map: HashMap<String, Vec<(Arc<dyn RpcProvider>, u32)>>,
    ws_map: HashMap<String, Vec<(Arc<dyn RpcWsProvider>, u32)>>,
}

impl ProviderRepository {
    pub fn get_provider_for_chain_id(&self, chain_id: &str) -> Option<&Arc<dyn RpcProvider>> {
        let providers = self.map.get(chain_id)?;

        if providers.is_empty() {
            return None;
        }

        let weights: Vec<_> = providers.iter().map(|(_, weight)| *weight).collect();
        let dist = WeightedIndex::new(&weights).unwrap();
        let provider = &providers[dist.sample(&mut OsRng)].0;

        Some(provider)
    }

    pub fn get_ws_provider_for_chain_id(&self, chain_id: &str) -> Option<&Arc<dyn RpcWsProvider>> {
        let providers = self.ws_map.get(chain_id)?;

        if providers.is_empty() {
            return None;
        }

        let weights: Vec<_> = providers.iter().map(|(_, weight)| *weight).collect();
        let dist = WeightedIndex::new(&weights).unwrap();
        let provider = &providers[dist.sample(&mut OsRng)].0;

        Some(provider)
    }

    pub fn add_ws_provider(
        &mut self,
        _provider_name: String,
        provider: Arc<dyn RpcWsProvider>,
        weight: u32,
    ) {
        provider
            .supported_caip_chainids()
            .into_iter()
            .for_each(|chain| {
                self.ws_map
                    .entry(chain)
                    .or_insert_with(Vec::new)
                    .push((provider.clone(), weight));
            });
    }

    pub fn add_provider(&mut self, provider: Arc<dyn RpcProvider>, weight: u32) {
        provider
            .supported_caip_chainids()
            .into_iter()
            .for_each(|chain| {
                self.map
                    .entry(chain)
                    .or_insert_with(Vec::new)
                    .push((provider.clone(), weight));
            });
    }
}

pub enum ProviderKind {
    Infura,
    Pokt,
    Binance,
    ZKSync,
}

impl Display for ProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            ProviderKind::Infura => "Infura",
            ProviderKind::Pokt => "Pokt",
            ProviderKind::Binance => "Binance",
            ProviderKind::ZKSync => "zkSync",
        })
    }
}

#[async_trait]
pub trait RpcProvider: Send + Sync + Provider {
    async fn proxy(
        &self,
        method: hyper::http::Method,
        xpath: axum::extract::MatchedPath,
        query_params: RpcQueryParams,
        headers: hyper::http::HeaderMap,
        body: hyper::body::Bytes,
    ) -> RpcResult<Response>;
}

#[async_trait]
pub trait RpcWsProvider: Send + Sync + Provider {
    async fn proxy(
        &self,
        ws: WebSocketUpgrade,
        query_params: RpcQueryParams,
    ) -> RpcResult<Response>;
}

pub trait Provider {
    fn supports_caip_chainid(&self, chain_id: &str) -> bool;

    fn supported_caip_chainids(&self) -> Vec<String>;

    fn provider_kind(&self) -> ProviderKind;

    fn project_id(&self) -> &str;
}
