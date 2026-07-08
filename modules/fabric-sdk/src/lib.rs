//! # Islamic Shariah Validator (ISV)
//!
//! Bibliothèque modulaire pour la validation de contrats financiers selon la Charia.
//!
//! ## Features
//!
//! | Feature | Description | Dépendances |
//! |---------|-------------|-------------|
//! | `fabric` | Intégration avec Hyperledger Fabric | `fabric-sdk`, `protobuf` |
//! | `indy`   | Intégration avec Hyperledger Indy   | `indy-sdk`, `indy-credx`, `indy-vdr` |
//! | `wasm`   | Bindings WebAssembly                | `wasm-bindgen`, `getrandom`, `serde-wasm-bindgen` |
//! | `full`   | Active toutes les features          | - |
//!
//! # Exemple d'utilisation (core only)
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
//! # Exemple avec Fabric
//!
//! ```rust,no_run
//! #[cfg(feature = "fabric")]
//! use islamic_shariah_validator::fabric::validate_with_fabric;
//! ```

// =============================================================================
// Modules fondamentaux (toujours compilés)
// =============================================================================

pub mod error;
pub mod types;
pub mod validator;

// Re-export des types principaux pour une utilisation facile
pub use types::{FinancialContract, IndustrySector, IslamicContractType};
pub use validator::{ShariahValidator, ValidationReport, ShariahViolation, ViolationCode};

// =============================================================================
// Modules conditionnels (feature-gated)
// =============================================================================

/// Intégration Hyperledger Fabric (activée par la feature `fabric`)
#[cfg(feature = "fabric")]
pub mod fabric;

/// Intégration Hyperledger Indy (activée par la feature `indy`)
#[cfg(feature = "indy")]
pub mod indy;

/// Bindings WebAssembly (activée par la feature `wasm`)
#[cfg(feature = "wasm")]
pub mod wasm_bindings;

// =============================================================================
// Réexports conditionnels pour faciliter l'usage
// =============================================================================

#[cfg(feature = "fabric")]
pub use fabric::FabricClientWrapper;

#[cfg(feature = "indy")]
pub use indy::IndyCredentialHandler;

// =============================================================================
// Constantes de version
// =============================================================================

/// Version du crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// Nom du projet
pub const PROJECT_NAME: &str = env!("CARGO_PKG_NAME");

// =============================================================================
// Tests (si besoin)
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
}
