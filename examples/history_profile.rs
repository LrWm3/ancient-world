//! Reproducible continuation benchmark; the history fingerprint detects semantic drift.
use ancient_world::gpu::{ContextGpu, Generator};
use anyhow::Result;
use std::time::Instant;
fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    anyhow::ensure!(
        args.len() == 4 || (args.len() == 5 && args[4] == "--refresh-economy"),
        "usage: history_profile WORLD MONTHS OUTPUT.json [--refresh-economy]"
    );
    let gpu = pollster::block_on(ContextGpu::headless())?;
    let mut g = Generator::load(gpu, std::path::Path::new(&args[1]))?;
    let months: u32 = args[2].parse()?;
    if args.len() == 5 {
        g.configure_economy(ancient_world::economy::EconomyCatalog::bundled()?)?;
    }
    g.progress.stage_ms.clear();
    let start = Instant::now();
    let mut samples = vec![];
    for i in 0..months {
        g.advance_history(1)?;
        if (i + 1) % 120 == 0 {
            let h = g.civilizations.as_ref().unwrap();
            let population = h
                .sites
                .iter()
                .map(|s| s.stocks.stock[0] as f64)
                .sum::<f64>();
            samples.push(serde_json::json!({"month":h.month,"population":population,"active_sites":h.sites.iter().filter(|s|!s.abandoned).count()}));
            eprintln!("year {}: {:.0} people", h.month / 12, population);
        }
    }
    let elapsed = start.elapsed().as_secs_f64();
    let h = g.civilizations.as_ref().unwrap();
    let bytes = serde_json::to_vec(h)?;
    let fingerprint = bytes.iter().fold(14695981039346656037u64, |hash, b| {
        (hash ^ *b as u64).wrapping_mul(1099511628211)
    });
    let rss = std::fs::read_to_string("/proc/self/status")
        .unwrap_or_default()
        .lines()
        .find(|l| l.starts_with("VmHWM:"))
        .unwrap_or("unavailable")
        .to_owned();
    let report = serde_json::json!({"refreshed_economy":args.len()==5,"samples":samples,"population":h.sites.iter().map(|s|s.stocks.stock[0] as f64).sum::<f64>(),"input":args[1],"months":months,"seconds":elapsed,"history_fingerprint":format!("{fingerprint:016x}"),"history_bytes":bytes.len(),"people":h.people.len(),"events":h.events.len(),"culture":h.cultural_summary(),"peak_host_memory":rss,"timings_ms":g.progress.stage_ms,"residuals":h.economy_residuals()});
    std::fs::write(&args[3], serde_json::to_vec_pretty(&report)?)?;
    if args.len() == 5 {
        g.save(std::path::Path::new(&args[3]).with_extension("world"))?;
    }
    println!("{}", serde_json::to_string(&report)?);
    Ok(())
}
