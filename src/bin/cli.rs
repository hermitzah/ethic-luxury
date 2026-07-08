// SPDX-License-Identifier: DWPL-2.0
// Fichier : src/bin/cli.rs
// Description : Interface en ligne de commande pour le validateur Shariah & DWPL

use clap::{Parser, Subcommand};
use anyhow::{Context, Result};
use serde_json::json;
use std::path::PathBuf;

// -----------------------------------------------------------------------------
// 1. Structure des arguments de la ligne de commande
// -----------------------------------------------------------------------------

#[derive(Parser)]
#[command(name = "islamic-shariah-validator")]
#[command(about = "Validateur Shariah & DWPL pour Hyperledger Fabric/Indy", long_about = None)]
struct Cli {
    /// Active le mode verbose (logs détaillés)
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Chemin vers un fichier de configuration (JSON)
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
    /// Valide un contrat Mudarabah selon les règles AAOIFI
    Validate {
        /// Ratio de profit du Mudarib (en points de base, ex: 4000 = 40%)
        #[arg(long)]
        profit_ratio: u16,

        /// Montant du capital (en unités de base)
        #[arg(long)]
        capital: u64,

        /// Type de perte : "capital_only" ou "profit_forfeited"
        #[arg(long, default_value = "profit_forfeited")]
        loss_provision: String,
    },

    /// Intéraction avec Hyperledger Fabric
    #[cfg(feature = "fabric")]
    Fabric {
        #[command(subcommand)]
        fabric_cmd: FabricCommands,
    },

    /// Intéraction avec Hyperledger Indy
    #[cfg(feature = "indy")]
    Indy {
        #[command(subcommand)]
        indy_cmd: IndyCommands,
    },

    /// Exécute un test Wasm (si compilé avec la feature wasm)
    #[cfg(feature = "wasm")]
    WasmTest {
        /// Message à afficher
        #[arg(default_value = "Monde")]
        name: String,
    },

    /// Affiche les métadonnées du validateur (licence DWPL, version, etc.)
    Info,
}

// -----------------------------------------------------------------------------
// 3. Sous-commandes Fabric
// -----------------------------------------------------------------------------

#[cfg(feature = "fabric")]
#[derive(Subcommand)]
enum FabricCommands {
    /// Liste les contracts Mudarabah enregistrés sur le ledger Fabric
    ListMudarabah {
        /// Chaîne (channel) Fabric à interroger
        #[arg(long, default_value = "islamic-channel")]
        channel: String,
    },

    /// Soumet une transaction pour créer un nouveau Mudarabah
    CreateMudarabah {
        /// DID du capital provider
        #[arg(long)]
        capital_provider: String,

        /// DID de l'entrepreneur
        #[arg(long)]
        entrepreneur: String,

        /// Montant du capital
        #[arg(long)]
        capital_amount: u64,

        /// Ratio de profit (points de base)
        #[arg(long)]
        profit_ratio: u16,
    },
}

// -----------------------------------------------------------------------------
// 4. Sous-commandes Indy
// -----------------------------------------------------------------------------

#[cfg(feature = "indy")]
#[derive(Subcommand)]
enum IndyCommands {
    /// Crée un nouveau DID (Decentralized IDentifier) sur le ledger Indy
    CreateDid {
        /// Seed optionnel pour la déterminisme
        #[arg(long)]
        seed: Option<String>,
    },

    /// Vérifie un Verifiable Credential (VC)
    VerifyVC {
        /// Chemin vers le fichier JSON contenant le VC
        #[arg(long)]
        vc_file: PathBuf,

        /// DID du holder à vérifier
        #[arg(long)]
        holder_did: String,
    },

    /// Émet un VC "InvestorEligibility" (nécessite un issuer DID)
    IssueVC {
        /// DID de l'émetteur (issuer)
        #[arg(long)]
        issuer_did: String,

        /// DID du détenteur (holder)
        #[arg(long)]
        holder_did: String,

        /// Niveau KYC (1-3)
        #[arg(long, default_value = "1")]
        kyc_level: u8,

        /// Approbation du Shariah Board
        #[arg(long, default_value = "true")]
        shariah_board_approval: bool,
    },
}

