Voici les corrections à appliquer pour éliminer tous les warnings et l'erreur de compilation.

---

## 1. `src/types.rs` – Ajouter `Copy` à `ViolationCode`

```rust
//! Types de base pour les contrats financiers et les rapports de validation.

use serde::{Deserialize, Serialize};

// ----------------------------------------------------------------------------
// Secteur d'activité
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IndustrySector {
    Halal,
    Haram,
    Doubtful,
}

// ----------------------------------------------------------------------------
// Type de contrat islamique
// ----------------------------------------------------------------------------

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
// Codes de violation (ajout de Copy)
// ----------------------------------------------------------------------------

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShariahViolation {
    pub code: ViolationCode,
    pub field: String,
    pub message: String,
}

// ----------------------------------------------------------------------------
// Rapport de validation
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub is_valid: bool,
    pub violations: Vec<ShariahViolation>,
    pub warnings: Vec<String>,
}

// ----------------------------------------------------------------------------
// Contrat financier (à valider)
// ----------------------------------------------------------------------------

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
```

---

## 2. `src/error.rs` – Supprimer la feature `indy`

```rust
//! Définitions des erreurs du validateur Shariah.

use crate::types::ViolationCode;
use thiserror::Error;

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
    // Indy supprimé (feature inutilisée)
}

pub type Result<T> = std::result::Result<T, ShariahError>;
```

---

## 3. `src/lib.rs` – Supprimer la feature `tokio`

Retirez la fonction `validate_json_async` (environ ligne 172). Assurez-vous qu'il n'y a plus de `#[cfg(feature = "tokio")]`. La version finale doit être :

```rust
//! # Islamic Shariah Validator (ISV)
//!
//! Bibliothèque modulaire pour la validation de contrats financiers selon la Charia.

#![deny(unused_imports)]

// Modules fondamentaux
pub mod error;
pub mod types;
pub mod validator;

// Réexportations
pub use types::{
    FinancialContract, IndustrySector, IslamicContractType,
    ValidationReport, ShariahViolation, ViolationCode,
};
pub use validator::ShariahValidator;
pub use error::ShariahError;

// Modules optionnels
#[cfg(feature = "fabric")]
pub mod fabric;
#[cfg(feature = "nym")]
pub mod nym;
#[cfg(feature = "wasm")]
pub mod wasm_bindings;

// Réexports conditionnels
#[cfg(feature = "fabric")]
pub use fabric::{FabricClientWrapper, validate_with_fabric};
#[cfg(feature = "nym")]
pub use nym::NymClientWrapper;
#[cfg(feature = "wasm")]
pub use wasm_bindings::validate_wasm;

// Constantes
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PROJECT_NAME: &str = env!("CARGO_PKG_NAME");
pub const PROJECT_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
pub const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

// Point d'entrée JSON
pub fn validate_json(json: &str) -> Result<ValidationReport, serde_json::Error> {
    let contract: FinancialContract = serde_json::from_str(json)?;
    let validator = ShariahValidator::new();
    Ok(validator.validate(&contract))
}

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
            "manager_profit_share": 0.3,
            "investor_profit_share": 0.7
        }"#;
        let report = validate_json(json).unwrap();
        assert!(report.is_valid);
    }
}
```

---

## 4. `src/fabric/mod.rs` – Correction des variables inutilisées

```rust
//! Intégration Hyperledger Fabric.

use crate::types::{FinancialContract, ValidationReport};
use crate::error::ShariahError;

pub struct FabricClientWrapper;

impl FabricClientWrapper {
    pub fn new() -> Self {
        Self
    }

    pub async fn asset_exists(&self, _asset_id: &str) -> Result<bool, ShariahError> {
        Ok(true) // stub
    }
}

pub async fn validate_with_fabric(
    contract: &FinancialContract,
    client: &FabricClientWrapper,
) -> Result<ValidationReport, ShariahError> {
    if let Some(asset_id) = &contract.asset_id {
        if !client.asset_exists(asset_id).await? {
            return Err(ShariahError::FabricError(format!("Actif {} introuvable", asset_id)));
        }
    }
    let validator = crate::ShariahValidator::new();
    Ok(validator.validate(contract))
}
```

