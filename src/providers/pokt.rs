use {
    super::{Provider, ProviderKind, RateLimited, RpcProvider, RpcProviderFactory},
    crate::{
        env::PoktConfig,
        error::{RpcError, RpcResult},
    },
    async_trait::async_trait,
    axum::response::{IntoResponse, Response},
    hyper::{self, client::HttpConnector, Client, Method},
    hyper_tls::HttpsConnector,
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct PoktProvider {
    pub client: Client<HttpsConnector<HttpConnector>>,
    pub project_id: String,
    pub supported_chains: HashMap<String, String>,
}

impl Provider for PoktProvider {
    fn supports_caip_chainid(&self, chain_id: &str) -> bool {
        self.supported_chains.contains_key(chain_id)
    }

    fn supported_caip_chains(&self) -> Vec<String> {
        self.supported_chains.keys().cloned().collect()
    }

    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Pokt
    }
}

#[async_trait]
impl RateLimited for PoktProvider {
    // async fn is_rate_limited(&self, response: &mut Response) -> bool
    // where
    //     Self: Sized,
    // {
    //     let Ok(bytes) = body::to_bytes(response.body_mut()).await else {return
    // false};     let Ok(jsonrpc_response) =
    // serde_json::from_slice::<jsonrpc::Response>(&bytes) else {return false};

    //     if let Some(err) = jsonrpc_response.error {
    //         // Code used by Pokt to indicate rate limited request
    //         // https://github.com/pokt-foundation/portal-api/blob/e06d1e50abfee8533c58768bb9b638c351b87a48/src/controllers/v1.controller.ts
    //         if err.code == -32068 {
    //             return true;
    //         }
    //     }

    //     let body: axum::body::Body =
    // axum::body::Body::wrap_stream(hyper::body::Body::from(bytes));
    //     let body: UnsyncBoxBody<bytes::Bytes, axum_core::Error> =
    // body.boxed_unsync();     let mut_body = response.body_mut();
    //     false
    // }

    // TODO: Implement rate limiting as this is mocked
    async fn is_rate_limited(&self, _response: &mut Response) -> bool {
        false
    }
}

#[async_trait]
impl RpcProvider for PoktProvider {
    async fn proxy(&self, chain_id: &str, body: hyper::body::Bytes) -> RpcResult<Response> {
        let chain = &self
            .supported_chains
            .get(chain_id)
            .ok_or(RpcError::ChainNotFound)?;

        let uri = format!(
            "https://{}.gateway.pokt.network/v1/lb/{}",
            chain, self.project_id
        );

        let hyper_request = hyper::http::Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("Content-Type", "application/json")
            .body(hyper::body::Body::from(body))?;

        let response = self.client.request(hyper_request).await?;

        Ok(response.into_response())
    }
}

impl RpcProviderFactory<PoktConfig> for PoktProvider {
    fn new(provider_config: &PoktConfig) -> Self {
        let forward_proxy_client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
        let supported_chains: HashMap<String, String> = provider_config
            .supported_chains
            .iter()
            .map(|(k, v)| (k.clone(), v.0.clone()))
            .collect();

        PoktProvider {
            client: forward_proxy_client,
            supported_chains,
            project_id: provider_config.project_id.clone(),
        }
    }
}
