use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct OmniatechConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl Default for OmniatechConfig {
    fn default() -> Self {
        Self {
            supported_chains: supported_chains(),
        }
    }
}

impl ProviderConfig for OmniatechConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Omniatech
    }
}

pub fn supported_chains() -> HashMap<String, (String, Weight)> {
    HashMap::from([
        // Ethereum mainnet
        (
            "eip155:1".into(),
            ("eth".into(), Weight::new(Priority::Low).unwrap()),
        ),
        // Binance Smart Chain mainnet
        (
            "eip155:56".into(),
            ("bsc".into(), Weight::new(Priority::Low).unwrap()),
        ),
        // Polygon
        (
            "eip155:137".into(),
            ("matic".into(), Weight::new(Priority::Low).unwrap()),
        ),
        // Near
        (
            "near".into(),
            ("near".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Aurora
        (
            "eip155:1313161554".into(),
            ("aurora".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Optimism
        (
            "eip155:10".into(),
            ("op".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Solana
        (
            "solana-mainnet".into(),
            ("sol".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Avalanche C chain
        (
            "eip155:43114".into(),
            ("avax".into(), Weight::new(Priority::Normal).unwrap()),
        ),
    ])
}
