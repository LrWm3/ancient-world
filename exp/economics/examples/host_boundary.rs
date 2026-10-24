//! Headless embedding harness using supplied nine-agent endowments. This is not
//! yet an Ancient World settlement import or an ecology/transport adapter.
use economics_compute_smoke::{
    acceptance::{SMITH, skilled_workshop},
    compute::Backend,
    host::{Boundary, Opening, Settlement},
};

const DEFAULT_MONTHS: u32 = 12;
const HARNESS_SITE: u32 = 1;
const FIRST_HOST_MONTH: u32 = 101;

fn main() -> Result<(), String> {
    let mut months = DEFAULT_MONTHS;
    let mut unavailable = None;
    let mut backend = Backend::Reference;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--cpu" => backend = Backend::CubeCpu,
            "--months" | "--unavailable-local-month" => {
                let value = args
                    .next()
                    .ok_or("missing month argument")?
                    .parse::<u32>()
                    .map_err(|e| e.to_string())?;
                if value == 0 {
                    return Err("month arguments must be positive".into());
                }
                if arg == "--months" {
                    months = value;
                } else {
                    unavailable = Some(value);
                }
            }
            _ => return Err(format!("unknown argument {arg}")),
        }
    }
    if unavailable.is_some_and(|m| m > months) {
        return Err("worker interruption lies outside the requested run".into());
    }
    let run = skilled_workshop::run(true, backend)?;
    let mut settlement = Settlement::attach(
        Boundary {
            site: HARNESS_SITE,
            month: FIRST_HOST_MONTH,
        },
        run.sim,
        run.audit,
    )?;
    for local_month in 1..=months {
        let mut capacities = settlement.capacity_ceilings();
        if unavailable == Some(local_month) {
            capacities
                .iter_mut()
                .find(|c| c.agent == SMITH)
                .ok_or("missing workshop worker")?
                .quantity = 0;
        }
        let outcome = settlement.advance_month(Opening {
            boundary: settlement.boundary(),
            capacities,
        })?;
        println!(
            "{}",
            serde_json::to_string(&outcome).map_err(|e| e.to_string())?
        );
    }
    Ok(())
}
