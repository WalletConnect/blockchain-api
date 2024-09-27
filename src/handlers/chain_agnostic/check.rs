use {
    super::{super::HANDLER_TASK_METRICS, BRIDGING_AVAILABLE_ASSETS},
    crate::{
        analytics::MessageSource,
        error::RpcError,
        state::AppState,
        utils::crypto::{decode_erc20_call_function_data, get_erc20_balance, Erc20FunctionType},
    },
    axum::{
        extract::{Query, State},
        response::{IntoResponse, Response},
        Json,
    },
    ethers::types::{H160, U256},
    serde::{Deserialize, Serialize},
    std::{str::FromStr, sync::Arc},
    tracing::{debug, error},
    wc::future::FutureExt,
};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    pub project_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckTransactionRequest {
    transaction: Transaction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    from: String,
    to: String,
    value: String,
    gas: String,
    gas_price: String,
    data: String,
    nonce: String,
    max_fee_per_gas: String,
    max_priority_fee_per_gas: String,
    chain_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequiresMultiChainResponse {
    requires_multi_chain: bool,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    query_params: Query<QueryParams>,
    Json(request_payload): Json<CheckTransactionRequest>,
) -> Result<Response, RpcError> {
    handler_internal(state, query_params, request_payload)
        .with_metrics(HANDLER_TASK_METRICS.with_name("ca_check"))
        .await
}

#[tracing::instrument(skip(state), level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    Query(query_params): Query<QueryParams>,
    request_payload: CheckTransactionRequest,
) -> Result<Response, RpcError> {
    state
        .validate_project_access_and_quota(&query_params.project_id.clone())
        .await?;

    let from_address =
        H160::from_str(&request_payload.transaction.from).map_err(|_| RpcError::InvalidAddress)?;

    // Check the native token balance
    let native_token_balance = get_erc20_balance(
        &request_payload.transaction.chain_id,
        H160::repeat_byte(0xee),
        from_address,
        &query_params.project_id.clone(),
        MessageSource::ChainAgnosticCheck,
    )
    .await?;
    let transfer_value_stripped = request_payload.transaction.value.trim_start_matches("0x");
    let transfer_value = U256::from_dec_str(transfer_value_stripped)
        .map_err(|_| RpcError::InvalidValue(transfer_value_stripped.to_string()))?;

    // If the native token balance is greater than the transfer value, we don't need multi-chain bridging
    if native_token_balance > transfer_value {
        return Ok(Json(RequiresMultiChainResponse {
            requires_multi_chain: false,
        })
        .into_response());
    }

    // Check if the transaction data is the `transfer`` ERC20 function
    let transaction_data = hex::decode(request_payload.transaction.data.trim_start_matches("0x"))
        .map_err(|e| RpcError::WrongHexFormat(e.to_string()))?;
    if decode_erc20_call_function_data(&transaction_data)? != Erc20FunctionType::Transfer {
        error!("The transaction data is not a transfer function");
        return Ok(Json(RequiresMultiChainResponse {
            requires_multi_chain: false,
        })
        .into_response());
    }

    // Check the ERC20 tokens balance for each of supported assets
    for (asset, chains) in BRIDGING_AVAILABLE_ASSETS.entries() {
        for (chain_id, contract_address) in chains.entries() {
            let erc20_balance = get_erc20_balance(
                chain_id,
                H160::from_str(contract_address).map_err(|_| RpcError::InvalidAddress)?,
                from_address,
                &query_params.project_id.clone(),
                MessageSource::ChainAgnosticCheck,
            )
            .await?;
            if erc20_balance > transfer_value {
                debug!(
                    "The balance of the asset {} on the chain {} can be used for the bridging",
                    asset, chain_id
                );
                return Ok(Json(RequiresMultiChainResponse {
                    requires_multi_chain: true,
                })
                .into_response());
            }
        }
    }

    // No balance is sufficient for the transfer or bridging
    Ok(Json(RequiresMultiChainResponse {
        requires_multi_chain: false,
    })
    .into_response())
}
