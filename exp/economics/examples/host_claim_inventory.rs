//! Inspect source claims without importing or advancing either economy.
use ancient_world::civilization::History;
use economics_compute_smoke::host::ancient_world::stocks::StockBindings;

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: host_claim_inventory STOCK_BINDINGS.json HISTORY.json".into());
    }
    let bindings: StockBindings =
        serde_json::from_slice(&std::fs::read(&args[0]).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let history: History =
        serde_json::from_slice(&std::fs::read(&args[1]).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let inventory = bindings.inspect_claims(&history)?;
    let blocked: Vec<_> = inventory
        .loans
        .iter()
        .filter(|l| l.blocks_liquid_import())
        .map(|l| l.loan)
        .collect();
    // Passing this scoped check is not full stock/catalog/world admission.
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "credit_check_passed": blocked.is_empty(),
            "blocking_loans": blocked,
            "inventory": inventory,
        }))
        .map_err(|e| e.to_string())?
    );
    Ok(())
}