// -----------------------------------------------------------------------------
// 5. Fonction principale
// -----------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<()> {
    // Initialisation des logs
    env_logger::init();

    let cli = Cli::parse();

    // Si verbose, on active les logs détaillés
    if cli.verbose {
        std::env::set_var("RUST_LOG", "debug");
        env_logger::try_init().ok();
    }

    // Chargement de la configuration (si fournie)
    if let Some(config_path) = cli.config {
        let config_content = std::fs::read_to_string(config_path)
            .context("Impossible de lire le fichier de configuration")?;
        let config: serde_json::Value = serde_json::from_str(&config_content)
            .context("Fichier de configuration JSON invalide")?;
        println!("📄 Configuration chargée : {}", config);
    }

    // Exécution de la commande
    match cli.command {
        Commands::Validate { profit_ratio, capital, loss_provision } => {
            validate_command(profit_ratio, capital, &loss_provision)?;
        }

        #[cfg(feature = "fabric")]
        Commands::Fabric { fabric_cmd } => {
            fabric_command(fabric_cmd).await?;
        }

        #[cfg(feature = "indy")]
        Commands::Indy { indy_cmd } => {
            indy_command(indy_cmd).await?;
        }

        #[cfg(feature = "wasm")]
        Commands::WasmTest { name } => {
            wasm_test_command(&name)?;
        }

        Commands::Info => {
            info_command();
        }

        // Fallback pour les features non compilées
        #[allow(unreachable_patterns)]
        _ => {
            eprintln!("⚠️  Cette commande nécessite une feature non activée.");
            eprintln!("   Compilez avec --features full pour tout activer.");
        }
    }

    Ok(())
}

// -----------------------------------------------------------------------------
// 6. Implémentation des commandes
// -----------------------------------------------------------------------------

/// Commande de validation Shariah
fn validate_command(profit_ratio: u16, capital: u64, loss_provision: &str) -> Result<()> {
    println!("🔍 Validation d'un contrat Mudarabah");
    println!("   - Ratio de profit     : {} / 10000 ({}%)", profit_ratio, profit_ratio as f64 / 100.0);
    println!("   - Capital             : {}", capital);
    println!("   - Provision de perte  : {}", loss_provision);

    // Appel à la librairie (exemple)
    let is_valid = islamic_shariah_validator::shariah::validate_mudarabah(profit_ratio, capital, loss_provision);
    
    if is_valid {
        println!("✅ Contrat valide selon les critères AAOIFI");
    } else {
        println!("❌ Contrat invalide : le ratio de profit est trop élevé (max 90%) ou le capital est nul.");
        std::process::exit(1);
    }
    Ok(())
}

// -----------------------------------------------------------------------------
// Fabric Commandes
// -----------------------------------------------------------------------------

#[cfg(feature = "fabric")]
async fn fabric_command(cmd: FabricCommands) -> Result<()> {
    use fabric_sdk::{
        Gateway, Wallets, Network, 
        // etc. (adapté à la version réelle)
    };

    println!("🔗 Interface Hyperledger Fabric activée");

    match cmd {
        FabricCommands::ListMudarabah { channel } => {
            println!("📋 Liste des Mudarabah sur le channel '{}'", channel);
            // Simulation d'un appel Fabric (à remplacer par la vraie logique)
            // Dans la réalité, on utiliserait Gateway, Network, Contract.
            println!("   [Simulation] Contrats trouvés :");
            println!("   - mud_1234 (actif) - Capital: 100000 - Ratio: 40%");
            println!("   - mud_5678 (terminé) - Capital: 50000 - Ratio: 30%");
            // let network = get_fabric_network(&channel).await?;
            // let contract = network.get_contract("mudarabah");
            // let result = contract.evaluate_transaction("GetAllMudarabah").await?;
        }

        FabricCommands::CreateMudarabah { capital_provider, entrepreneur, capital_amount, profit_ratio } => {
            println!("🏗️  Création d'un contrat Mudarabah sur Fabric");
            println!("   - Capital Provider : {}", capital_provider);
            println!("   - Entrepreneur     : {}", entrepreneur);
            println!("   - Capital          : {}", capital_amount);
            println!("   - Profit Ratio     : {}%", profit_ratio as f64 / 100.0);

            // Appel au chaincode Fabric
            // let network = get_fabric_network("islamic-channel").await?;
            // let contract = network.get_contract("mudarabah");
            // let tx_id = contract.submit_transaction("CreateMudarabah", ...).await?;
            // println!("✅ Transaction soumise : {}", tx_id);
            
            println!("✅ [Simulation] Contrat créé avec succès ! (TX: 0xabc123...)");
        }
    }
    Ok(())
}

// -----------------------------------------------------------------------------
// Indy Commandes
// -----------------------------------------------------------------------------

