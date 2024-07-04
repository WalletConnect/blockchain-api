use serde::{Deserialize, Serialize};

pub mod context;
pub mod create;
pub mod get;
pub mod list;

/// Payload to create a new permission
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewPermissionPayload {
    pub permission: PermissionItem,
}

// Payload to get permission by PCI
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetPermissionsRequest {
    address: String,
    pci: String,
}

/// Permission item schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionItem {
    permission_type: String,
    data: String,
    required: bool,
    on_chain_validated: bool,
}

/// Permissions Context item schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionContextItem {
    pci: String,
    signature: String,
    context: PermissionSubContext,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionSubContext {
    signer: PermissionContextSigner,
    expiry: usize,
    signer_data: PermissionContextSignerData,
    factory: String,
    factory_data: String,
    permissions_context: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionContextSigner {
    permission_type: String,
    ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionContextSignerData {
    user_op_builder: String,
}

/// Serialized permission item schema to store it in the IRN database
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoragePermissionsItem {
    permissions: PermissionItem,
    context: Option<PermissionContextItem>,
    verification_key: String,
}
