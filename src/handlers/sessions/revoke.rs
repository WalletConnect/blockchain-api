use {
    super::{super::HANDLER_TASK_METRICS, PermissionRevokeRequest, QueryParams},
    crate::{
        error::RpcError, state::AppState, storage::irn::OperationType,
        utils::crypto::disassemble_caip10,
    },
    axum::{
        extract::{Path, Query, State},
        response::Response,
        Json,
    },
    std::{sync::Arc, time::SystemTime},
    wc::future::FutureExt,
};

pub async fn handler(
    state: State<Arc<AppState>>,
    address: Path<String>,
    query_params: Query<QueryParams>,
    Json(request_payload): Json<PermissionRevokeRequest>,
) -> Result<Response, RpcError> {
    handler_internal(state, address, query_params, request_payload)
        .with_metrics(HANDLER_TASK_METRICS.with_name("sessions_revoke"))
        .await
}

#[tracing::instrument(skip(state), level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    Path(address): Path<String>,
    query_params: Query<QueryParams>,
    request_payload: PermissionRevokeRequest,
) -> Result<Response, RpcError> {
    let project_id = query_params.project_id.clone();
    state.validate_project_access_and_quota(&project_id).await?;

    let irn_client = state.irn.as_ref().ok_or(RpcError::IrnNotConfigured)?;

    // Checking the CAIP-10 address format
    disassemble_caip10(&address)?;

    // Remove the session/permission item from the IRN
    let irn_call_start = SystemTime::now();
    irn_client.hdel(address, request_payload.pci).await?;
    state
        .metrics
        .add_irn_latency(irn_call_start, OperationType::Hdel);

    Ok(Response::default())
}
