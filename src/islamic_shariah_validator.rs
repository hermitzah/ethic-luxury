//! Validateur de conformité à la Charia islamique (Shariah) pour transactions financières.
//! Supporte Hyperledger Fabric, Indy et WASM via les features.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

// ------------------------------------------------------------
// 1. Erreurs personnalisées
// ------------------------------------------------------------

/// Erreurs spécifiques à la validation Shariah.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ShariahError {
    #[error("Violation de la Charia : {code:?} sur le champ '{field}' - {message}")]
    Violation {
        code: ViolationCode,
        field: String,
        message: String,
    },
    #[error("Erreur de validation : {0}")]
    ValidationError(String),
    #[cfg(feature = "fabric")]
    #[error("Erreur de communication avec Fabric : {0}")]
    FabricError(String),
    #[cfg(feature = "indy")]
    #[error("Erreur Indy : {0}")]
    IndyError(String),
}

// ------------------------------------------------------------
// 2. Types de base
// ------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IndustrySector {
    Halal,
    Haram,
    Doubtful,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IslamicContractType {
    Mudarabah,
    Musharakah,
    Murabaha,
    Ijarah,
    Salam,
    Istisna,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShariahViolation {
    pub code: ViolationCode,
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub is_valid: bool,
    pub violations: Vec<ShariahViolation>,
    pub warnings: Vec<String>,
}

// ------------------------------------------------------------
// 3. Contrat à valider
// ------------------------------------------------------------

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

// ------------------------------------------------------------
// 4. Validateur principal
// ------------------------------------------------------------

#[derive(Debug)]
pub struct ShariahValidator {
    haram_industries: HashSet<IndustrySector>,
    max_uncertainty_threshold: f64,
}

impl Default for ShariahValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl ShariahValidator {
    pub fn new() -> Self {
        let mut haram = HashSet::new();
        haram.insert(IndustrySector::Haram);
        Self {
            haram_industries: haram,
            max_uncertainty_threshold: 0.15,
        }
    }

    /// Validation synchrone (pour usage général)
    pub fn validate(&self, contract: &FinancialContract) -> ValidationReport {
        let mut violations = Vec::new();
        let mut warnings = Vec::new();

        self.validate_industry(contract, &mut violations);
        self.validate_riba(contract, &mut violations);
        self.validate_gharar(contract, &mut violations, &mut warnings);
        self.validate_maysir(contract, &mut violations);
        self.validate_asset_backing(contract, &mut violations);
        self.validate_profit_sharing_ratios(contract, &mut violations);
        self.validate_clarity(contract, &mut violations);

        ValidationReport {
            is_valid: violations.is_empty(),
            violations,
            warnings,
        }
    }

    /// Validation asynchrone (peut interroger des sources externes)
    #[cfg(feature = "fabric")]
    pub async fn validate_async<F, Fut>(
        &self,
        contract: &FinancialContract,
        asset_resolver: F,
    ) -> Result<ValidationReport, ShariahError>
    where
        F: Fn(&str) -> Fut,
        Fut: std::future::Future<Output = Result<bool, ShariahError>>,
    {
        // Exemple : vérifier que l'actif existe sur le ledger Fabric
        if let Some(asset_id) = &contract.asset_id {
            let exists = asset_resolver(asset_id).await?;
            if !exists {
                return Err(ShariahError::Violation {
                    code: ViolationCode::NoAssetBacking,
                    field: "asset_id".to_string(),
                    message: format!("L'actif '{}' n'existe pas sur le ledger.", asset_id),
                });
            }
        }
        Ok(self.validate(contract))
    }

    // --- Méthodes internes de validation ---

    fn validate_industry(&self, contract: &FinancialContract, violations: &mut Vec<ShariahViolation>) {
        if self.haram_industries.contains(&contract.industry) {
            violations.push(ShariahViolation {
                code: ViolationCode::HaramIndustry,
                field: "industry".to_string(),
                message: "Secteur Haram (interdit).".to_string(),
            });
        }
        if contract.industry == IndustrySector::Doubtful {
            violations.push(ShariahViolation {
                code: ViolationCode::HaramIndustry,
                field: "industry".to_string(),
                message: "Secteur douteux, doit être clarifié ou évité.".to_string(),
            });
        }
    }

    fn validate_riba(&self, contract: &FinancialContract, violations: &mut Vec<ShariahViolation>) {
        if let Some(rate) = contract.expected_profit_rate {
            if rate > 0.0 && contract.contract_type != IslamicContractType::Murabaha {
                violations.push(ShariahViolation {
                    code: ViolationCode::RibaDetected,
                    field: "expected_profit_rate".to_string(),
                    message: format!("Taux fixe de {}% interdit (Riba) pour ce type de contrat.", rate),
                });
            } else if rate > 0.0 && contract.contract_type == IslamicContractType::Murabaha && contract.asset_id.is_none() {
                violations.push(ShariahViolation {
                    code: ViolationCode::RibaDetected,
                    field: "expected_profit_rate".to_string(),
                    message: "Profit fixe sans actif tangible = Riba.".to_string(),
                });
            }
        }
        if let (Some(mgr), Some(inv)) = (contract.manager_profit_share, contract.investor_profit_share) {
            if (mgr + inv) > 1.0 + 1e-6 {
                violations.push(ShariahViolation {
                    code: ViolationCode::RibaDetected,
                    field: "profit_shares",
                    message: "Somme des parts > 100%, cela peut cacher un intérêt.".to_string(),
                });
            }
        }
    }

    fn validate_gharar(&self, contract: &FinancialContract, violations: &mut Vec<ShariahViolation>, warnings: &mut Vec<String>) {
        let mut uncertainty = 0.0;
        if !contract.ambiguous_clauses.is_empty() {
            uncertainty += 0.1 * contract.ambiguous_clauses.len() as f64;
        }
        if contract.asset_id.is_none() && contract.contract_type != IslamicContractType::Salam {
            uncertainty += 0.3;
        }
        if !contract.is_fixed_term {
            uncertainty += 0.2;
            warnings.push("Terme non fixe → Gharar accru.".to_string());
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if contract.maturity_timestamp > now + 5 * 365 * 24 * 60 * 60 {
            uncertainty += 0.2;
            warnings.push("Échéance > 5 ans, termes à clarifier.".to_string());
        }
        if uncertainty > self.max_uncertainty_threshold {
            violations.push(ShariahViolation {
                code: ViolationCode::GhararExcessive,
                field: "contract_terms",
                message: format!("Incertitude {:.2} > seuil {:.2}.", uncertainty, self.max_uncertainty_threshold),
            });
        }
    }

    fn validate_maysir(&self, contract: &FinancialContract, violations: &mut Vec<ShariahViolation>) {
        if contract.asset_id.is_none() && contract.expected_profit_rate.is_some() {
            violations.push(ShariahViolation {
                code: ViolationCode::MaysirDetected,
                field: "asset_id",
                message: "Profit attendu sans actif sous-jacent = jeu de hasard (Maysir).".to_string(),
            });
        }
        if contract.contract_type == IslamicContractType::Salam && contract.asset_id.is_none() {
            violations.push(ShariahViolation {
                code: ViolationCode::MaysirDetected,
                field: "asset_id",
                message: "Salam nécessite un actif à livrer, sinon c'est du Maysir.".to_string(),
            });
        }
    }

    fn validate_asset_backing(&self, contract: &FinancialContract, violations: &mut Vec<ShariahViolation>) {
        let requires_backing = !matches!(contract.contract_type, IslamicContractType::Salam | IslamicContractType::Istisna);
        if requires_backing && contract.asset_id.is_none() {
            violations.push(ShariahViolation {
                code: ViolationCode::NoAssetBacking,
                field: "asset_id",
                message: "Pas d'actif tangible → Asset-Backing manquant.".to_string(),
            });
        }
        if contract.collateral_amount == 0 && contract.asset_id.is_none() {
            violations.push(ShariahViolation {
                code: ViolationCode::NoAssetBacking,
                field: "collateral_amount",
                message: "Aucun collatéral pour un contrat sans actif tangible.".to_string(),
            });
        }
    }

    fn validate_profit_sharing_ratios(&self, contract: &FinancialContract, violations: &mut Vec<ShariahViolation>) {
        match contract.contract_type {
            IslamicContractType::Mudarabah | IslamicContractType::Musharakah => {
                let (mgr, inv) = match (contract.manager_profit_share, contract.investor_profit_share) {
                    (Some(m), Some(i)) => (m, i),
                    _ => {
                        violations.push(ShariahViolation {
                            code: ViolationCode::InvalidProfitRatio,
                            field: "profit_sharing",
                            message: "Parts de profit manquantes pour ce type de contrat.".to_string(),
                        });
                        return;
                    }
                };
                if mgr <= 0.0 || inv <= 0.0 {
                    violations.push(ShariahViolation {
                        code: ViolationCode::InvalidProfitRatio,
                        field: "profit_sharing",
                        message: "Les parts doivent être strictement positives.".to_string(),
                    });
                }
                if (mgr + inv - 1.0).abs() > 1e-6 {
                    violations.push(ShariahViolation {
                        code: ViolationCode::InvalidProfitRatio,
                        field: "profit_sharing",
                        message: format!("Somme {:.2}+{:.2} ≠ 1.0", mgr, inv),
                    });
                }
            }
            _ => {}
        }
    }

    fn validate_clarity(&self, contract: &FinancialContract, violations: &mut Vec<ShariahViolation>) {
        if contract.signing_timestamp == 0 || contract.maturity_timestamp == 0 {
            violations.push(ShariahViolation {
                code: ViolationCode::UnclearTerms,
                field: "timestamps",
                message: "Dates de signature ou d'échéance manquantes.".to_string(),
            });
        }
        if contract.signing_timestamp >= contract.maturity_timestamp {
            violations.push(ShariahViolation {
                code: ViolationCode::UnclearTerms,
                field: "maturity_timestamp",
                message: "Échéance doit être postérieure à la signature.".to_string(),
            });
        }
        let vague = ["environ", "peut-être", "si possible", "approximativement", "selon"];
        for clause in &contract.ambiguous_clauses {
            let lower = clause.to_lowercase();
            if vague.iter().any(|w| lower.contains(w)) {
                violations.push(ShariahViolation {
                    code: ViolationCode::UnclearTerms,
                    field: "ambiguous_clauses",
                    message: format!("Clause vague : '{}'.", clause),
                });
            }
        }
    }
}

// ------------------------------------------------------------
// 5. Implémentation par défaut pour FinancialContract
// ------------------------------------------------------------

impl Default for FinancialContract {
    fn default() -> Self {
        Self {
            contract_type: IslamicContractType::Mudarabah,
            industry: IndustrySector::Halal,
            principal_amount: 0,
            expected_profit_rate: None,
            manager_profit_share: Some(0.3),
            investor_profit_share: Some(0.7),
            maturity_timestamp: 0,
            signing_timestamp: 0,
            asset_id: Some("asset_123".to_string()),
            collateral_amount: 1000,
            is_fixed_term: true,
            ambiguous_clauses: Vec::new(),
        }
    }
}

// ------------------------------------------------------------
// 6. Intégration WASM (feature)
// ------------------------------------------------------------
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn validate_wasm(json: &str) -> Result<String, JsValue> {
    let contract: FinancialContract = serde_json::from_str(json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let validator = ShariahValidator::new();
    let report = validator.validate(&contract);
    serde_json::to_string(&report).map_err(|e| JsValue::from_str(&e.to_string()))
}

// ------------------------------------------------------------
// 7. Tests unitaires
// ------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn valid_contract() -> FinancialContract {
        FinancialContract {
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

    #[test]
    fn test_valid() {
        let validator = ShariahValidator::new();
        let report = validator.validate(&valid_contract());
        assert!(report.is_valid);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_haram() {
        let mut c = valid_contract();
        c.industry = IndustrySector::Haram;
        let report = ShariahValidator::new().validate(&c);
        assert!(!report.is_valid);
        assert_eq!(report.violations[0].code, ViolationCode::HaramIndustry);
    }

    #[test]
    fn test_riba() {
        let mut c = valid_contract();
        c.expected_profit_rate = Some(5.0);
        c.asset_id = None;
        let report = ShariahValidator::new().validate(&c);
        assert!(!report.is_valid);
        assert_eq!(report.violations[0].code, ViolationCode::RibaDetected);
    }

    #[test]
    fn test_gharar() {
        let mut c = valid_contract();
        c.ambiguous_clauses = vec!["livraison environ".to_string()];
        c.is_fixed_term = false;
        c.asset_id = None;
        let report = ShariahValidator::new().validate(&c);
        assert!(!report.is_valid);
        assert_eq!(report.violations[0].code, ViolationCode::GhararExcessive);
    }

    #[test]
    fn test_maysir() {
        let mut c = valid_contract();
        c.asset_id = None;
        c.expected_profit_rate = Some(10.0);
        let report = ShariahValidator::new().validate(&c);
        assert!(!report.is_valid);
        assert_eq!(report.violations[0].code, ViolationCode::MaysirDetected);
    }
}
