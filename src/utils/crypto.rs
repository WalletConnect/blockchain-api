use {ethers::types::H160, std::str::FromStr};

/// Veryfy message signature signed by the keccak256
#[tracing::instrument]
pub fn verify_message_signature(
    message: &str,
    signature: &str,
    owner: &H160,
) -> Result<bool, Box<dyn std::error::Error>> {
    let prefixed_message = format!("\x19Ethereum Signed Message:\n{}{}", message.len(), message);
    let message_hash = ethers::core::utils::keccak256(prefixed_message.clone());
    let message_hash = ethers::types::H256::from_slice(&message_hash);

    let sign = ethers::types::Signature::from_str(signature)?;
    match sign.verify(message_hash, *owner) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Convert EVM chain ID to coin type ENSIP-11
#[tracing::instrument]
pub fn convert_evm_chain_id_to_coin_type(chain_id: u32) -> u32 {
    0x80000000 | chain_id
}

/// Convert coin type ENSIP-11 to EVM chain ID
#[tracing::instrument]
pub fn convert_coin_type_to_evm_chain_id(coin_type: u32) -> u32 {
    0x7FFFFFFF & coin_type
}

#[cfg(test)]
mod tests {
    use {super::*, ethers::types::H160, std::str::FromStr};

    #[test]
    fn test_verify_message_signature_valid() {
        let message = "test message signature";
        let signature = "0x660739ee06920c5f55fbaf0da4f435faaa9c55e2c9da303c50c4b3865191d67e5002a0b10eb0f89bae66823f7f07415ea9d5bbb607ee61ac98b7f2a0a44fcb5c1b";
        let owner = H160::from_str("0xAff392551773CCb2574fAE23195CC3aFDBe98d18").unwrap();

        let result = verify_message_signature(message, signature, &owner);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_verify_message_signature_json() {
        let message = r#"{\"test\":\"some my text\"}"#;
        let signature = "0x2fe0b640b4036c9c97911e6f22c72a2c934f1d67db02948055c0e0c84dbf4f2b33c2f8c4b000642735dbf5d1c96ba48ccd2a998324c9e4cb7bb776f0c95ee2fc1b";
        let owner = H160::from_str("0xAff392551773CCb2574fAE23195CC3aFDBe98d18").unwrap();

        let result = verify_message_signature(message, signature, &owner);
        assert!(result.is_ok());
        println!("result: {:?}", result);
        assert!(result.unwrap());
    }

    #[test]
    fn test_verify_message_signature_invalid() {
        let message = "wrong message signature";
        let signature = "0x660739ee06920c5f55fbaf0da4f435faaa9c55e2c9da303c50c4b3865191d67e5002a0b10eb0f89bae66823f7f07415ea9d5bbb607ee61ac98b7f2a0a44fcb5c1b"; // The signature of the message
        let owner = H160::from_str("0xAff392551773CCb2574fAE23195CC3aFDBe98d18").unwrap(); // The Ethereum address of the signer

        let result = verify_message_signature(message, signature, &owner);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_convert_coin_type_to_evm_chain_id() {
        // Polygon
        let chain_id = 137;
        let coin_type = 2147483785;
        assert_eq!(convert_evm_chain_id_to_coin_type(chain_id), coin_type);
        assert_eq!(convert_coin_type_to_evm_chain_id(coin_type), chain_id);
    }
}
