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


@group(0) @binding(0) var<storage,read> world:array<Cell>;
@group(0) @binding(1) var<storage,read> cells:array<u32>;
@group(0) @binding(2) var<storage,read> spans:array<vec4<u32>>;
@group(0) @binding(3) var<storage,read_write> results:array<u32>;
@compute @workgroup_size(NAVIGATION_WORKGROUP_SIZE)
fn inspect(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=g.x;if i>=arrayLength(&spans){return;}let s=spans[i];var exposed=false;
 for(var k=0u;k<s.y;k++) {
  let c=world[cells[s.x+k]];
  if s.z==2u {exposed=exposed||c.water.x<=MIN_NAVIGABLE_WATER_DEPTH_M;}
  else {let corridor=c.routing.x!=0xffffffffu&&c.water.w>RIVER_CORRIDOR_MIN_DISCHARGE_M3_S&&c.hydro.x-c.terrain.x<RIVER_CORRIDOR_SPILL_TOLERANCE_M;exposed=exposed||c.water.x/select(1.,RIVER_CORRIDOR_AREA_FRACTION,corridor)>=FLOOD_EXPOSURE_DEPTH_M;}
 }
 if s.z==1u {exposed=exposed||world[s.w].water.x<=MIN_NAVIGABLE_WATER_DEPTH_M;}
 results[i]=select(0u,1u,exposed);
}
