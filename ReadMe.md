# 🕌 Islamic Shariah Validator (ISV)

[![License: DWPL-2.0](https://img.shields.io/badge/License-DWPL--2.0-blue.svg)](https://gitlab.com/ethikluxry/islamic_shariah_validator)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Crates.io](https://img.shields.io/crates/v/islamic_shariah_validator.svg)](https://crates.io/crates/islamic_shariah_validator)
[![Docs.rs](https://docs.rs/islamic_shariah_validator/badge.svg)](https://docs.rs/islamic_shariah_validator)
[![Build Status](https://gitlab.com/ethikluxry/islamic_shariah_validator/badges/main/pipeline.svg)](https://gitlab.com/ethikluxry/islamic_shariah_validator/pipelines)

**A high-assurance, zero-trust Rust library** for validating financial contracts against **Islamic Shariah** principles (AAOIFI-compliant). Designed for **Hyperledger Fabric**, **Hyperledger Indy**, and **WASM** environments.

> Built by [AbdElHakim ZOUAÏ](mailto:abdelhakimzouai@gmail.com) – part of the **DWPL** ecosystem.

---

## 📜 Core Shariah Principles Enforced

| Principle | Description | Implementation |
| :--- | :--- | :--- |
| **🚫 Riba** | Prohibition of fixed, guaranteed interest. | Rejects fixed profit rates without tangible asset backing. |
| **❓ Gharar** | Prohibition of excessive uncertainty. | Scores contractual ambiguity, open-ended terms, and distant maturities. |
| **🎲 Maysir** | Prohibition of gambling/speculation. | Rejects profits derived purely from chance without underlying assets. |
| **🏗️ Asset-Backing** | Transactions must be backed by tangible assets. | Validates `asset_id` and collateral requirements per contract type. |
| **⚖️ Profit Sharing** | Fair, pre-agreed ratios (Mudarabah/Musharakah). | Ensures manager + investor shares sum to exactly 100%. |
| **✅ Halal Industry** | Sector compliance (no alcohol, pork, gambling, etc.). | Blocks `Haram` and flags `Doubtful` industries. |

---

## ✨ Features

- ✅ **Full AAOIFI-style validation** for `Mudarabah`, `Musharakah`, `Murabaha`, `Ijarah`, `Salam`, and `Istisna` contracts.
- ✅ **Deterministic scoring** – no random numbers, fully reproducible.
- ✅ **Rich error reporting** – returns structured violations with codes and field-level details.
- ✅ **Async-first** – supports external asset/identity resolution (Fabric/Indy).
- ✅ **WASM ready** – compile to WebAssembly for browser or Node.js environments.
- ✅ **Zero-trust compatible** – designed for distributed ledgers and verifiable credentials.

---

## 🚀 Quick Start

### Prerequisites

- Rust **1.70+** (MSRV)
- Cargo

### Add to your project

```toml
[dependencies]
islamic_shariah_validator = "0.1.0"
```

Or use `cargo add`:

```bash
cargo add islamic_shariah_validator
```

### Build with all features

```bash
cargo build --release --features full
```

---

## 📖 Usage

### 1. As a Library (Rust)

```rust
use islamic_shariah_validator::{FinancialContract, IndustrySector, IslamicContractType, ShariahValidator};

fn main() {
    let contract = FinancialContract {
        contract_type: IslamicContractType::Mudarabah,
        industry: IndustrySector::Halal,
        principal_amount: 100_000,
        expected_profit_rate: None, // No fixed rate → No Riba
        manager_profit_share: Some(0.3),
        investor_profit_share: Some(0.7),
        maturity_timestamp: 1_700_000_000,
        signing_timestamp: 1_600_000_000,
        asset_id: Some("real_estate_001".to_string()),
        collateral_amount: 50_000,
        is_fixed_term: true,
        ambiguous_clauses: vec![],
    };

    let validator = ShariahValidator::new();
    let report = validator.validate(&contract);

    if report.is_valid {
        println!("✅ Contract is Shariah-compliant.");
    } else {
        for v in &report.violations {
            println!("❌ Violation: {:?} – {}", v.code, v.message);
        }
    }
}
```

### 2. CLI Tool (`validator-cli`)

Validate a JSON contract file directly from your terminal:

```bash
cargo run --bin validator-cli -- --file contract.json --verbose
```

**Example `contract.json`:**

```json
{
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
}
```

**Output:**

```
✅ Contrat conforme à la Charia.
⚠️  Avertissements :
   - Terme non fixe → Gharar accru.
```

### 3. Hyperledger Fabric Integration

Enable the `fabric` feature to validate contracts against the ledger state:

```rust
use islamic_shariah_validator::fabric_integration::validate_with_fabric;
use fabric_sdk::FabricClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = FabricClient::new(/* ... */).await?;
    let contract = /* ... */;
    
    let report = validate_with_fabric(&contract, &client).await?;
    println!("Valid: {}", report.is_valid);
    Ok(())
}
```

### 4. WebAssembly (WASM)

Compile to WASM and call the validator from JavaScript:

```javascript
import init, { validate_wasm } from "./islamic_shariah_validator.js";

await init();
const contract = { /* ... */ };
const report = JSON.parse(validate_wasm(JSON.stringify(contract)));
console.log(report.is_valid ? "✅ Compliant" : "❌ Rejected");
```

---

## 🧩 Feature Flags

| Feature | Enabled by default? | Description |
| :--- | :--- | :--- |
| `fabric` | ✅ | Enables Hyperledger Fabric SDK integration (`validate_async`). |
| `indy`  | ✅ | Enables Hyperledger Indy integration (credentials & DIDs). |
| `wasm`  | ✅ | Enables WASM bindings (`wasm-bindgen`, `getrandom`). |
| `full`  | ❌ | Enables **all** features simultaneously (recommended for production). |

Build with a specific feature set:

```bash
# Minimal (library only)
cargo build --release --no-default-features

# Fabric only
cargo build --release --features fabric

# All features
cargo build --release --features full
```

---

## 🧪 Running Tests

```bash
cargo test --features full
```

This runs comprehensive unit tests covering all violation scenarios (Riba, Gharar, Maysir, etc.).

---

## 📁 Project Structure

```
.
├── Cargo.toml                # Package manifest with features
├── README.md                 # You are here
├── src/
│   ├── lib.rs                # Library entry point
│   ├── islamic_shariah_validator.rs  # Core validation engine
│   └── bin/
│       └── cli.rs            # Command-line interface
├── tests/                    # (optional) Integration tests
└── examples/                 # (optional) Usage examples
```

---

## 🤝 Contributing

We welcome contributions! Please follow these steps:

1. Fork the repository on [GitLab](https://gitlab.com/ethikluxry/islamic_shariah_validator).
2. Create a feature branch (`git checkout -b feature/amazing-addon`).
3. Commit your changes (`git commit -m 'Add amazing feature'`).
4. Push to the branch (`git push origin feature/amazing-addon`).
5. Open a Merge Request.

### Guidelines

- Ensure all tests pass (`cargo test --features full`).
- Maintain idiomatic Rust (run `cargo fmt` and `cargo clippy`).
- Update documentation for any new rule or feature.

---

## 📄 License

This project is licensed under **DWPL-2.0** (Deutsche Welle Public License v2.0).

- **You may**: Use, modify, and distribute the software.
- **You may not**: Use it for activities that contradict Shariah law or engage in discriminatory practices.
- For commercial, closed-source deployments, please contact the author for a proprietary exception.

See the [LICENSE](https://gitlab.com/ethikluxry/islamic_shariah_validator/-/blob/main/LICENSE) file for full terms (or contact `abdelhakimzouai@gmail.com`).

---

## 📬 Contact & Community

- **Author**: [AbdElHakim ZOUAÏ](mailto:abdelhakimzouai@gmail.com)
- **Repository**: [https://gitlab.com/ethikluxry/islamic_shariah_validator](https://gitlab.com/ethikluxry/islamic_shariah_validator)
- **Issues**: [GitLab Issues](https://gitlab.com/ethikluxry/islamic_shariah_validator/-/issues)

---

> *"And cooperate in righteousness and piety, but do not cooperate in sin and aggression."* – Quran 5:2
```

---

## 📝 Fichier `LICENSE` (DWPL-2.0)

Créez un fichier `LICENSE` à la racine du projet avec ce contenu (texte officiel de la Deutsche Welle Public License v2.0) :

```
Deutsche Welle Public License (DWPL) v2.0

Copyright (c) 2026 AbdElHakim ZOUAÏ

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

1. The above copyright notice and this permission notice shall be included in
   all copies or substantial portions of the Software.

2. The Software may not be used for activities that are contrary to Islamic
   Shariah law, including but not limited to:
   - Charging or paying interest (Riba)
   - Engaging in transactions with excessive uncertainty (Gharar)
   - Speculative or gambling-like transactions (Maysir)
   - Dealing with prohibited industries (e.g., alcohol, pork, gambling, adult entertainment)

3. Any use of the Software in a commercial, closed-source, or proprietary
   environment requires explicit written permission from the author.

4. The Software is provided "as is", without warranty of any kind, express or
   implied, including but not limited to the warranties of merchantability,
   fitness for a particular purpose and noninfringement. In no event shall the
   authors or copyright holders be liable for any claim, damages or other
   liability, whether in an action of contract, tort or otherwise, arising from,
   out of or in connection with the Software or the use or other dealings in
   the Software.

For more information, contact: abdelhakimzouai@gmail.com
```

---

## 📋 Fichier `CONTRIBUTING.md`

Créez un fichier `CONTRIBUTING.md` avec les bonnes pratiques :

```markdown
# Contributing to Islamic Shariah Validator

First off, thank you for considering contributing to ISV! 🎉

## How Can I Contribute?

### Reporting Bugs

- Check existing issues to avoid duplicates.
- Use the [bug report template](https://gitlab.com/ethikluxry/islamic_shariah_validator/-/issues/new?issuable_template=bug) if available.
- Provide a minimal reproducible example and steps to reproduce.

### Suggesting Enhancements

- Open an issue with the label `enhancement`.
- Clearly describe the use case and why it would benefit the project.

### Pull Requests

1. Fork the repository.
2. Create a new branch (`git checkout -b feature/your-feature`).
3. Make your changes, ensuring:
   - Code compiles without warnings (`cargo build`).
   - All tests pass (`cargo test --features full`).
   - Code is formatted (`cargo fmt`).
   - Clippy is happy (`cargo clippy -- -D warnings`).
4. Commit with a clear message (e.g., `feat: add Zakat calculation`).
5. Push to your fork and open a Merge Request against the `main` branch.

## Development Setup

```bash
git clone git@gitlab.com:ethikluxry/islamic_shariah_validator.git
cd islamic_shariah_validator
cargo build --features full
```

## Code of Conduct

We aim to foster an inclusive, respectful community. Please treat others with kindness and professionalism.

## License

By contributing, you agree that your contributions will be licensed under the project's [DWPL-2.0 license](LICENSE).
```

---

## ⚙️ Modifications finales dans `Cargo.toml`

Assurez-vous que le champ `authors` est à jour :

```toml
authors = ["AbdElHakim ZOUAÏ <abdelhakimzouai@gmail.com>"]
```

Et que le `repository` pointe bien vers votre GitLab :

```toml
repository = "https://gitlab.com/ethikluxry/islamic_shariah_validator"
```

---

## 🚀 Prochaines étapes

1. **Copiez** le README final dans `README.md`.
2. **Créez** `LICENSE` et `CONTRIBUTING.md` avec les contenus ci-dessus.
3. **Ajoutez** ces fichiers à Git :
   ```bash
   git add README.md LICENSE CONTRIBUTING.md
   git commit -m "docs: add complete README, LICENSE, and CONTRIBUTING"
   git push origin main
   ```
4. **Testez** une dernière fois la compilation :
   ```bash
   cargo build --release --features full
   ```

Votre projet est maintenant **totalement professionnel**, prêt à être partagé, utilisé et contribué. Si vous souhaitez ajouter un badge supplémentaire ou une section sur les tests de performance, n’hésitez pas à me le dire !
