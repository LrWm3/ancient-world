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


struct Params {dims:vec4<u32>,physical:vec4<f32>,limits:vec4<u32>}
@group(0) @binding(0) var<storage,read> world:array<Cell>;
@group(0) @binding(1) var<storage,read> scores:array<vec4<f32>>;
@group(0) @binding(2) var<storage,read_write> labels:array<atomic<u32>>;
@group(0) @binding(3) var<storage,read_write> records:array<vec4<u32>>;
@group(0) @binding(4) var<storage,read_write> counter:atomic<u32>;
@group(0) @binding(5) var<uniform> p:Params;
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


@compute @workgroup_size(64)
fn survey(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=g.x+g.y*65535u*64u;if i>=p.limits.y||world[i].tags.x!=2u{return;}
 var coast=false;var landing=false;
 for(var k=0u;k<4u;k++){let j=neighbor(i,k);if world[j].tags.x==1u {coast=true;if world[j].water.x>.25 && world[i].water.x<.25 {landing=true;}}}
 let coasts=p.dims.w==1u;
 if coasts {if !landing{return;}}else{if scores[i].x<=450.{return;}}
 var landmark=0u;if world[i].terrain.x>1000.{landmark=1u;}if world[i].water.w>1.{landmark=2u;}if coast{landmark=3u;}
 let at=atomicAdd(&counter,1u);records[at*2u]=vec4(i,atomicLoad(&labels[i]),landmark,0u);
 if coasts {records[at*2u+1u]=vec4(0u);}else{records[at*2u+1u]=bitcast<vec4<u32>>(scores[i]);}
}
