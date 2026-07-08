//! Benchmarks complets pour le validateur Shariah islamique.
//!
//! Ce fichier mesure les performances du validateur dans diverses conditions :
//! - Validation de contrats valides / invalides
//! - Chaque type de violation (Riba, Gharar, Maysir, etc.)
//! - Sérialisation / désérialisation JSON
//! - Impact de la taille des données (nombre de clauses ambiguës)
//! - Validation asynchrone (si feature `fabric` activée)
//!
//! Exécution : `cargo bench --features full`

use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkId, Criterion,
    Throughput,
};
use islamic_shariah_validator::{
    FinancialContract, IndustrySector, IslamicContractType, ShariahValidator,
    ViolationCode,
};

// ============================================================================
// 1. Générateurs de contrats pour les benchmarks
// ============================================================================

/// Contrat valide de type Mudarabah.
fn valid_mudarabah() -> FinancialContract {
    FinancialContract {
        contract_type: IslamicContractType::Mudarabah,
        industry: IndustrySector::Halal,
        principal_amount: 1_000_000,
        expected_profit_rate: None,
        manager_profit_share: Some(0.3),
        investor_profit_share: Some(0.7),
        maturity_timestamp: 1_700_000_000,
        signing_timestamp: 1_600_000_000,
        asset_id: Some("asset_001".to_string()),
        collateral_amount: 500_000,
        is_fixed_term: true,
        ambiguous_clauses: vec![],
    }
}

/// Contrat valide de type Murabaha.
fn valid_murabaha() -> FinancialContract {
    let mut c = valid_mudarabah();
    c.contract_type = IslamicContractType::Murabaha;
    c.expected_profit_rate = Some(3.5);
    c
}

/// Contrat valide de type Musharakah.
fn valid_musharakah() -> FinancialContract {
    let mut c = valid_mudarabah();
    c.contract_type = IslamicContractType::Musharakah;
    c
}

/// Contrat avec violation Riba (taux fixe sans actif).
fn riba_contract() -> FinancialContract {
    let mut c = valid_mudarabah();
    c.expected_profit_rate = Some(5.0);
    c.asset_id = None;
    c
}

/// Contrat avec Gharar excessif (clauses ambiguës + terme non fixe).
fn gharar_contract(num_ambiguous: usize) -> FinancialContract {
    let mut c = valid_mudarabah();
    c.ambiguous_clauses = (0..num_ambiguous)
        .map(|i| format!("clause ambiguë {}", i))
        .collect();
    c.is_fixed_term = false;
    c.asset_id = None;
    c
}

/// Contrat avec Maysir (profit attendu sans actif).
fn maysir_contract() -> FinancialContract {
    let mut c = valid_mudarabah();
    c.asset_id = None;
    c.expected_profit_rate = Some(10.0);
    c
}

/// Contrat avec parts de profit invalides (somme > 1).
fn invalid_profit_ratio_contract() -> FinancialContract {
    let mut c = valid_mudarabah();
    c.manager_profit_share = Some(0.8);
    c.investor_profit_share = Some(0.8);
    c
}

/// Contrat avec dates invalides (échéance avant signature).
fn invalid_dates_contract() -> FinancialContract {
    let mut c = valid_mudarabah();
    c.signing_timestamp = 1_700_000_000;
    c.maturity_timestamp = 1_600_000_000;
    c
}

/// Contrat sans adossement à un actif (Asset-Backing manquant).
fn no_asset_backing_contract() -> FinancialContract {
    let mut c = valid_mudarabah();
    c.asset_id = None;
    c.collateral_amount = 0;
    c
}

/// Contrat avec secteur Haram.
fn haram_industry_contract() -> FinancialContract {
    let mut c = valid_mudarabah();
    c.industry = IndustrySector::Haram;
    c
}

/// Contrat avec clauses vagues (mots interdits).
fn vague_clauses_contract() -> FinancialContract {
    let mut c = valid_mudarabah();
    c.ambiguous_clauses = vec![
        "livraison environ".to_string(),
        "prix peut-être".to_string(),
        "si possible".to_string(),
    ];
    c
}

// ============================================================================
// 2. Benchmarks de validation
// ============================================================================

fn bench_validation(c: &mut Criterion) {
    let validator = ShariahValidator::new();
    let mut group = c.benchmark_group("validation");

    // Cas valides
    for (name, contract) in [
        ("Mudarabah", valid_mudarabah()),
        ("Murabaha", valid_murabaha()),
        ("Musharakah", valid_musharakah()),
    ] {
        group.bench_with_input(
            BenchmarkId::new("valid", name),
            &contract,
            |b, c| b.iter(|| black_box(validator.validate(black_box(c)))),
        );
    }

    // Cas invalides par violation
    for (name, contract) in [
        ("Riba", riba_contract()),
        ("Maysir", maysir_contract()),
        ("InvalidProfitRatio", invalid_profit_ratio_contract()),
        ("InvalidDates", invalid_dates_contract()),
        ("NoAssetBacking", no_asset_backing_contract()),
        ("HaramIndustry", haram_industry_contract()),
        ("VagueClauses", vague_clauses_contract()),
    ] {
        group.bench_with_input(
            BenchmarkId::new("invalid", name),
            &contract,
            |b, c| b.iter(|| black_box(validator.validate(black_box(c)))),
        );
    }

    // Cas Gharar avec nombre variable de clauses ambiguës
    for n in [0, 1, 5, 10, 20] {
        let contract = gharar_contract(n);
        group.bench_with_input(
            BenchmarkId::new("gharar", format!("{}_clauses", n)),
            &contract,
            |b, c| b.iter(|| black_box(validator.validate(black_box(c)))),
        );
    }

    group.finish();
}

