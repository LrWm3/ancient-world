struct Eco {pools:array<vec4<f32>,41>}
struct Transfer {ids:vec4<u32>,flow:vec4<f32>}
@group(0) @binding(0) var<storage,read_write> river:array<vec4<f32>>;
@group(0) @binding(1) var<storage,read_write> ecology:array<Eco>;
@group(0) @binding(2) var<storage,read> transfers:array<Transfer>;
// Sparse town transfers serialize overlapping destinations; river evolution stays parallel.
@compute @workgroup_size(1)
fn commit() {
 for(var k=0u;k<arrayLength(&transfers);k++) {
  let t=transfers[k];let area=bitcast<f32>(t.ids.z);
  river[t.ids.x]+=t.flow;
  ecology[t.ids.y].pools[27]+=t.flow/area;
  ecology[t.ids.y].pools[25].w+=t.flow.w/area;
 }
}
