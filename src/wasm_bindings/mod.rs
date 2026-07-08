//! Bindings WebAssembly (feature `wasm`).

use wasm_bindgen::prelude::*;
use serde_json;

/// Fonction exportée pour WASM : valide un contrat JSON et retourne le rapport.
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn validate_wasm(json: &str) -> Result<String, JsValue> {
    let contract: crate::FinancialContract = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let validator = crate::ShariahValidator::new();
    let report = validator.validate(&contract);
    serde_json::to_string(&report)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