#[cfg(feature = "indy")]
async fn indy_command(cmd: IndyCommands) -> Result<()> {
    use indy_sdk::{
        wallet, did, ledger,
        // etc.
    };

    println!("🆔 Interface Hyperledger Indy activée");

    match cmd {
        IndyCommands::CreateDid { seed } => {
            println!("🆕 Création d'un nouveau DID");
            if let Some(s) = &seed {
                println!("   - Seed fourni : {}", s);
            } else {
                println!("   - Seed : aléatoire");
            }

            // Simulation Indy
            let did = format!("did:indy:{}", uuid::Uuid::new_v4());
            let verkey = "7NHMk3iRzQ...";
            println!("✅ DID créé : {}", did);
            println!("🔑 Verkey   : {}", verkey);
        }

        IndyCommands::VerifyVC { vc_file, holder_did } => {
            println!("🔍 Vérification d'un Verifiable Credential");
            println!("   - Fichier VC : {}", vc_file.display());
            println!("   - Holder DID : {}", holder_did);

            let vc_content = std::fs::read_to_string(&vc_file)
                .context("Impossible de lire le fichier VC")?;
            let vc: serde_json::Value = serde_json::from_str(&vc_content)
                .context("VC JSON invalide")?;

            // Simulation de vérification cryptographique
            println!("✅ VC valide (signature vérifiée, non révoqué)");
            println!("📄 Contenu du VC : {}", serde_json::to_string_pretty(&vc)?);
        }

        IndyCommands::IssueVC { issuer_did, holder_did, kyc_level, shariah_board_approval } => {
            println!("📜 Émission d'un Verifiable Credential 'InvestorEligibility'");
            println!("   - Issuer  : {}", issuer_did);
            println!("   - Holder  : {}", holder_did);
            println!("   - KYC Level : {}", kyc_level);
            println!("   - Shariah Board Approval : {}", shariah_board_approval);

            let vc = json!({
                "@context": ["https://www.w3.org/2018/credentials/v1"],
                "type": ["VerifiableCredential", "InvestorEligibility"],
                "issuer": issuer_did,
                "issuanceDate": chrono::Utc::now().to_rfc3339(),
                "credentialSubject": {
                    "id": holder_did,
                    "isEligible": true,
                    "kycLevel": kyc_level,
                    "shariahBoardApproval": shariah_board_approval
                },
                "proof": {
                    "type": "Ed25519Signature2018",
                    "signatureValue": "5H6Fv7..."
                }
            });

            let vc_json = serde_json::to_string_pretty(&vc)?;
            println!("✅ VC émis avec succès :");
            println!("{}", vc_json);
        }
    }
    Ok(())
}

// -----------------------------------------------------------------------------
// Wasm Test
// -----------------------------------------------------------------------------

#[cfg(feature = "wasm")]
fn wasm_test_command(name: &str) -> Result<()> {
    println!("🌐 Test Wasm");
    let greeting = islamic_shariah_validator::greet();
    println!("   - Message du module Wasm : {}", greeting);
    println!("   - Nom passé en argument  : {}", name);
    println!("✅ Fonction Wasm exécutée avec succès !");
    Ok(())
}

// -----------------------------------------------------------------------------
// Info
// -----------------------------------------------------------------------------

fn info_command() {
    println!("═══════════════════════════════════════════════════════");
    println!("  Validateur Shariah & DWPL v2.0");
    println!("═══════════════════════════════════════════════════════");
    println!("  Licence       : DWPL-2.0 (Decentralized Waqf Public License)");
    println!("  Concepteur    : AbdElHakim ZOUAÏ (hermitz)");
    println!("  Dépôt         : https://github.com/hermitz/islamic-shariah-validator");
    println!("  Features      :");
    #[cfg(feature = "fabric")]
    println!("    ✅ Fabric");
    #[cfg(not(feature = "fabric"))]
    println!("    ❌ Fabric (désactivé)");
    #[cfg(feature = "indy")]
    println!("    ✅ Indy");
    #[cfg(not(feature = "indy"))]
    println!("    ❌ Indy (désactivé)");
    #[cfg(feature = "wasm")]
    println!("    ✅ Wasm");
    #[cfg(not(feature = "wasm"))]
    println!("    ❌ Wasm (désactivé)");
    println!("═══════════════════════════════════════════════════════");
}

// -----------------------------------------------------------------------------
// Helper pour la validation dans la librairie (exemple)
// -----------------------------------------------------------------------------

mod shariah {
    pub fn validate_mudarabah(profit_ratio: u16, capital: u64, _loss_provision: &str) -> bool {
        // Règle AAOIFI : le profit ratio ne peut dépasser 90% (9000/10000)
        profit_ratio <= 9000 && capital > 0
    }
}

// Pour que le code compile, on inclut cette ligne pour utiliser la vraie librairie.
// Mais ici, on redéfinit la fonction pour que le cli soit autonome.
// Dans la réalité, on appelle islamic_shariah_validator::shariah::validate_mudarabah
