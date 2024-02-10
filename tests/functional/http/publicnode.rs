use {
    super::check_if_rpc_is_responding_correctly_for_supported_chain,
    crate::context::ServerContext,
    rpc_proxy::providers::ProviderKind,
    test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn publicnode_provider(ctx: &mut ServerContext) {
    // Ethereum mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Publicnode,
        "eip155:1",
        "0x1",
    )
    .await;

    // Ethereum goerli
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Publicnode,
        "eip155:5",
        "0x5",
    )
    .await;

    // Base mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Publicnode,
        "eip155:8453",
        "0x2105",
    )
    .await;

    // BSC mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Publicnode,
        "eip155:56",
        "0x38",
    )
    .await;

    // BSC testnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Publicnode,
        "eip155:97",
        "0x61",
    )
    .await;

    // Avalanche c chain
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Publicnode,
        "eip155:43114",
        "0xa86a",
    )
    .await;

    // Avalanche fuji testnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Publicnode,
        "eip155:43113",
        "0xa869",
    )
    .await;

    // Polygon mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Publicnode,
        "eip155:137",
        "0x89",
    )
    .await;

    // Polygon mumbai
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Publicnode,
        "eip155:80001",
        "0x13881",
    )
    .await;
}
