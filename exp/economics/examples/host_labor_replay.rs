//! Replay supplied main-engine History exports without changing the source world.
use ancient_world::civilization::History;
use economics_compute_smoke::{
    acceptance::skilled_workshop,
    compute::Backend,
    host::{
        Boundary, Settlement,
        ancient_world::{LaborBindings, LaborReplay},
    },
};

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let bindings_path = args
        .next()
        .ok_or("usage: host_labor_replay BINDINGS.json HISTORY.json [HISTORY.json ...]")?;
    let bindings: LaborBindings =
        serde_json::from_slice(&std::fs::read(&bindings_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let mut economy = None;
    for path in args {
        let history: History =
            serde_json::from_slice(&std::fs::read(&path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        if economy.is_none() {
            let run = skilled_workshop::run(true, Backend::Reference)?;
            let settlement = Settlement::attach(
                Boundary {
                    site: bindings.site,
                    month: history.month,
                },
                run.sim,
                run.audit,
            )?;
            economy = Some(LaborReplay::attach(bindings.clone(), &history, settlement)?);
        }
        let outcome = economy.as_mut().unwrap().advance(&history)?;
        println!(
            "{}",
            serde_json::to_string(&outcome).map_err(|e| e.to_string())?
        );
    }
    if economy.is_none() {
        return Err("supply at least one main-engine History export".into());
    }
    Ok(())
}
