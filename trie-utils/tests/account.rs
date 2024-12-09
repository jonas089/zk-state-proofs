#[cfg(test)]
mod test {
    use std::str::FromStr;

    use alloy::{
        consensus::Account,
        hex::{self, FromHex, ToHex},
        primitives::{Address, FixedBytes},
        providers::{Provider, ProviderBuilder},
    };
    use crypto_ops::{keccak::digest_keccak, types::MerkleProofInput, verify_merkle_proof};
    use trie_utils::{get_ethereum_account_proof_inputs, load_infura_key_from_env};
    use url::Url;

    /// This test could fail in case a new block is produced during execution.
    /// The risk of this happening is accepted and it's recommended to re-run the test
    /// in the very rare case where it fails for said reason.
    /// The same is true for the test in storage.rs
    #[tokio::test]
    async fn test_verify_account_and_storage_proof() {
        let key = load_infura_key_from_env();
        let rpc_url = "https://mainnet.infura.io/v3/".to_string() + &key;
        let provider = ProviderBuilder::new().on_http(Url::from_str(&rpc_url).unwrap());
        let block = provider
            .get_block(
                alloy::eips::BlockId::Number(provider.get_block_number().await.unwrap().into()),
                alloy::rpc::types::BlockTransactionsKind::Full,
            )
            .await
            .unwrap()
            .unwrap();
        let proof = provider
            .get_proof(
                Address::from_hex("0xdAC17F958D2ee523a2206206994597C13D831ec7").unwrap(),
                vec![FixedBytes::from_hex(
                    "0000000000000000000000000000000000000000000000000000000000000000",
                )
                .unwrap()],
            )
            .await
            .expect("Failed to get proof");

        let inputs: MerkleProofInput = get_ethereum_account_proof_inputs(
            Address::from_hex("0xdAC17F958D2ee523a2206206994597C13D831ec7").unwrap(),
        )
        .await;
        println!("Proof: {:?}", &proof);
        let account_proof: Vec<u8> = verify_merkle_proof(
            block.header.state_root,
            inputs.proof,
            &digest_keccak(&hex::decode("0xdAC17F958D2ee523a2206206994597C13D831ec7").unwrap()),
        );
        let decoded_account: Account = alloy_rlp::decode_exact(&account_proof).unwrap();
        assert_eq!(
            decoded_account.storage_root.encode_hex::<String>(),
            hex::encode(&proof.storage_hash)
        );
        let _ = verify_merkle_proof(
            proof.storage_hash,
            proof
                .storage_proof
                .first()
                .unwrap()
                .proof
                .clone()
                .into_iter()
                .map(|b| b.to_vec())
                .collect(),
            &digest_keccak(
                &hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
                    .unwrap(),
            ),
        );
        println!("[Success] Verified Account Proof against Block Root")
    }
}
