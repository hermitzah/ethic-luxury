//! Définitions des erreurs du validateur Shariah.

use crate::types::ViolationCode;
use thiserror::Error;

/// Erreurs spécifiques à la validation Shariah.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ShariahError {
    /// Violation d'une règle de la Charia.
    #[error("Violation de la Charia : {code:?} sur le champ '{field}' - {message}")]
    Violation {
        code: ViolationCode,
        field: String,
        message: String,
    },

    /// Erreur de validation générique (ex: paramètres invalides).
    #[error("Erreur de validation : {0}")]
    ValidationError(String),

    /// Erreur de communication avec Hyperledger Fabric (feature `fabric`).
    #[cfg(feature = "fabric")]
    #[error("Erreur de communication avec Fabric : {0}")]
    FabricError(String),

    /// Erreur de communication avec le mixnet Nym (feature `nym`).
    #[cfg(feature = "nym")]
    #[error("Erreur de communication avec Nym : {0}")]
    NymError(String),

    /// Erreur de sérialisation/désérialisation.
    #[error("Erreur de sérialisation : {0}")]
    SerializationError(String),
}

/// Résultat de validation avec notre type d'erreur.
pub type Result<T> = std::result::Result<T, ShariahError>;

impl From<serde_json::Error> for ShariahError {
    fn from(err: serde_json::Error) -> Self {
        ShariahError::SerializationError(err.to_string())
    }
}

impl From<std::io::Error> for ShariahError {
    fn from(err: std::io::Error) -> Self {
        ShariahError::SerializationError(err.to_string())
    }
}
