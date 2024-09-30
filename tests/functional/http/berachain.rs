use {
    super::check_if_rpc_is_responding_correctly_for_supported_chain, crate::context::ServerContext,
    rpc_proxy::providers::ProviderKind, test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn berachain_provider_eip155_80084(ctx: &mut ServerContext) {
    // Berachain bArtio
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Berachain,
        "eip155:80084",
        "0x138d4",
    )
    .await;
}
