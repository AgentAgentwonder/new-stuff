#[cfg(test)]
mod multisig_tests {
    use eclipse_market_pro::wallet::multisig::{
        CreateMultisigRequest, CreateProposalRequest, MultisigDatabase, ProposalStatus,
        SignProposalRequest,
    };
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_create_multisig_wallet() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_path_buf();
        let db = MultisigDatabase::new(db_path).await.unwrap();

        let request = CreateMultisigRequest {
            name: "Test Wallet".to_string(),
            members: vec![
                "11111111111111111111111111111111".to_string(),
                "22222222222222222222222222222222".to_string(),
                "33333333333333333333333333333333".to_string(),
            ],
            threshold: 2,
        };

        let wallet = db.create_wallet(request).await.unwrap();

        assert_eq!(wallet.name, "Test Wallet");
        assert_eq!(wallet.threshold, 2);
        assert_eq!(wallet.members.len(), 3);
        assert_eq!(wallet.balance, 0.0);
    }

    #[tokio::test]
    async fn test_list_wallets() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_path_buf();
        let db = MultisigDatabase::new(db_path).await.unwrap();

        // Create multiple wallets
        for i in 1..=3 {
            let request = CreateMultisigRequest {
                name: format!("Wallet {}", i),
                members: vec![
                    format!("{}1111111111111111111111111111111", i),
                    format!("{}2222222222222222222222222222222", i),
                ],
                threshold: 1,
            };
            db.create_wallet(request).await.unwrap();
        }

        let wallets = db.list_wallets().await.unwrap();
        assert_eq!(wallets.len(), 3);
    }

    #[tokio::test]
    async fn test_create_proposal() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_path_buf();
        let db = MultisigDatabase::new(db_path).await.unwrap();

        let wallet_request = CreateMultisigRequest {
            name: "Test Wallet".to_string(),
            members: vec![
                "11111111111111111111111111111111".to_string(),
                "22222222222222222222222222222222".to_string(),
            ],
            threshold: 2,
        };
        let wallet = db.create_wallet(wallet_request).await.unwrap();

        let proposal_request = CreateProposalRequest {
            wallet_id: wallet.id.clone(),
            transaction_data: "base64_encoded_transaction".to_string(),
            description: Some("Test proposal".to_string()),
            created_by: "11111111111111111111111111111111".to_string(),
        };

        let proposal = db.create_proposal(proposal_request).await.unwrap();

        assert_eq!(proposal.wallet_id, wallet.id);
        assert_eq!(proposal.status, ProposalStatus::Pending);
        assert_eq!(proposal.signatures.len(), 0);
        assert_eq!(proposal.description, Some("Test proposal".to_string()));
    }

    #[tokio::test]
    async fn test_sign_proposal() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_path_buf();
        let db = MultisigDatabase::new(db_path).await.unwrap();

        let wallet_request = CreateMultisigRequest {
            name: "Test Wallet".to_string(),
            members: vec![
                "11111111111111111111111111111111".to_string(),
                "22222222222222222222222222222222".to_string(),
            ],
            threshold: 2,
        };
        let wallet = db.create_wallet(wallet_request).await.unwrap();

        let proposal_request = CreateProposalRequest {
            wallet_id: wallet.id.clone(),
            transaction_data: "base64_encoded_transaction".to_string(),
            description: Some("Test proposal".to_string()),
            created_by: "11111111111111111111111111111111".to_string(),
        };
        let proposal = db.create_proposal(proposal_request).await.unwrap();

        // Add first signature
        let sign_request = SignProposalRequest {
            proposal_id: proposal.id.clone(),
            signer: "11111111111111111111111111111111".to_string(),
            signature: "signature1".to_string(),
        };
        db.add_signature(sign_request).await.unwrap();

        let updated_proposal = db.get_proposal(&proposal.id).await.unwrap().unwrap();
        assert_eq!(updated_proposal.signatures.len(), 1);
        assert_eq!(updated_proposal.status, ProposalStatus::Pending);

        // Add second signature (should reach threshold)
        let sign_request2 = SignProposalRequest {
            proposal_id: proposal.id.clone(),
            signer: "22222222222222222222222222222222".to_string(),
            signature: "signature2".to_string(),
        };
        db.add_signature(sign_request2).await.unwrap();

        let final_proposal = db.get_proposal(&proposal.id).await.unwrap().unwrap();
        assert_eq!(final_proposal.signatures.len(), 2);
        assert_eq!(final_proposal.status, ProposalStatus::Approved);
    }

    #[tokio::test]
    async fn test_execute_proposal() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_path_buf();
        let db = MultisigDatabase::new(db_path).await.unwrap();

        let wallet_request = CreateMultisigRequest {
            name: "Test Wallet".to_string(),
            members: vec![
                "11111111111111111111111111111111".to_string(),
                "22222222222222222222222222222222".to_string(),
            ],
            threshold: 1,
        };
        let wallet = db.create_wallet(wallet_request).await.unwrap();

        let proposal_request = CreateProposalRequest {
            wallet_id: wallet.id.clone(),
            transaction_data: "base64_encoded_transaction".to_string(),
            description: None,
            created_by: "11111111111111111111111111111111".to_string(),
        };
        let proposal = db.create_proposal(proposal_request).await.unwrap();

        // Sign to reach threshold
        let sign_request = SignProposalRequest {
            proposal_id: proposal.id.clone(),
            signer: "11111111111111111111111111111111".to_string(),
            signature: "signature1".to_string(),
        };
        db.add_signature(sign_request).await.unwrap();

        // Execute
        db.execute_proposal(&proposal.id, "tx_signature_hash".to_string())
            .await
            .unwrap();

        let executed_proposal = db.get_proposal(&proposal.id).await.unwrap().unwrap();
        assert_eq!(executed_proposal.status, ProposalStatus::Executed);
        assert!(executed_proposal.executed_at.is_some());
        assert_eq!(
            executed_proposal.tx_signature,
            Some("tx_signature_hash".to_string())
        );
    }

    #[tokio::test]
    async fn test_cancel_proposal() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_path_buf();
        let db = MultisigDatabase::new(db_path).await.unwrap();

        let wallet_request = CreateMultisigRequest {
            name: "Test Wallet".to_string(),
            members: vec!["11111111111111111111111111111111".to_string()],
            threshold: 1,
        };
        let wallet = db.create_wallet(wallet_request).await.unwrap();

        let proposal_request = CreateProposalRequest {
            wallet_id: wallet.id.clone(),
            transaction_data: "base64_encoded_transaction".to_string(),
            description: None,
            created_by: "11111111111111111111111111111111".to_string(),
        };
        let proposal = db.create_proposal(proposal_request).await.unwrap();

        db.cancel_proposal(&proposal.id).await.unwrap();

        let cancelled_proposal = db.get_proposal(&proposal.id).await.unwrap().unwrap();
        assert_eq!(cancelled_proposal.status, ProposalStatus::Cancelled);
    }

    #[tokio::test]
    async fn test_list_proposals_with_filter() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_path_buf();
        let db = MultisigDatabase::new(db_path).await.unwrap();

        let wallet_request = CreateMultisigRequest {
            name: "Test Wallet".to_string(),
            members: vec!["11111111111111111111111111111111".to_string()],
            threshold: 1,
        };
        let wallet = db.create_wallet(wallet_request).await.unwrap();

        // Create multiple proposals with different statuses
        let proposal1 = db
            .create_proposal(CreateProposalRequest {
                wallet_id: wallet.id.clone(),
                transaction_data: "tx1".to_string(),
                description: None,
                created_by: "11111111111111111111111111111111".to_string(),
            })
            .await
            .unwrap();

        let proposal2 = db
            .create_proposal(CreateProposalRequest {
                wallet_id: wallet.id.clone(),
                transaction_data: "tx2".to_string(),
                description: None,
                created_by: "11111111111111111111111111111111".to_string(),
            })
            .await
            .unwrap();

        db.cancel_proposal(&proposal2.id).await.unwrap();

        // List all proposals
        let all_proposals = db.list_proposals(&wallet.id, None).await.unwrap();
        assert_eq!(all_proposals.len(), 2);

        // List only pending proposals
        let pending_proposals = db
            .list_proposals(&wallet.id, Some("pending".to_string()))
            .await
            .unwrap();
        assert_eq!(pending_proposals.len(), 1);

        // List only cancelled proposals
        let cancelled_proposals = db
            .list_proposals(&wallet.id, Some("cancelled".to_string()))
            .await
            .unwrap();
        assert_eq!(cancelled_proposals.len(), 1);
    }
}
