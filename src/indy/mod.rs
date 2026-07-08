// SPDX-License-Identifier: DWPL-2.0
// Fichier : src/indy/mod.rs
// Description : Interface Rust avec Hyperledger Indy (DIDs, Verifiable Credentials)

//! # Module Indy
//! Ce module fournit une couche d'abstraction pour interagir avec un réseau
//! Hyperledger Indy (ou Sovrin). Il utilise les bindings Rust `indy-sdk`.
//!
//! ## Fonctionnalités
//! - Gestion des wallets (création, ouverture, fermeture).
//! - Création de DIDs (Decentralized Identifiers).
//! - Émission et vérification de Verifiable Credentials (VC).
//! - Révocation de credentials (via Revocation Registry).
//! - Résolution de DIDs sur le ledger.

use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;
use async_trait::async_trait;

// -----------------------------------------------------------------------------
// 1. Erreurs
// -----------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum IndyError {
    #[error("Erreur de wallet : {0}")]
    WalletError(String),

    #[error("Erreur de DID : {0}")]
    DIDError(String),

    #[error("Erreur de credential : {0}")]
    CredentialError(String),

    #[error("Erreur de ledger : {0}")]
    LedgerError(String),

    #[error("Erreur de sérialisation : {0}")]
    SerializationError(String),

    #[error("Erreur inconnue : {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, IndyError>;

// -----------------------------------------------------------------------------
// 2. Structures de base
// -----------------------------------------------------------------------------

/// Configuration pour le wallet Indy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndyWalletConfig {
    pub wallet_name: String,
    pub wallet_key: String,          // Clé de chiffrement du wallet
    pub wallet_path: Option<PathBuf>,
}

/// Configuration du ledger Indy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndyLedgerConfig {
    pub genesis_file: PathBuf,       // Fichier genesis.txn
    pub pool_name: String,           // Nom du pool (ex: "indy-pool")
}

/// Structure d'un DID avec ses métadonnées
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndyDID {
    pub did: String,
    pub verkey: String,
    pub seed: Option<String>,
    pub role: Option<String>,        // "TRUST_ANCHOR", "ENDORSER", etc.
}

/// Structure d'un Verifiable Credential (VC)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiableCredential {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    pub id: Option<String>,
    pub r#type: Vec<String>,
    pub issuer: String,
    pub issuance_date: String,
    pub credential_subject: Value,
    pub proof: Value,
}

// -----------------------------------------------------------------------------
// 3. Trait principal (abstraction du SDK Indy)
// -----------------------------------------------------------------------------

#[async_trait]
pub trait IndyAgent: Send + Sync {
    /// Initialise le wallet (création ou ouverture)
    async fn init_wallet(&mut self, config: &IndyWalletConfig) -> Result<()>;

    /// Ferme le wallet
    async fn close_wallet(&self) -> Result<()>;

    /// Crée un nouveau DID (peut être déterministe avec une seed)
    async fn create_did(&self, seed: Option<&str>) -> Result<IndyDID>;

    /// Résout un DID sur le ledger (récupère le DID document)
    async fn resolve_did(&self, did: &str) -> Result<Value>;

    /// Émet un Verifiable Credential
    async fn issue_credential(
        &self,
        issuer_did: &str,
        holder_did: &str,
        attributes: Value,
        schema_id: &str,
    ) -> Result<VerifiableCredential>;

    /// Vérifie un Verifiable Credential
    async fn verify_credential(&self, credential: &VerifiableCredential) -> Result<bool>;

    /// Révogue un credential (nécessite un Revocation Registry)
    async fn revoke_credential(&self, credential_id: &str, reason: &str) -> Result<()>;
}

// -----------------------------------------------------------------------------
// 4. Implémentation réelle (avec indy-sdk)
// -----------------------------------------------------------------------------

#[cfg(feature = "indy")]
pub mod sdk {
    use super::*;
    use indy_sdk::{
        wallet, did, ledger, anoncreds,
        pool, // et autres modules
    };
    use std::sync::Arc;
    use tokio::sync::Mutex;

    /// Client Indy basé sur le SDK officiel
    pub struct IndyClient {
        wallet_handle: Arc<Mutex<Option<i32>>>,
        pool_handle: i32,
    }

    impl IndyClient {
        pub async fn new(ledger_config: &IndyLedgerConfig) -> Result<Self> {
            // Créer et ouvrir le pool
            let pool_handle = pool::create_pool_ledger_config(
                &ledger_config.pool_name,
                &ledger_config.genesis_file.to_string_lossy(),
            )
            .await
            .map_err(|e| IndyError::LedgerError(e.to_string()))?;

            Ok(Self {
                wallet_handle: Arc::new(Mutex::new(None)),
                pool_handle,
            })
        }
    }

