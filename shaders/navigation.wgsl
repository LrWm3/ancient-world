struct Cell {
 terrain: vec4<f32>, // elevation m, sediment m, soil m, crust age Myr
 climate: vec4<f32>, // temperature C, precipitation mm/year, vapor mm, wind m/s
 water: vec4<f32>, // water depth m, groundwater m, snow m, discharge m3/s
 life: vec4<f32>, // coverage, fertility, salinity g/kg, runoff m/year
 geology: vec4<f32>, // stress, crust km, resource probability, depth m
 hydro: vec4<f32>, // spill m, seasonal mean temperature, mean rain, annual water input m
 ids: vec4<u32>, // rock, soil, plant, biome
 routing: vec4<u32>, // downstream, rank, basin, plate
 tags: vec4<u32>, // region: 0 ocean 1 great lake 2 inner land 3 outer land; mineral, layer2, layer3
 budget: vec4<f32>, // precipitation m, evaporation m, eroded m, deposited m
 strata: vec4<f32>, // top, middle, basement thickness m; cumulative bedrock removed m
}

struct Params { dims:vec4<u32>, physical:vec4<f32>, limits:vec4<u32> }
@group(0) @binding(0) var<storage,read> world:array<Cell>;
@group(0) @binding(1) var<storage,read_write> distances:array<atomic<u32>>;
@group(0) @binding(2) var<storage,read_write> qa:array<u32>;
@group(0) @binding(3) var<storage,read_write> qb:array<u32>;
@group(0) @binding(4) var<storage,read_write> marks:array<atomic<u32>>;
// queue counts, phase, wave, indirect x/y/z, error, and terminal cost bound.
@group(0) @binding(5) var<storage,read_write> control:array<atomic<u32>>;
@group(0) @binding(6) var<storage,read_write> output:array<u32>;
@group(0) @binding(7) var<uniform> p:Params;
const NONE:u32=0xffffffffu;
fn direction(f:u32,u:f32,v:f32)->vec3<f32> {
 var d=vec3(u,v,-1.);
 switch f { case 0u:{d=vec3(1.,u,v);} case 1u:{d=vec3(-1.,u,v);} case 2u:{d=vec3(u,1.,v);} case 3u:{d=vec3(u,-1.,v);} case 4u:{d=vec3(u,v,1.);} default:{} }
 return normalize(d);
}
fn pos(i:u32)->vec3<f32> { let n=p.dims.x; return direction(i/(n*n),2.*(f32(i%n)+.5)/f32(n)-1.,2.*(f32((i/n)%n)+.5)/f32(n)-1.); }
fn index(d:vec3<f32>)->u32 {
 let a=abs(d); var f=0u; var uv=vec2(0.);
 if a.x>=a.y && a.x>=a.z { f=select(1u,0u,d.x>=0.);uv=d.yz/a.x; }
 else if a.y>=a.z { f=select(3u,2u,d.y>=0.);uv=d.xz/a.y; }
 else { f=select(5u,4u,d.z>=0.);uv=d.xy/a.z; }
 let xy=vec2<u32>(clamp(floor((uv+1.)*.5*f32(p.dims.x)),vec2(0.),vec2(f32(p.dims.x-1u))));
 return f*p.dims.x*p.dims.x+xy.y*p.dims.x+xy.x;
}
fn neighbor(i:u32,k:u32)->u32 {
 let n=p.dims.x; let offsets=array<vec2<i32>,4>(vec2(-1,0),vec2(1,0),vec2(0,-1),vec2(0,1));
 let xy=vec2<i32>(vec2<u32>(i%n,(i/n)%n))+offsets[k];
 if all(xy>=vec2(0)) && all(xy<vec2(i32(n))) {return (i/(n*n))*n*n+u32(xy.y)*n+u32(xy.x);}
 let uv=2.*(vec2<f32>(xy)+.5)/f32(n)-1.;
 return index(direction(i/(n*n),uv.x,uv.y));
}