// ============================================================================
// 3. Benchmarks des règles individuelles (pour analyse fine)
// ============================================================================

fn bench_individual_rules(c: &mut Criterion) {
    let validator = ShariahValidator::new();
    let mut group = c.benchmark_group("individual_rules");

    // On isole chaque règle en appelant les méthodes privées via une fonction wrappers
    // (on utilise une astuce : on crée un validateur et on appelle les méthodes directement)
    // Pour cela, on va définir des fonctions internes dans le benchmark.

    let contract = valid_mudarabah();

    // Benchmark de la validation de l'industrie
    group.bench_function("validate_industry", |b| {
        b.iter(|| {
            let mut violations = vec![];
            validator.validate_industry(black_box(&contract), &mut violations);
            black_box(violations)
        })
    });

    // Benchmark de la validation Riba
    group.bench_function("validate_riba", |b| {
        b.iter(|| {
            let mut violations = vec![];
            validator.validate_riba(black_box(&contract), &mut violations);
            black_box(violations)
        })
    });

    // Benchmark de la validation Gharar
    group.bench_function("validate_gharar", |b| {
        b.iter(|| {
            let mut violations = vec![];
            let mut warnings = vec![];
            validator.validate_gharar(black_box(&contract), &mut violations, &mut warnings);
            black_box((violations, warnings))
        })
    });

    // Benchmark de la validation Maysir
    group.bench_function("validate_maysir", |b| {
        b.iter(|| {
            let mut violations = vec![];
            validator.validate_maysir(black_box(&contract), &mut violations);
            black_box(violations)
        })
    });

    // Benchmark de la validation Asset-Backing
    group.bench_function("validate_asset_backing", |b| {
        b.iter(|| {
            let mut violations = vec![];
            validator.validate_asset_backing(black_box(&contract), &mut violations);
            black_box(violations)
        })
    });

    // Benchmark de la validation des parts de profit
    group.bench_function("validate_profit_sharing", |b| {
        b.iter(|| {
            let mut violations = vec![];
            validator.validate_profit_sharing_ratios(black_box(&contract), &mut violations);
            black_box(violations)
        })
    });

    // Benchmark de la validation de clarté
    group.bench_function("validate_clarity", |b| {
        b.iter(|| {
            let mut violations = vec![];
            validator.validate_clarity(black_box(&contract), &mut violations);
            black_box(violations)
        })
    });

    group.finish();
}

// ============================================================================
// 4. Benchmarks de sérialisation / désérialisation
// ============================================================================

fn bench_serialization(c: &mut Criterion) {
    let contract = valid_mudarabah();
    let json = serde_json::to_string(&contract).unwrap();

    let mut group = c.benchmark_group("serialization");

    group.bench_function("serialize_to_json", |b| {
        b.iter(|| {
            let s = serde_json::to_string(black_box(&contract)).unwrap();
            black_box(s)
        })
    });

    group.bench_function("deserialize_from_json", |b| {
        b.iter(|| {
            let c: FinancialContract =
                serde_json::from_str(black_box(&json)).unwrap();
            black_box(c)
        })
    });

    group.finish();
}

// ============================================================================
// 5. Benchmark de validation asynchrone (si feature fabric)
// ============================================================================

#[cfg(feature = "fabric")]
fn bench_async_validation(c: &mut Criterion) {
    use std::future::Future;
    use tokio::runtime::Runtime;

    let validator = ShariahValidator::new();
    let contract = valid_mudarabah();

    // Simulateur de résolveur d'actif (toujours vrai, temps constant)
    let resolver = |_asset_id: &str| async { Ok::<_, islamic_shariah_validator::ShariahError>(true) };

    let rt = Runtime::new().unwrap();

    c.bench_function("async_validate_with_asset_check", |b| {
        b.to_async(rt.clone()).iter(|| async {
            let report = validator
                .validate_async(&contract, |id| resolver(id))
                .await
                .unwrap();
            black_box(report)
        })
    });
}

// ============================================================================
// 6. Benchmark de throughput (validation en lot)
// ============================================================================

fn bench_throughput(c: &mut Criterion) {
    let validator = ShariahValidator::new();
    let contracts: Vec<_> = (0..100)
        .map(|_| valid_mudarabah())
        .collect();

    let mut group = c.benchmark_group("throughput");
    group.throughput(Throughput::Elements(contracts.len() as u64));

    group.bench_function("validate_100_contracts", |b| {
        b.iter(|| {
            for contract in &contracts {
                black_box(validator.validate(black_box(contract)));
            }
        })
    });

    group.finish();
}

// ============================================================================
// 7. Enregistrement des groupes
// ============================================================================

criterion_group!(
    benches,
    bench_validation,
    bench_individual_rules,
    bench_serialization,
    #[cfg(feature = "fabric")]
    bench_async_validation,
    bench_throughput,
);
criterion_main!(benches);
