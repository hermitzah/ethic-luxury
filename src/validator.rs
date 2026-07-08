//! Cœur du validateur Shariah.
//!
//! Ce module implémente le moteur de validation des contrats financiers
//! selon les principes de la Charia islamique (AAOIFI-compliant).

use crate::types::*;
#[cfg(feature = "fabric")]
use crate::error::{ShariahError, Result};
use std::collections::HashSet;

// ============================================================================
// Validateur principal
// ============================================================================

/// Validateur de conformité à la Charia islamique.
///
/// Effectue des vérifications sur les contrats financiers selon les règles :
/// - Riba (intérêt)
/// - Gharar (incertitude excessive)
/// - Maysir (spéculation)
/// - Asset-Backing (adossement à des actifs)
/// - Profit Sharing (partage des profits)
/// - Halal Industry (secteur d'activité)
#[derive(Debug)]
pub struct ShariahValidator {
    /// Secteurs d'activité considérés comme Haram (interdits).
    haram_industries: HashSet<IndustrySector>,
    /// Seuil maximal d'incertitude (Gharar) autorisé (0.0 à 1.0).
    max_uncertainty_threshold: f64,
}

impl Default for ShariahValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl ShariahValidator {
    /// Crée un nouveau validateur avec les paramètres par défaut.
    ///
    /// # Paramètres par défaut
    /// - Seuil de Gharar : 0.15 (15%)
    /// - Industries Haram : `IndustrySector::Haram`
    ///
    /// # Exemple
    ///
    /// ```rust
    /// use islamic_shariah_validator::ShariahValidator;
    ///
    /// let validator = ShariahValidator::new();
    /// ```
    pub fn new() -> Self {
        let mut haram = HashSet::new();
        haram.insert(IndustrySector::Haram);
        Self {
            haram_industries: haram,
            max_uncertainty_threshold: 0.15,
        }
    }

    /// Crée un validateur avec un seuil de Gharar personnalisé.
    ///
    /// # Arguments
    ///
    /// * `threshold` - Seuil d'incertitude maximum (entre 0.0 et 1.0)
    ///
    /// # Exemple
    ///
    /// ```rust
    /// use islamic_shariah_validator::ShariahValidator;
    ///
    /// let validator = ShariahValidator::with_gharar_threshold(0.20);
    /// ```
    pub fn with_gharar_threshold(threshold: f64) -> Self {
        let mut haram = HashSet::new();
        haram.insert(IndustrySector::Haram);
        Self {
            haram_industries: haram,
            max_uncertainty_threshold: threshold.clamp(0.0, 1.0),
        }
    }

    /// Valide un contrat financier selon les règles de la Charia.
    ///
    /// # Arguments
    ///
    /// * `contract` - Le contrat à valider
    ///
    /// # Retourne
    ///
    /// Un `ValidationReport` contenant :
    /// - `is_valid` : `true` si toutes les vérifications sont passées
    /// - `violations` : Liste des violations détectées
    /// - `warnings` : Avertissements (non bloquants)
    ///
    /// # Exemple
    ///
    /// ```rust
    /// use islamic_shariah_validator::{FinancialContract, ShariahValidator};
    ///
    /// let contract = FinancialContract::default();
    /// let validator = ShariahValidator::new();
    /// let report = validator.validate(&contract);
    /// assert!(report.is_valid);
    /// ```
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