fn linear(g:vec3<u32>)->u32 {return g.x+g.y*65535u*64u;}
fn lake(i:u32)->bool {return world[i].tags.x==1u&&world[i].water.x>.25;}
fn dry(i:u32, region:u32)->bool {return world[i].tags.x==region&&world[i].water.x<.25;}
fn allowed(i:u32)->bool {
 switch p.dims.w {case 0u:{return dry(i,2u);}case 1u:{return dry(i,2u)||lake(i);}case 2u:{return lake(i);}default:{return lake(i)||dry(i,3u);}}
}
fn terminal(i:u32)->bool {
 switch p.dims.w {case 0u,2u:{return i==p.dims.z;}case 1u:{return lake(i);}default:{return dry(i,3u);}}
}
fn edge(a:u32,b:u32)->u32 {
 // atan2 form is well-conditioned for adjacent cells at high resolution.
 let x=pos(a);let y=pos(b);let km=atan2(length(cross(x,y)),clamp(dot(x,y),-1.,1.))*p.physical.x;
 var friction=1.;
 if p.dims.w==0u || (p.dims.w==1u && !lake(b)) {
  friction+=abs(world[a].terrain.x-world[b].terrain.x)/500.+min(world[b].water.w/1000.,3.);
 }
 return u32(clamp(km*1000.*friction,1.,f32(p.limits.x+1u)));
}
@compute @workgroup_size(64)
fn initialize(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=linear(g);if i>=p.limits.y{return;}
 atomicStore(&distances[i],NONE);atomicStore(&marks[i],0u);
 if i==p.dims.y && allowed(i) {atomicStore(&distances[i],0u);qa[0]=i;}
 if i==0u {
  atomicStore(&control[0],select(0u,1u,allowed(p.dims.y)));
  for(var k=1u;k<8u;k++){atomicStore(&control[k],0u);}
  atomicStore(&control[8],NONE);
 }
}
@compute @workgroup_size(1)
fn prepare() {
 let phase=atomicLoad(&control[2]);let count=atomicLoad(&control[phase]);
 atomicStore(&control[1u-phase],0u);atomicAdd(&control[3],1u);
 let groups=(count+63u)/64u;
 atomicStore(&control[4],min(groups,65535u));
 atomicStore(&control[5],select(0u,(groups+65534u)/65535u,groups>0u));atomicStore(&control[6],1u);
}
@compute @workgroup_size(64)
fn relax(@builtin(global_invocation_id) g:vec3<u32>) {
 let slot=linear(g);let phase=atomicLoad(&control[2]);if slot>=atomicLoad(&control[phase]){return;}
 var i=0u;if phase==0u{i=qa[slot];}else{i=qb[slot];}
 let d=atomicLoad(&distances[i]);if d>=p.limits.x || d>=atomicLoad(&control[8]) || terminal(i){return;}
 for(var k=0u;k<4u;k++) {
  let j=neighbor(i,k);if !allowed(j){continue;}
  let cost=edge(i,j);if cost>p.limits.x-d{continue;}let candidate=d+cost;
  if candidate>atomicLoad(&control[8]){continue;}
  if atomicMin(&distances[j],candidate)>candidate {
   if terminal(j){atomicMin(&control[8],candidate);}
   let wave=atomicLoad(&control[3]);
   if atomicExchange(&marks[j],wave)!=wave {
    let at=atomicAdd(&control[1u-phase],1u);
    if at>=p.limits.y {atomicStore(&control[7],1u);continue;}
    if phase==0u {qb[at]=j;}else{qa[at]=j;}
   }
  }
 }
}
@compute @workgroup_size(1)
fn finish(){atomicStore(&control[2],1u-atomicLoad(&control[2]));}
@compute @workgroup_size(64)
fn choose(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=linear(g);if i>=p.limits.y || !terminal(i){return;}
 atomicMin(&control[0],atomicLoad(&distances[i]));
}
@compute @workgroup_size(64)
fn choose_id(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=linear(g);if i>=p.limits.y || !terminal(i){return;}
 if atomicLoad(&distances[i])==atomicLoad(&control[0]) {atomicMin(&control[1],i);}
}
@compute @workgroup_size(1)
fn reconstruct() {
 let total=atomicLoad(&control[0]);var i=atomicLoad(&control[1]);
 output[0]=0u;output[1]=total;output[2]=i;output[3]=0u;
 if total==NONE {return;}
 var path_cost=0u;
 for(var n=0u;n<p.limits.y;n++) {
  output[4u+n]=i;
  if i==p.dims.y {output[0]=n+1u;output[1]=path_cost;return;}
  let d=atomicLoad(&distances[i]);var best=NONE;var score=NONE;var chosen_cost=0u;
  for(var k=0u;k<4u;k++) {
   let j=neighbor(i,k);let prev=atomicLoad(&distances[j]);
   // Do not require transcendental edge calculations in different pipelines
   // to round to the same integer. A strictly descending distance prevents cycles.
   if prev<d && !terminal(j) && allowed(j) {
    let step=edge(j,i);let candidate=prev+step;
    if candidate<score || (candidate==score && j<best) {best=j;score=candidate;chosen_cost=step;}
   }
  }
  if best==NONE {output[3]=1u;return;}if chosen_cost>NONE-path_cost {output[3]=3u;return;}path_cost+=chosen_cost;i=best;
 }
 output[3]=2u;
}
// Reuse the distance workspace for component roots. Union by minimum root keeps
// old island identity conventions. Hook/compress run in separate global passes.
@compute @workgroup_size(64)
fn label_init(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=linear(g);if i<p.limits.y {atomicStore(&distances[i],select(NONE,i,world[i].tags.x==2u));}
}
fn root(i:u32)->u32 {
 var r=i;
 for(var k=0u;k<64u;k++){let parent=atomicLoad(&distances[r]);if parent==r{return r;}r=parent;}
 return r;
}
@compute @workgroup_size(64)
fn hook(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=linear(g);if i>=p.limits.y||world[i].tags.x!=2u{return;}
 for(var k=0u;k<4u;k++) {
  let j=neighbor(i,k);if world[j].tags.x!=2u{continue;}
  let a=root(i);let b=root(j);
  if a!=b && atomicMin(&distances[max(a,b)],min(a,b))>min(a,b){atomicAdd(&control[7],1u);}
 }
}
@compute @workgroup_size(64)
fn compress(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=linear(g);if i<p.limits.y && world[i].tags.x==2u {atomicMin(&distances[i],root(i));}
}
