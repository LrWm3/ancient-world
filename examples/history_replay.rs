//! Verify mature-world continuation across a checkpoint and different dispatch batches.
use ancient_world::gpu::{ContextGpu, Generator};
use anyhow::{ensure, Result};
use std::path::Path;
fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    ensure!(args.len() == 3, "usage: history_replay WORLD REPORT.json");
    let gpu = pollster::block_on(ContextGpu::headless())?;
    let mut uninterrupted = Generator::load(gpu.clone(), Path::new(&args[1]))?;
    let mut split = Generator::load(gpu.clone(), Path::new(&args[1]))?;
    uninterrupted.advance_history(24)?;
    for _ in 0..12 {
        split.advance_history(1)?;
    }
    let checkpoint =
        std::env::temp_dir().join(format!("mature-replay-{}.world", std::process::id()));
    split.save(&checkpoint)?;
    drop(split);
    let mut resumed = Generator::load(gpu, &checkpoint)?;
    std::fs::remove_file(checkpoint)?;
    for _ in 0..12 {
        resumed.advance_history(1)?;
    }
    let expected = serde_json::to_vec(&uninterrupted.civilizations)?;
    ensure!(
        expected == serde_json::to_vec(&resumed.civilizations)?,
        "historical continuation differs"
    );
    let a = uninterrupted.snapshot()?;
    let b = resumed.snapshot()?;
    ensure!(
        bytemuck::cast_slice::<_, u8>(&a) == bytemuck::cast_slice::<_, u8>(&b),
        "terrain differs"
    );
    let a = uninterrupted
        .ecology
        .snapshot(&uninterrupted.gpu, &uninterrupted.config)?;
    let b = resumed.ecology.snapshot(&resumed.gpu, &resumed.config)?;
    ensure!(
        bytemuck::cast_slice::<_, u8>(&a) == bytemuck::cast_slice::<_, u8>(&b),
        "ecological continuation differs"
    );
    let fingerprint = expected.iter().fold(14695981039346656037u64, |hash, b| {
        (hash ^ *b as u64).wrapping_mul(1099511628211)
    });
    let result = serde_json::json!({"passed":true,"input":args[1],"months":24,"checkpoint_after_months":12,"history_fingerprint":format!("{fingerprint:016x}"),"terrain_bitwise_equal":true,"ecology_bitwise_equal":true,"culture":uninterrupted.civilizations.as_ref().unwrap().cultural_summary()});
    std::fs::write(&args[2], serde_json::to_vec_pretty(&result)?)?;
    println!("{}", result);
    Ok(())
}