    /// Validation asynchrone (peut interroger des sources externes comme Fabric).
    ///
    /// # Arguments
    ///
    /// * `contract` - Le contrat à valider
    /// * `asset_resolver` - Fonction async pour vérifier l'existence d'un actif
    ///
    /// # Exemple
    ///
    /// ```rust,no_run
    /// #[cfg(feature = "fabric")]
    /// # async fn example() {
    /// use islamic_shariah_validator::{FinancialContract, ShariahValidator};
    ///
    /// let contract = FinancialContract::default();
    /// let validator = ShariahValidator::new();
    /// let resolver = |asset_id: &str| async move { Ok(true) };
    /// let report = validator.validate_async(&contract, resolver).await.unwrap();
    /// # }
    /// ```
    #[cfg(feature = "fabric")]
    pub async fn validate_async<F, Fut>(
        &self,
        contract: &FinancialContract,
        asset_resolver: F,
    ) -> Result<ValidationReport>
    where
        F: Fn(&str) -> Fut,
        Fut: std::future::Future<Output = Result<bool>>,
    {
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

    // ========================================================================
    // Règles de validation individuelles
    // ========================================================================

    /// Vérifie que le secteur d'activité n'est pas Haram.
    fn validate_industry(&self, contract: &FinancialContract, violations: &mut Vec<ShariahViolation>) {
        if self.haram_industries.contains(&contract.industry) {
            violations.push(ShariahViolation {
                code: ViolationCode::HaramIndustry,
                field: "industry".to_string(),
                message: "Secteur Haram (interdit) : les activités liées à l'alcool, au porc, aux jeux d'argent, à la pornographie, etc. sont interdites.".to_string(),
            });
        }
        if contract.industry == IndustrySector::Doubtful {
            violations.push(ShariahViolation {
                code: ViolationCode::HaramIndustry,
                field: "industry".to_string(),
                message: "Secteur douteux (Doubtful) : ce secteur doit être clarifié ou évité par précaution.".to_string(),
            });
        }
    }

    /// Vérifie l'absence de Riba (intérêt fixe garanti).
    fn validate_riba(&self, contract: &FinancialContract, violations: &mut Vec<ShariahViolation>) {
        if let Some(rate) = contract.expected_profit_rate {
            if rate > 0.0 && contract.contract_type != IslamicContractType::Murabaha {
                violations.push(ShariahViolation {
                    code: ViolationCode::RibaDetected,
                    field: "expected_profit_rate".to_string(),
                    message: format!("Taux de profit fixe de {}% interdit (Riba) pour ce type de contrat. Le profit doit être partagé ou basé sur un actif tangible.", rate),
                });
            } else if rate > 0.0
                && contract.contract_type == IslamicContractType::Murabaha
                && contract.asset_id.is_none()
            {
                violations.push(ShariahViolation {
                    code: ViolationCode::RibaDetected,
                    field: "expected_profit_rate".to_string(),
                    message: "Profit fixe sans actif tangible = Riba. La Murabaha doit être adossée à un actif réel.".to_string(),
                });
            }
        }
        if let (Some(mgr), Some(inv)) = (contract.manager_profit_share, contract.investor_profit_share) {
            if (mgr + inv) > 1.0 + 1e-6 {
                violations.push(ShariahViolation {
                    code: ViolationCode::RibaDetected,
                    field: "profit_shares".to_string(),
                    message: format!(
                        "Somme des parts de profit ({:.2} + {:.2}) > 100%. Cela peut cacher un intérêt garanti.",
                        mgr, inv
                    ),
                });
            }
        }
    }

    /// Vérifie que le contrat n'a pas une incertitude excessive (Gharar).
    fn validate_gharar(&self, contract: &FinancialContract, violations: &mut Vec<ShariahViolation>, warnings: &mut Vec<String>) {
        let mut uncertainty = 0.0;

        // Clauses ambiguës
        if !contract.ambiguous_clauses.is_empty() {
            uncertainty += 0.1 * contract.ambiguous_clauses.len() as f64;
        }

        // Absence d'actif identifié (sauf pour Salam qui est une vente à terme)
        if contract.asset_id.is_none() && contract.contract_type != IslamicContractType::Salam {
            uncertainty += 0.3;
        }

        // Durée indéterminée
        if !contract.is_fixed_term {
            uncertainty += 0.2;
            warnings.push("Le contrat n'a pas de terme fixe, ce qui augmente le Gharar.".to_string());
        }

        // Échéance trop lointaine (> 5 ans)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if contract.maturity_timestamp > now + 5 * 365 * 24 * 60 * 60 {
            uncertainty += 0.2;
            warnings.push("Échéance très lointaine (> 5 ans) : assurez-vous que les termes sont extrêmement clairs.".to_string());
        }

        if uncertainty > self.max_uncertainty_threshold {
            violations.push(ShariahViolation {
                code: ViolationCode::GhararExcessive,
                field: "contract_terms".to_string(),
                message: format!(
                    "Niveau d'incertitude (Gharar) élevé : {:.2} (seuil max : {:.2}).",
                    uncertainty, self.max_uncertainty_threshold
                ),
            });
        }
    }

    /// Vérifie l'absence de Maysir (spéculation / jeu de hasard).
    fn validate_maysir(&self, contract: &FinancialContract, violations: &mut Vec<ShariahViolation>) {
        if contract.asset_id.is_none() && contract.expected_profit_rate.is_some() {
            violations.push(ShariahViolation {
                code: ViolationCode::MaysirDetected,
                field: "asset_id".to_string(),
                message: "Profit attendu sans actif sous-jacent identifié. Cela s'apparente à du Maysir (spéculation pure).".to_string(),
            });
        }
        if contract.contract_type == IslamicContractType::Salam && contract.asset_id.is_none() {
            violations.push(ShariahViolation {
                code: ViolationCode::MaysirDetected,
                field: "asset_id".to_string(),
                message: "Un contrat Salam doit obligatoirement spécifier l'actif à livrer, sinon c'est du Maysir.".to_string(),
            });
        }
    }

