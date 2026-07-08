//! # Islamic Shariah Validator (ISV)
//!
//! Bibliothèque modulaire pour la validation de contrats financiers selon la Charia islamique.
//!
//! ## Principes Shariah implémentés
//!
//! - **Riba** : Interdiction des taux d'intérêt fixes et garantis.
//! - **Gharar** : Interdiction de l'incertitude excessive dans les contrats.
//! - **Maysir** : Interdiction des transactions spéculatives ou aléatoires.
//! - **Asset-Backing** : Exigence d'adossement à des actifs tangibles.
//! - **Profit Sharing** : Validation des ratios de partage des profits (Mudarabah/Musharakah).
//! - **Halal Industry** : Vérification de la conformité du secteur d'activité.
//!
//! ## Features
//!
//! | Feature | Description | Dépendances |
//! |---------|-------------|-------------|
//! | `fabric` | Intégration avec Hyperledger Fabric | `fabric-sdk`, `protobuf` |
//! | `nym`    | Intégration avec le mixnet Nym | `nym-sdk` |
//! | `wasm`   | Bindings WebAssembly | `wasm-bindgen`, `getrandom`, `serde-wasm-bindgen` |
//! | `full`   | Active toutes les features (fabric, wasm, nym) | - |
//!
//! ## Exemple d'utilisation (core only)
//!
//! ```rust
//! use islamic_shariah_validator::{FinancialContract, ShariahValidator};
//!
//! let contract = FinancialContract::default();
//! let validator = ShariahValidator::new();
//! let report = validator.validate(&contract);
//! assert!(report.is_valid);
//! ```
//!
//! ## Exemple avec Fabric
//!
//! ```rust,no_run
//! #[cfg(feature = "fabric")]
//! use islamic_shariah_validator::{validate_with_fabric, FabricClientWrapper, FinancialContract};
//!
//! #[cfg(feature = "fabric")]
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = FabricClientWrapper::new();
//!     let contract = FinancialContract::default();
//!     let report = validate_with_fabric(&contract, &client).await?;
//!     println!("Rapport : {:?}", report);
//!     Ok(())
//! }
//! ```
//!
//! ## Exemple avec WASM
//!
//! ```javascript
//! import init, { validate_wasm } from "./islamic_shariah_validator.js";
//!
//! await init();
//! const contract = { contract_type: "Mudarabah", industry: "Halal", ... };
//! const report = JSON.parse(validate_wasm(JSON.stringify(contract)));
//! console.log(report.is_valid ? "✅ Conforme" : "❌ Non conforme");
//! ```

#![deny(unused_imports)]
#![allow(missing_docs)]

// =============================================================================
// Modules fondamentaux (toujours compilés)
// =============================================================================

/// Gestion des erreurs spécifiques au validateur.
pub mod error;

/// Types de base : contrats, secteurs, rapports.
pub mod types;

/// Cœur du validateur Shariah.
pub mod validator;

// =============================================================================
// Réexportations des types principaux
// =============================================================================

pub use types::{
    FinancialContract,
    IndustrySector,
    IslamicContractType,
    ValidationReport,
    ShariahViolation,
    ViolationCode,
};
pub use validator::ShariahValidator;
pub use error::ShariahError;

// =============================================================================
// Modules optionnels (feature-gated)
// =============================================================================

/// Intégration avec Hyperledger Fabric.
/// Activé par la feature `fabric`.
#[cfg(feature = "fabric")]
pub mod fabric;

/// Intégration avec le mixnet Nym (communications anonymes).
/// Activé par la feature `nym`.
#[cfg(feature = "nym")]
pub mod nym;

/// Bindings WebAssembly.
/// Activé par la feature `wasm`.
#[cfg(feature = "wasm")]
pub mod wasm_bindings;

// =============================================================================
// Réexports conditionnels pour simplifier l'usage
// =============================================================================

#[cfg(feature = "fabric")]
pub use fabric::{FabricClientWrapper, validate_with_fabric};

#[cfg(feature = "nym")]
pub use nym::NymClientWrapper;

#[cfg(feature = "wasm")]
pub use wasm_bindings::validate_wasm;

// =============================================================================
// Constantes globales
// =============================================================================

/// Version du crate (provenant de Cargo.toml).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Nom du projet.
pub const PROJECT_NAME: &str = env!("CARGO_PKG_NAME");

/// Description du projet.
pub const PROJECT_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// URL du dépôt.
pub const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

// =============================================================================
// Point d'entrée JSON (toujours disponible)
// =============================================================================

/// Fonction utilitaire pour valider un contrat à partir d'un JSON.
///
/// # Exemple
///
/// ```rust
/// use islamic_shariah_validator::validate_json;
///
/// let json = r#"{
///     "contract_type": "Mudarabah",
///     "industry": "Halal",
///     "principal_amount": 100000,
///     "expected_profit_rate": null,
///     "manager_profit_share": 0.3,
///     "investor_profit_share": 0.7,
///     "maturity_timestamp": 1700000000,
///     "signing_timestamp": 1600000000,
///     "asset_id": "real_estate_001",
///     "collateral_amount": 50000,
///     "is_fixed_term": true,
///     "ambiguous_clauses": []
/// }"#;
///
/// let report = validate_json(json).unwrap();
/// assert!(report.is_valid);
/// ```
pub fn validate_json(json: &str) -> Result<ValidationReport, serde_json::Error> {
    let contract: FinancialContract = serde_json::from_str(json)?;
    let validator = ShariahValidator::new();
    Ok(validator.validate(&contract))
}

// =============================================================================
// Tests unitaires
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_validation() {
        let contract = FinancialContract::default();
        let validator = ShariahValidator::new();
        let report = validator.validate(&contract);
        assert!(report.is_valid);
    }

    #[test]
    fn test_validate_json() {
        let json = r#"{
            "contract_type": "Mudarabah",
            "industry": "Halal",
            "principal_amount": 100000,
            "expected_profit_rate": null,
            "manager_profit_share": 0.3,
            "investor_profit_share": 0.7,
            "maturity_timestamp": 1700000000,
            "signing_timestamp": 1600000000,
            "asset_id": "real_estate_001",
            "collateral_amount": 50000,
            "is_fixed_term": true,
            "ambiguous_clauses": []
        }"#;

        let report = validate_json(json).unwrap();
        assert!(report.is_valid);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_validate_json_invalid() {
        let json = r#"{
            "contract_type": "Mudarabah",
            "industry": "Haram",
            "principal_amount": 100000,
            "expected_profit_rate": null,
            "manager_profit_share": 0.3,
            "investor_profit_share": 0.7,
            "maturity_timestamp": 1700000000,
            "signing_timestamp": 1600000000,
            "asset_id": "real_estate_001",
            "collateral_amount": 50000,
            "is_fixed_term": true,
            "ambiguous_clauses": []
        }"#;

        let report = validate_json(json).unwrap();
        assert!(!report.is_valid);
        assert!(!report.violations.is_empty());
        assert_eq!(report.violations[0].code, ViolationCode::HaramIndustry);
    }

    #[cfg(feature = "fabric")]
    #[test]
    fn test_fabric_module_exists() {
        let _ = FabricClientWrapper::new();
    }

    #[cfg(feature = "nym")]
    #[test]
    fn test_nym_module_exists() {
        let _ = NymClientWrapper::new();
    }
}
