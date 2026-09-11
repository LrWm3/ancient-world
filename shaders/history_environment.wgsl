// Bitwise gather of the 176-byte terrain Cell ABI. No environmental calculation
// happens here: selected cells must exactly match a full terrain readback.
struct Cell { words: array<vec4<u32>, 11> }
@group(0) @binding(0) var<storage, read> terrain: array<Cell>;
@group(0) @binding(1) var<storage, read> indices: array<u32>;
@group(0) @binding(2) var<storage, read_write> observations: array<Cell>;
@group(0) @binding(3) var<uniform> params: vec4<u32>;
@compute @workgroup_size(64)
fn gather(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x + gid.y * params.y;
    if i < params.x { observations[i] = terrain[indices[i]]; }
}
