//! Host grouping and a CubeCL segmented gather; no contended balance writes.
use cubecl::{cpu::CpuRuntime, prelude::*};

const GATHER_UNITS: u32 = 32;

#[derive(Clone, Copy, Debug)]
pub enum Backend {
    Reference,
    CubeCpu,
}

#[cube(launch)]
fn gather(
    opening: &Array<i32>,
    offsets: &Array<u32>,
    deltas: &Array<i32>,
    output: &mut Array<i32>,
) {
    if ABSOLUTE_POS < opening.len() {
        let mut value = opening[ABSOLUTE_POS];
        for j in offsets[ABSOLUTE_POS]..offsets[ABSOLUTE_POS + 1] {
            value += deltas[j as usize];
        }
        output[ABSOLUTE_POS] = value;
    }
}

/// Inputs are private to settlement, which checks all prefix sums and offsets.
pub(crate) fn apply(
    backend: Backend,
    opening: &[i32],
    offsets: &[u32],
    deltas: &[i32],
) -> Result<Vec<i32>, String> {
    if opening.is_empty() {
        return Ok(Vec::new());
    }
    match backend {
        Backend::Reference => Ok(opening
            .iter()
            .enumerate()
            .map(|(i, start)| {
                deltas[offsets[i] as usize..offsets[i + 1] as usize]
                    .iter()
                    .fold(*start, |a, b| a + b)
            })
            .collect()),
        Backend::CubeCpu => launch::<CpuRuntime>(&Default::default(), opening, offsets, deltas),
    }
}

fn launch<R: Runtime>(
    device: &R::Device,
    opening: &[i32],
    offsets: &[u32],
    deltas: &[i32],
) -> Result<Vec<i32>, String> {
    let client = R::client(device);
    let a = client.create_from_slice(i32::as_bytes(opening));
    let b = client.create_from_slice(u32::as_bytes(offsets));
    // Empty segments are legal, but allocate a nonempty backing buffer.
    let values = if deltas.is_empty() { &[0][..] } else { deltas };
    let c = client.create_from_slice(i32::as_bytes(values));
    let out = client.empty(core::mem::size_of_val(opening));
    let count = u32::try_from(opening.len()).map_err(|_| "too many accounts")?;
    // SAFETY: Each argument describes its exact allocation. Checked CSR offsets
    // index only deltas; the tail guard protects excess units and each unit owns
    // one output. Settlement checked every integer prefix before launching.
    unsafe {
        gather::launch::<R>(
            &client,
            CubeCount::Static(count.div_ceil(GATHER_UNITS), 1, 1),
            CubeDim::new_1d(GATHER_UNITS),
            ArrayArg::from_raw_parts(a, opening.len()),
            ArrayArg::from_raw_parts(b, offsets.len()),
            ArrayArg::from_raw_parts(c, values.len()),
            ArrayArg::from_raw_parts(out.clone(), opening.len()),
        );
    }
    let bytes = client
        .read_one(out)
        .map_err(|e| format!("CubeCL readback: {e:?}"))?;
    Ok(i32::from_bytes(&bytes).to_vec())
}