    /// Vérifie que le contrat est adossé à un actif tangible.
    fn validate_asset_backing(&self, contract: &FinancialContract, violations: &mut Vec<ShariahViolation>) {
        let requires_backing =
            !matches!(contract.contract_type, IslamicContractType::Salam | IslamicContractType::Istisna);

        if requires_backing && contract.asset_id.is_none() {
            violations.push(ShariahViolation {
                code: ViolationCode::NoAssetBacking,
                field: "asset_id".to_string(),
                message: "Pas d'actif tangible associé. Asset-Backing requis pour ce type de contrat.".to_string(),
            });
        }

        if contract.collateral_amount == 0 && contract.asset_id.is_none() {
            violations.push(ShariahViolation {
                code: ViolationCode::NoAssetBacking,
                field: "collateral_amount".to_string(),
                message: "Aucun collatéral fourni pour un contrat sans actif tangible. Violation du principe d'Asset-Backing.".to_string(),
            });
        }
    }

    /// Vérifie que les ratios de partage des profits sont valides.
    fn validate_profit_sharing_ratios(&self, contract: &FinancialContract, violations: &mut Vec<ShariahViolation>) {
        match contract.contract_type {
            IslamicContractType::Mudarabah | IslamicContractType::Musharakah => {
                let (mgr, inv) = match (contract.manager_profit_share, contract.investor_profit_share) {
                    (Some(m), Some(i)) => (m, i),
                    _ => {
                        violations.push(ShariahViolation {
                            code: ViolationCode::InvalidProfitRatio,
                            field: "profit_sharing".to_string(),
                            message: "Parts de profit (manager et investisseur) requises pour ce type de contrat.".to_string(),
                        });
                        return;
                    }
                };

                if mgr <= 0.0 || inv <= 0.0 {
                    violations.push(ShariahViolation {
                        code: ViolationCode::InvalidProfitRatio,
                        field: "profit_sharing".to_string(),
                        message: "Les parts de profit doivent être strictement positives (> 0).".to_string(),
                    });
                }

                if (mgr + inv - 1.0).abs() > 1e-6 {
                    violations.push(ShariahViolation {
                        code: ViolationCode::InvalidProfitRatio,
                        field: "profit_sharing".to_string(),
                        message: format!(
                            "Somme des parts de profit ({:.2} + {:.2}) doit être égale à 1.0 (100%).",
                            mgr, inv
                        ),
                    });
                }
            }
            _ => {
                // Les autres types de contrats n'ont pas de partage de profit explicite
            }
        }
    }

    /// Vérifie la clarté et la précision des termes du contrat.
    fn validate_clarity(&self, contract: &FinancialContract, violations: &mut Vec<ShariahViolation>) {
        if contract.signing_timestamp == 0 || contract.maturity_timestamp == 0 {
            violations.push(ShariahViolation {
                code: ViolationCode::UnclearTerms,
                field: "timestamps".to_string(),
                message: "Dates de signature ou d'échéance manquantes ou nulles. Clause ambiguë.".to_string(),
            });
        }

        if contract.signing_timestamp >= contract.maturity_timestamp {
            violations.push(ShariahViolation {
                code: ViolationCode::UnclearTerms,
                field: "maturity_timestamp".to_string(),
                message: "La date d'échéance doit être postérieure à la date de signature.".to_string(),
            });
        }

        // Détection des mots vagues dans les clauses ambiguës
        let vague_words = ["environ", "peut-être", "si possible", "approximativement", "selon"];
        for clause in &contract.ambiguous_clauses {
            let lower = clause.to_lowercase();
            if vague_words.iter().any(|w| lower.contains(w)) {
                violations.push(ShariahViolation {
                    code: ViolationCode::UnclearTerms,
                    field: "ambiguous_clauses".to_string(),
                    message: format!(
                        "Clause ambiguë détectée : '{}'. Contient des termes vagues interdits par la Charia.",
                        clause
                    ),
                });
            }
        }
    }
}