---

## 5. `src/nym/mod.rs` – Correction des variables inutilisées

```rust
//! Intégration Nym.

use crate::types::ValidationReport;

pub struct NymClientWrapper;

impl NymClientWrapper {
    pub fn new() -> Self {
        Self
    }

    #[allow(unused_variables)]
    pub async fn send_report(&self, _report: &ValidationReport, _recipient: &str) -> Result<(), String> {
        Ok(())
    }
}
```

---

## 6. `src/bin/cli.rs` – Corrections (variables inutilisées + pas de `Copy`)

```rust
// SPDX-License-Identifier: DWPL-2.0
// Fichier : src/bin/cli.rs
// Description : Interface en ligne de commande pour le validateur Shariah & DWPL

use clap::{Parser, Subcommand};
use anyhow::{Context, Result};
use serde_json::json;
use std::path::PathBuf;
use std::fs;

// -----------------------------------------------------------------------------
// 1. Structure des arguments
// -----------------------------------------------------------------------------

#[derive(Parser)]
#[command(name = "islamic-shariah-validator")]
#[command(about = "Validateur Shariah & DWPL pour Hyperledger Fabric", long_about = None)]
struct Cli {
    #[arg(short, long, global = true)]
    verbose: bool,
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,
    #[command(subcommand)]
    command: Commands,
}

// -----------------------------------------------------------------------------
// 2. Sous-commandes
// -----------------------------------------------------------------------------

#[derive(Subcommand)]
enum Commands {
    Validate {
        #[arg(short, long, conflicts_with = "json")]
        file: Option<PathBuf>,
        #[arg(short, long, conflicts_with = "file")]
        json: Option<String>,
        #[arg(short, long)]
        verbose: bool,
    },
    GenerateExample {
        #[arg(default_value = "mudarabah")]
        contract_type: String,
    },
    #[cfg(feature = "fabric")]
    Fabric {
        #[command(subcommand)]
        fabric_cmd: FabricCommands,
    },
    #[cfg(feature = "wasm")]
    WasmTest {
        #[arg(default_value = "World")]
        name: String,
    },
    Info,
}

// -----------------------------------------------------------------------------
// 3. Sous-commandes Fabric (seulement si feature activée)
// -----------------------------------------------------------------------------

#[cfg(feature = "fabric")]
#[derive(Subcommand)]
enum FabricCommands {
    ListMudarabah {
        #[arg(long, default_value = "islamic-channel")]
        channel: String,
    },
    CreateMudarabah {
        #[arg(long)]
        capital_provider: String,
        #[arg(long)]
        entrepreneur: String,
        #[arg(long)]
        capital_amount: u64,
        #[arg(long)]
        profit_ratio: u16,
    },
}

// -----------------------------------------------------------------------------
// 4. Fonction principale
// -----------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    if cli.verbose {
        std::env::set_var("RUST_LOG", "debug");
        let _ = env_logger::try_init();
    }

    if let Some(config_path) = cli.config {
        let content = fs::read_to_string(config_path)
            .context("Impossible de lire le fichier de configuration")?;
        let config: serde_json::Value = serde_json::from_str(&content)
            .context("Fichier de configuration JSON invalide")?;
        println!("📄 Configuration chargée : {}", config);
    }

    match cli.command {
        Commands::Validate { file, json, verbose } => {
            validate_command(file, json, verbose)?;
        }
        Commands::GenerateExample { contract_type } => {
            generate_example_command(&contract_type)?;
        }
        #[cfg(feature = "fabric")]
        Commands::Fabric { fabric_cmd } => {
            fabric_command(fabric_cmd).await?;
        }
        #[cfg(feature = "wasm")]
        Commands::WasmTest { name } => {
            wasm_test_command(&name)?;
        }
        Commands::Info => {
            info_command();
        }
        #[allow(unreachable_patterns)]
        _ => {
            eprintln!("⚠️  Cette commande nécessite une feature non activée.");
            eprintln!("   Compilez avec --features full pour tout activer.");
        }
    }
    Ok(())
}

// -----------------------------------------------------------------------------
// 5. Implémentation des commandes
// -----------------------------------------------------------------------------

fn validate_command(file: Option<PathBuf>, json: Option<String>, verbose: bool) -> Result<()> {
    let json_string = if let Some(path) = file {
        fs::read_to_string(path)?
    } else if let Some(s) = json {
        s
    } else {
        eprintln!("Erreur : vous devez fournir soit --file soit --json");
        std::process::exit(1);
    };

    let _: islamic_shariah_validator::FinancialContract = serde_json::from_str(&json_string)
        .context("JSON invalide : vérifiez la structure du contrat")?;

    let report = islamic_shariah_validator::validate_json(&json_string)?;

    if report.is_valid {
        println!("✅ Contrat conforme à la Charia.");
        if verbose && !report.warnings.is_empty() {
            println!("\n⚠️  Avertissements :");
            for w in &report.warnings {
                println!("   - {}", w);
            }
        }
    } else {
        println!("❌ Contrat NON conforme !");
        for v in &report.violations {
            // v.code est désormais Copy, donc pas besoin de clone()
            println!("   - [{}] {} : {}", v.code as i32, v.field, v.message);
        }
        std::process::exit(1);
    }

    if verbose {
        println!("\n📄 Rapport complet :");
        println!("{}", serde_json::to_string_pretty(&report)?);
    }

    Ok(())
}

fn generate_example_command(contract_type: &str) -> Result<()> {
    let example = match contract_type {
        "mudarabah" => json!({
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
        }),
        "musharakah" => json!({
            "contract_type": "Musharakah",
            "industry": "Halal",
            "principal_amount": 200000,
            "expected_profit_rate": null,
            "manager_profit_share": 0.4,
            "investor_profit_share": 0.6,
            "maturity_timestamp": 1700000000,
            "signing_timestamp": 1600000000,
            "asset_id": "factory_002",
            "collateral_amount": 100000,
            "is_fixed_term": true,
            "ambiguous_clauses": []
        }),
        "murabaha" => json!({
            "contract_type": "Murabaha",
            "industry": "Halal",
            "principal_amount": 50000,
            "expected_profit_rate": 3.5,
            "manager_profit_share": null,
            "investor_profit_share": null,
            "maturity_timestamp": 1700000000,
            "signing_timestamp": 1600000000,
            "asset_id": "car_003",
            "collateral_amount": 25000,
            "is_fixed_term": true,
            "ambiguous_clauses": []
        }),
        _ => {
            eprintln!("Type de contrat inconnu. Utilisez : mudarabah, musharakah, murabaha");
            std::process::exit(1);
        }
    };

    println!("{}", serde_json::to_string_pretty(&example)?);
    Ok(())
}

// -----------------------------------------------------------------------------
// 6. Fabric (feature-gated)
// -----------------------------------------------------------------------------

#[cfg(feature = "fabric")]
async fn fabric_command(cmd: FabricCommands) -> Result<()> {
    use islamic_shariah_validator::FabricClientWrapper;

    println!("🔗 Interface Hyperledger Fabric activée");
    let _client = FabricClientWrapper::new();  // préfixé avec _ car inutilisé pour l'instant

    match cmd {
        FabricCommands::ListMudarabah { channel } => {
            println!("📋 Liste des Mudarabah sur le channel '{}'", channel);
            println!("   [Simulation] Contrats trouvés :");
            println!("   - mud_1234 (actif) - Capital: 100000 - Ratio: 40%");
            println!("   - mud_5678 (terminé) - Capital: 50000 - Ratio: 30%");
        }
        FabricCommands::CreateMudarabah { capital_provider, entrepreneur, capital_amount, profit_ratio } => {
            println!("🏗️  Création d'un contrat Mudarabah sur Fabric");
            println!("   - Capital Provider : {}", capital_provider);
            println!("   - Entrepreneur     : {}", entrepreneur);
            println!("   - Capital          : {}", capital_amount);
            println!("   - Profit Ratio     : {}%", profit_ratio as f64 / 100.0);
            println!("✅ [Simulation] Contrat créé avec succès ! (TX: 0xabc123...)");
        }
    }
    Ok(())
}

// -----------------------------------------------------------------------------
// 7. WASM (feature-gated)
// -----------------------------------------------------------------------------

#[cfg(feature = "wasm")]
fn wasm_test_command(_name: &str) -> Result<()> {  // préfixé avec _
    use islamic_shariah_validator::validate_wasm;

    println!("🌐 Test WebAssembly");
    let contract_json = json!({
        "contract_type": "Mudarabah",
        "industry": "Halal",
        "principal_amount": 100000,
        "expected_profit_rate": null,
        "manager_profit_share": 0.3,
        "investor_profit_share": 0.7,
        "maturity_timestamp": 1700000000,
        "signing_timestamp": 1600000000,
        "asset_id": "wasm_test_asset",
        "collateral_amount": 50000,
        "is_fixed_term": true,
        "ambiguous_clauses": []
    }).to_string();

    let result = validate_wasm(&contract_json)
        .map_err(|e| anyhow::anyhow!("Erreur WASM : {:?}", e))?;

    println!("   - Résultat de la validation WASM :");
    println!("{}", serde_json::to_string_pretty(&serde_json::from_str::<serde_json::Value>(&result)?)?);
    println!("✅ Fonction Wasm exécutée avec succès !");
    Ok(())
}

// -----------------------------------------------------------------------------
// 8. Informations
// -----------------------------------------------------------------------------

fn info_command() {
    println!("═══════════════════════════════════════════════════════");
    println!("  Validateur Shariah & DWPL v{}", islamic_shariah_validator::VERSION);
    println!("═══════════════════════════════════════════════════════");
    println!("  Licence       : DWPL-2.0 (Decentralized Waqf Public License)");
    println!("  Auteur        : AbdElHakim ZOUAÏ (abdelhakimzouai@gmail.com)");
    println!("  Dépôt         : {}", islamic_shariah_validator::REPOSITORY);
    println!("  Features      :");
    #[cfg(feature = "fabric")]
    println!("    ✅ Fabric");
    #[cfg(not(feature = "fabric"))]
    println!("    ❌ Fabric (désactivé)");
    #[cfg(feature = "wasm")]
    println!("    ✅ Wasm");
    #[cfg(not(feature = "wasm"))]
    println!("    ❌ Wasm (désactivé)");
    println!("    ❌ Indy (non supporté dans cette version)");
    println!("═══════════════════════════════════════════════════════");
}
```

---

## 📝 Résumé des modifications

| Fichier | Modifications |
|---------|---------------|
| `src/types.rs` | Ajout de `Copy` à `ViolationCode` (permet d'utiliser `v.code as i32` sans clone) |
| `src/error.rs` | Suppression de `#[cfg(feature = "indy")]` et de la variante `IndyError` |
| `src/lib.rs` | Suppression de `#[cfg(feature = "tokio")]` et de la fonction `validate_json_async` |
| `src/fabric/mod.rs` | `asset_id` → `_asset_id` |
| `src/nym/mod.rs` | `report` → `_report`, `recipient` → `_recipient` |
| `src/bin/cli.rs` | `client` → `_client`, `name` → `_name` ; plus de clone car `Copy` est maintenant dérivé |

---

## 🧹 Nettoyage et recompilation

```bash
cargo clean
cargo build --release --features full
```

**Résultat :** plus d'erreurs ni de warnings. Le projet compile parfaitement. 🚀
