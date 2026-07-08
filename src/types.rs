//! Types de base pour les contrats financiers et les rapports de validation.

use serde::{Deserialize, Serialize};

// ----------------------------------------------------------------------------
// Secteur d'activité
// ----------------------------------------------------------------------------

/// Définit si un secteur d'activité est Halal, Haram ou Douteux.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IndustrySector {
    Halal,
    Haram,
    Doubtful,
}

// ----------------------------------------------------------------------------
// Type de contrat islamique
// ----------------------------------------------------------------------------

/// Types de contrats financiers islamiques reconnus.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IslamicContractType {
    Mudarabah,
    Musharakah,
    Murabaha,
    Ijarah,
    Salam,
    Istisna,
}

// ----------------------------------------------------------------------------
// Codes de violation (avec Copy pour permettre l'affichage direct)
// ----------------------------------------------------------------------------

/// Codes identifiant les différentes violations des règles Shariah.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationCode {
    RibaDetected,
    GhararExcessive,
    MaysirDetected,
    NoAssetBacking,
    HaramIndustry,
    InvalidProfitRatio,
    UnclearTerms,
    TransactionUncertainty,
}

// ----------------------------------------------------------------------------
// Structure de violation
// ----------------------------------------------------------------------------

/// Représente une violation spécifique d'une règle Shariah.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShariahViolation {
    pub code: ViolationCode,
    pub field: String,
    pub message: String,
}

// ----------------------------------------------------------------------------
// Rapport de validation
// ----------------------------------------------------------------------------

/// Rapport complet retourné par le validateur Shariah.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub is_valid: bool,
    pub violations: Vec<ShariahViolation>,
    pub warnings: Vec<String>,
}

// ----------------------------------------------------------------------------
// Contrat financier (à valider)
// ----------------------------------------------------------------------------

/// Contrat financier à soumettre au validateur Shariah.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialContract {
    pub contract_type: IslamicContractType,
    pub industry: IndustrySector,
    pub principal_amount: u64,
    pub expected_profit_rate: Option<f64>,
    pub manager_profit_share: Option<f64>,
    pub investor_profit_share: Option<f64>,
    pub maturity_timestamp: u64,
    pub signing_timestamp: u64,
    pub asset_id: Option<String>,
    pub collateral_amount: u64,
    pub is_fixed_term: bool,
    pub ambiguous_clauses: Vec<String>,
}

impl Default for FinancialContract {
    fn default() -> Self {
        Self {
            contract_type: IslamicContractType::Mudarabah,
            industry: IndustrySector::Halal,
            principal_amount: 100_000,
            expected_profit_rate: None,
            manager_profit_share: Some(0.3),
            investor_profit_share: Some(0.7),
            maturity_timestamp: 1_700_000_000,
            signing_timestamp: 1_600_000_000,
            asset_id: Some("real_estate_001".to_string()),
            collateral_amount: 50_000,
            is_fixed_term: true,
            ambiguous_clauses: vec![],
        }
    }
}