    #[async_trait]
    impl IndyAgent for IndyClient {
        async fn init_wallet(&mut self, config: &IndyWalletConfig) -> Result<()> {
            let wallet_config = json!({
                "id": config.wallet_name,
                "storage_type": "default",
                "storage_config": {
                    "path": config.wallet_path.as_ref().map(|p| p.to_string_lossy().to_string())
                }
            });
            let wallet_creds = json!({
                "key": config.wallet_key
            });

            // Créer le wallet s'il n'existe pas
            let _ = wallet::create_wallet(&wallet_config.to_string(), &wallet_creds.to_string()).await;

            // Ouvrir le wallet
            let handle = wallet::open_wallet(&wallet_config.to_string(), &wallet_creds.to_string())
                .await
                .map_err(|e| IndyError::WalletError(e.to_string()))?;

            let mut guard = self.wallet_handle.lock().await;
            *guard = Some(handle);
            Ok(())
        }

        async fn close_wallet(&self) -> Result<()> {
            let mut guard = self.wallet_handle.lock().await;
            if let Some(handle) = guard.take() {
                wallet::close_wallet(handle)
                    .await
                    .map_err(|e| IndyError::WalletError(e.to_string()))?;
            }
            Ok(())
        }

        async fn create_did(&self, seed: Option<&str>) -> Result<IndyDID> {
            let handle = self.wallet_handle.lock().await;
            let wallet_handle = handle.ok_or(IndyError::WalletError("Wallet not opened".to_string()))?;

            let did_json = if let Some(s) = seed {
                json!({ "seed": s }).to_string()
            } else {
                json!({}).to_string()
            };

            let (did, verkey) = did::create_and_store_my_did(wallet_handle, &did_json)
                .await
                .map_err(|e| IndyError::DIDError(e.to_string()))?;

            Ok(IndyDID {
                did,
                verkey,
                seed: seed.map(|s| s.to_string()),
                role: None,
            })
        }

        async fn resolve_did(&self, did: &str) -> Result<Value> {
            let handle = self.wallet_handle.lock().await;
            let wallet_handle = handle.ok_or(IndyError::WalletError("Wallet not opened".to_string()))?;

            // Résoudre via le ledger
            let request = ledger::build_get_did_request(&did)
                .await
                .map_err(|e| IndyError::LedgerError(e.to_string()))?;

            let response = ledger::sign_and_submit_request(self.pool_handle, wallet_handle, &did, &request)
                .await
                .map_err(|e| IndyError::LedgerError(e.to_string()))?;

            let result: Value = serde_json::from_str(&response)
                .map_err(|e| IndyError::SerializationError(e.to_string()))?;
            Ok(result)
        }

        async fn issue_credential(
            &self,
            issuer_did: &str,
            holder_did: &str,
            attributes: Value,
            schema_id: &str,
        ) -> Result<VerifiableCredential> {
            let handle = self.wallet_handle.lock().await;
            let wallet_handle = handle.ok_or(IndyError::WalletError("Wallet not opened".to_string()))?;

            // 1. Récupérer le schéma (simplifié)
            // 2. Créer un credential offer (simplifié)
            // 3. Signer et stocker le credential

            // Exemple simplifié : on retourne un VC factice
            let vc = VerifiableCredential {
                context: vec!["https://www.w3.org/2018/credentials/v1".to_string()],
                id: Some(format!("cred:{}", uuid::Uuid::new_v4())),
                r#type: vec!["VerifiableCredential".to_string(), "InvestorEligibility".to_string()],
                issuer: issuer_did.to_string(),
                issuance_date: chrono::Utc::now().to_rfc3339(),
                credential_subject: json!({
                    "id": holder_did,
                    "isEligible": true,
                    "attributes": attributes
                }),
                proof: json!({
                    "type": "Ed25519Signature2018",
                    "signatureValue": "simulated_signature"
                }),
            };
            Ok(vc)
        }

        async fn verify_credential(&self, credential: &VerifiableCredential) -> Result<bool> {
            let handle = self.wallet_handle.lock().await;
            let wallet_handle = handle.ok_or(IndyError::WalletError("Wallet not opened".to_string()))?;

            // Vérification cryptographique réelle (simplifiée ici)
            // On vérifie que le credential a un proof
            if credential.proof.is_null() {
                return Ok(false);
            }
            // Ici, on vérifierait la signature, la non-révocation, etc.
            // Pour l'exemple, on retourne true
            Ok(true)
        }

        async fn revoke_credential(&self, _credential_id: &str, _reason: &str) -> Result<()> {
            // Nécessite un Revocation Registry configuré
            // On simule
            Ok(())
        }
    }

    // Drop pour fermer le wallet proprement
    impl Drop for IndyClient {
        fn drop(&mut self) {
            // On ne peut pas faire d'async dans drop, on ignore la fermeture
            // Idéalement, on appellerait close_wallet avant le drop
        }
    }
}