// ============================================================================
// Tests unitaires
// ============================================================================

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
    fn test_valid_contract() {
        let validator = ShariahValidator::new();
        let report = validator.validate(&valid_contract());
        assert!(report.is_valid);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_haram_industry() {
        let mut contract = valid_contract();
        contract.industry = IndustrySector::Haram;
        let validator = ShariahValidator::new();
        let report = validator.validate(&contract);
        assert!(!report.is_valid);
        assert_eq!(report.violations[0].code, ViolationCode::HaramIndustry);
    }

    #[test]
    fn test_doubtful_industry() {
        let mut contract = valid_contract();
        contract.industry = IndustrySector::Doubtful;
        let validator = ShariahValidator::new();
        let report = validator.validate(&contract);
        assert!(!report.is_valid);
        assert_eq!(report.violations[0].code, ViolationCode::HaramIndustry);
    }

    #[test]
    fn test_riba_fixed_rate() {
        let mut contract = valid_contract();
        contract.expected_profit_rate = Some(5.0);
        contract.asset_id = None;
        let validator = ShariahValidator::new();
        let report = validator.validate(&contract);
        assert!(!report.is_valid);
        assert_eq!(report.violations[0].code, ViolationCode::RibaDetected);
    }

    #[test]
    fn test_riba_murabaha_valid() {
        let mut contract = valid_contract();
        contract.contract_type = IslamicContractType::Murabaha;
        contract.expected_profit_rate = Some(3.0);
        contract.asset_id = Some("car_001".to_string());
        let validator = ShariahValidator::new();
        let report = validator.validate(&contract);
        assert!(report.is_valid);
    }

    #[test]
    fn test_gharar_excessive() {
        let mut contract = valid_contract();
        contract.ambiguous_clauses = vec![
            "livraison environ".to_string(),
            "prix peut-être".to_string(),
        ];
        contract.is_fixed_term = false;
        contract.asset_id = None;
        let validator = ShariahValidator::new();
        let report = validator.validate(&contract);
        assert!(!report.is_valid);
        assert_eq!(report.violations[0].code, ViolationCode::GhararExcessive);
    }

    #[test]
    fn test_maysir() {
        let mut contract = valid_contract();
        contract.asset_id = None;
        contract.expected_profit_rate = Some(10.0);
        let validator = ShariahValidator::new();
        let report = validator.validate(&contract);
        assert!(!report.is_valid);
        // La première violation peut être Riba ou Maysir selon l'ordre d'exécution
        // On accepte les deux
        let code = &report.violations[0].code;
        assert!(matches!(code, ViolationCode::RibaDetected | ViolationCode::MaysirDetected));
    }

    #[test]
    fn test_no_asset_backing() {
        let mut contract = valid_contract();
        contract.asset_id = None;
        contract.collateral_amount = 0;
        // On utilise un seuil de Gharar élevé pour éviter que Gharar ne masque NoAssetBacking
        let validator = ShariahValidator::with_gharar_threshold(0.5);
        let report = validator.validate(&contract);
        assert!(!report.is_valid);
        assert_eq!(report.violations[0].code, ViolationCode::NoAssetBacking);
    }

    #[test]
    fn test_invalid_profit_ratios() {
        let mut contract = valid_contract();
        contract.manager_profit_share = Some(0.8);
        contract.investor_profit_share = Some(0.8);
        let validator = ShariahValidator::new();
        let report = validator.validate(&contract);
        assert!(!report.is_valid);
        assert_eq!(report.violations[0].code, ViolationCode::RibaDetected);
    }

    #[test]
    fn test_unclear_terms() {
        let mut contract = valid_contract();
        contract.signing_timestamp = 1_700_000_000;
        contract.maturity_timestamp = 1_600_000_000;
        let validator = ShariahValidator::new();
        let report = validator.validate(&contract);
        assert!(!report.is_valid);
        assert_eq!(report.violations[0].code, ViolationCode::UnclearTerms);
    }

    #[test]
    fn test_ambiguous_clause_detected() {
        let mut contract = valid_contract();
        contract.ambiguous_clauses = vec!["livraison approximativement".to_string()];
        let validator = ShariahValidator::new();
        let report = validator.validate(&contract);
        assert!(!report.is_valid);
        assert_eq!(report.violations[0].code, ViolationCode::UnclearTerms);
    }

    #[test]
    fn test_custom_gharar_threshold() {
        let mut contract = valid_contract();
        contract.ambiguous_clauses = vec!["clause ambiguë".to_string()];
        contract.is_fixed_term = false;
        // On ajoute un asset_id pour réduire l'incertitude (sinon 0.6 > 0.5)
        contract.asset_id = Some("asset_123".to_string());

        let validator = ShariahValidator::with_gharar_threshold(0.5);
        let report = validator.validate(&contract);
        assert!(report.is_valid); // Le seuil plus élevé accepte le contrat
    }

    #[test]
    fn test_salam_without_asset() {
        let mut contract = valid_contract();
        contract.contract_type = IslamicContractType::Salam;
        contract.asset_id = None;
        let validator = ShariahValidator::new();
        let report = validator.validate(&contract);
        assert!(!report.is_valid);
        assert_eq!(report.violations[0].code, ViolationCode::MaysirDetected);
    }

    #[test]
    fn test_missing_profit_shares() {
        let mut contract = valid_contract();
        contract.manager_profit_share = None;
        contract.investor_profit_share = None;
        let validator = ShariahValidator::new();
        let report = validator.validate(&contract);
        assert!(!report.is_valid);
        assert_eq!(report.violations[0].code, ViolationCode::InvalidProfitRatio);
    }
}
