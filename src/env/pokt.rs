use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct PoktConfig {
    pub project_id: String,

    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl PoktConfig {
    pub fn new(project_id: String) -> Self {
        Self {
            project_id,
            supported_chains: default_supported_chains(),
        }
    }
}

impl ProviderConfig for PoktConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Pokt
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    HashMap::from([
        // Solana Mainnet
        (
            "solana:4sgjmw1sunhzsxgspuhpqldx6wiyjntz".into(),
            (
                "solana-mainnet".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Avax C-Chain
        (
            "eip155:43114".into(),
            (
                "avax-mainnet".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Gnosis
        (
            "eip155:100".into(),
            ("poa-xdai".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Binance Smart Chain
        (
            "eip155:56".into(),
            ("bsc-mainnet".into(), Weight::new(Priority::High).unwrap()),
        ),
        // Ethereum
        // TODO: Reenable once Pokt fixes
        (
            "eip155:1".into(),
            ("mainnet".into(), Weight::new(Priority::Disabled).unwrap()),
        ),
        (
            "eip155:5".into(),
            ("goerli".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Optimism
        (
            "eip155:10".into(),
            (
                "optimism-mainnet".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Arbitrum
        (
            "eip155:42161".into(),
            (
                "arbitrum-one".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Polygon
        (
            "eip155:137".into(),
            ("poly-mainnet".into(), Weight::new(Priority::High).unwrap()),
        ),
        (
            "eip155:80001".into(),
            (
                "polygon-mumbai".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        (
            "eip155:1101".into(),
            (
                "polygon-zkevm-mainnet".into(),
                Weight::new(Priority::High).unwrap(),
            ),
        ),
        // Celo
        (
            "eip155:42220".into(),
            (
                "celo-mainnet".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
    ])
}