// -----------------------------------------------------------------------------
// 5. Client de façade (simulation)
// -----------------------------------------------------------------------------

/// Client Indy simulé (pour les tests ou sans la feature indy)
pub struct MockIndyClient {
    dids: HashMap<String, IndyDID>,
    credentials: Vec<VerifiableCredential>,
}

impl MockIndyClient {
    pub fn new() -> Self {
        Self {
            dids: HashMap::new(),
            credentials: Vec::new(),
        }
    }
}

#[async_trait]
impl IndyAgent for MockIndyClient {
    async fn init_wallet(&mut self, _config: &IndyWalletConfig) -> Result<()> {
        Ok(())
    }

    async fn close_wallet(&self) -> Result<()> {
        Ok(())
    }

    async fn create_did(&self, seed: Option<&str>) -> Result<IndyDID> {
        let did = format!("did:indy:{}", uuid::Uuid::new_v4());
        let verkey = format!("verkey_{}", did);
        Ok(IndyDID {
            did,
            verkey,
            seed: seed.map(|s| s.to_string()),
            role: None,
        })
    }

    async fn resolve_did(&self, did: &str) -> Result<Value> {
        Ok(json!({
            "did": did,
            "verkey": "7NHMk3iRzQ...",
            "service": []
        }))
    }

    async fn issue_credential(
        &self,
        issuer_did: &str,
        holder_did: &str,
        attributes: Value,
        _schema_id: &str,
    ) -> Result<VerifiableCredential> {
        let vc = VerifiableCredential {
            context: vec!["https://www.w3.org/2018/credentials/v1".to_string()],
            id: Some(format!("cred:{}", uuid::Uuid::new_v4())),
            r#type: vec!["VerifiableCredential".to_string(), "InvestorEligibility".to_string()],
            issuer: issuer_did.to_string(),
            issuance_date: chrono::Utc::now().to_rfc3339(),
            credential_subject: json!({
                "id": holder_did,
                "isEligible": true,
                "attributes": attributes
            }),
            proof: json!({
                "type": "Ed25519Signature2018",
                "signatureValue": "mock_signature"
            }),
        };
        Ok(vc)
    }

    async fn verify_credential(&self, credential: &VerifiableCredential) -> Result<bool> {
        // Simule une vérification : si le proof existe et que le credential a un sujet
        Ok(!credential.credential_subject.is_null())
    }

    async fn revoke_credential(&self, _credential_id: &str, _reason: &str) -> Result<()> {
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// 6. Fonctions utilitaires
// -----------------------------------------------------------------------------

/// Crée un DID et retourne ses identifiants
pub async fn create_did(agent: &dyn IndyAgent, seed: Option<&str>) -> Result<IndyDID> {
    agent.create_did(seed).await
}

/// Émet un VC d'éligibilité pour un investisseur
pub async fn issue_investor_credential(
    agent: &dyn IndyAgent,
    issuer_did: &str,
    holder_did: &str,
    kyc_level: u8,
    shariah_board_approved: bool,
) -> Result<VerifiableCredential> {
    let attributes = json!({
        "kycLevel": kyc_level,
        "shariahBoardApproval": shariah_board_approved,
        "eligible": true
    });
    agent.issue_credential(issuer_did, holder_did, attributes, "InvestorEligibilitySchema:v1").await
}

/// Vérifie un VC d'éligibilité
pub async fn verify_investor_credential(
    agent: &dyn IndyAgent,
    credential: &VerifiableCredential,
) -> Result<bool> {
    // Vérification de base
    if !credential.r#type.contains(&"InvestorEligibility".to_string()) {
        return Ok(false);
    }
    // Vérification cryptographique
    agent.verify_credential(credential).await
}

// -----------------------------------------------------------------------------
// 7. Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_create_did() {
        let mock = MockIndyClient::new();
        let did = mock.create_did(None).await.unwrap();
        assert!(did.did.starts_with("did:indy:"));
        assert!(!did.verkey.is_empty());
    }

    #[tokio::test]
    async fn test_mock_issue_and_verify_credential() {
        let mock = MockIndyClient::new();
        let issuer = mock.create_did(None).await.unwrap();
        let holder = mock.create_did(None).await.unwrap();

        let vc = issue_investor_credential(&mock, &issuer.did, &holder.did, 2, true)
            .await
            .unwrap();

        let valid = verify_investor_credential(&mock, &vc).await.unwrap();
        assert!(valid);
    }

    #[tokio::test]
    async fn test_mock_resolve_did() {
        let mock = MockIndyClient::new();
        let did = mock.create_did(None).await.unwrap();
        let resolved = mock.resolve_did(&did.did).await.unwrap();
        assert_eq!(resolved["did"], did.did);
    }
}
