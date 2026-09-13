// Settlement survey and legacy climate parameters. Managed crops apply their
// catalog-specific climate after recovering this generic survey potential.
const SURVEY_COLD_OFFSET_C:f32=5.;
const SURVEY_COLD_RAMP_C:f32=20.;
const SURVEY_HEAT_CEILING_C:f32=45.;
const SURVEY_HEAT_RAMP_C:f32=15.;
const SURVEY_RAIN_SATURATION_MM_YEAR:f32=800.;
const SURVEY_RAIN_CEILING_MM_YEAR:f32=5000.;
const SURVEY_EXCESS_RAIN_RAMP_MM_YEAR:f32=2000.;
const SURVEY_MIN_SOIL_FACTOR:f32=.1;
const SURVEY_MAX_FLOOD_DEPTH_M:f32=.25;
const SURVEY_MAX_ELEVATION_M:f32=3000.;
const SURVEY_POTENTIAL_KG_HA_YEAR:f32=2200.;
const SURVEY_GROUNDWATER_BONUS_CAP_M:f32=.5;
const SURVEY_ELEVATION_PENALTY_START_M:f32=800.;
const SURVEY_ELEVATION_PENALTY_PER_M:f32=.001;
const SURVEY_CLIMATE_DIVISOR_FLOOR:f32=.1;
const LEGACY_WEATHER_MIN_FACTOR:f32=.7;
const LEGACY_WEATHER_FACTOR_SPAN:f32=.6;
const LIVE_HARVEST_WEATHER_CAP:f32=1.25;
const LEGACY_CULTIVATED_HA_PER_PERSON:f32=.45;
const MANAGED_CULTIVATED_HA_PER_PERSON:f32=.465;
const LEGACY_HARVEST_SEASON_AMPLITUDE:f32=.6;
const LEGACY_HARVEST_PHASE_MONTHS:f32=2.;
const LEGACY_HARVEST_RADIANS_PER_MONTH:f32=.5235988;

const LEGACY_CONTAINER_MAX_SPOILAGE_REDUCTION:f32=.5;
const LEGACY_CONTAINER_KG_PER_PERSON:f32=2.;
const LEGACY_CONTAINER_POPULATION_FLOOR:f32=1.;
const LEGACY_GRANARY_BASE_MONTHS:f32=12.;
const LEGACY_GRANARY_EXTENSION_MONTHS:f32=24.;
const LEGACY_MONTHLY_FOOD_SPOILAGE:f32=.01;
const LEGACY_RATION_KG_PER_PERSON_MONTH:f32=18.;
const LEGACY_NEED_DIVISOR_FLOOR_KG:f32=.01;
const LEGACY_FOOD_CNP_FRACTION:vec3<f32>=vec3(.45,.02,.003);
const LEGACY_MONTHLY_BIRTH_RATE:f32=.0025;
const LEGACY_MONTHLY_DEATH_RATE:f32=.0012;
const LEGACY_MONTHLY_STARVATION_DEATH_RATE:f32=.04;

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

struct Site { stock:vec4<f32>, habitat:vec4<f32>, ledger:vec4<f32>, people:vec4<f32> }
struct Params { dims:vec4<u32>, options:vec4<u32>, weather:vec4<u32> }
@group(0) @binding(0) var<storage,read> world:array<Cell>;
@group(0) @binding(1) var<storage,read> src:array<Site>;
@group(0) @binding(2) var<storage,read_write> dst:array<Site>;
@group(0) @binding(3) var<uniform> p:Params;
@group(0) @binding(4) var<storage,read_write> prospects:array<vec4<f32>>;
@compute @workgroup_size(HISTORY_WORKGROUP_SIZE)
fn survey(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=g.x+g.y*65535u*64u;if i>=p.dims.x{return;}let c=world[i];
 let warmth=clamp((c.hydro.y+SURVEY_COLD_OFFSET_C)/SURVEY_COLD_RAMP_C,0.,1.)*clamp((SURVEY_HEAT_CEILING_C-c.hydro.y)/SURVEY_HEAT_RAMP_C,0.,1.);
 let moisture=clamp(c.hydro.z/SURVEY_RAIN_SATURATION_MM_YEAR,0.,1.)*clamp((SURVEY_RAIN_CEILING_MM_YEAR-c.hydro.z)/SURVEY_EXCESS_RAIN_RAMP_MM_YEAR,0.,1.);
 let soil=clamp(c.life.y,SURVEY_MIN_SOIL_FACTOR,1.);let dry=c.tags.x==2u&&flood_depth(c)<SURVEY_MAX_FLOOD_DEPTH_M&&c.terrain.x<SURVEY_MAX_ELEVATION_M;
 let harvest=select(0.,SURVEY_POTENTIAL_KG_HA_YEAR*warmth*moisture*soil,dry);
 // Reserve a bounded, non-overlapping farm footprint within this world cell.
 let score=harvest*(1.+min(c.water.y,SURVEY_GROUNDWATER_BONUS_CAP_M)) /(1.+max(0.,c.terrain.x-SURVEY_ELEVATION_PENALTY_START_M)*SURVEY_ELEVATION_PENALTY_PER_M);
 prospects[i]=vec4(harvest,score,c.hydro.y,f32(c.tags.x));
}
fn flood_depth(c:Cell)->f32 {return c.water.x/select(1.,RIVER_CORRIDOR_AREA_FRACTION,c.routing.x!=0xffffffffu&&c.water.w>RIVER_CORRIDOR_MIN_DISCHARGE_M3_S&&c.hydro.x-c.terrain.x<RIVER_CORRIDOR_SPILL_TOLERANCE_M);}
fn hash(x:u32)->u32 {var v=x;v=(v^(v>>16u))*0x7feb352du;v=(v^(v>>15u))*0x846ca68bu;return v^(v>>16u);}
// Spatial bins use normalized 3D position, so weather regions cross cube-face seams.
fn regional_weather(cell:u32)->f32 {
 let n=p.weather.w;let face=cell/(n*n);
 let u=2.*(f32(cell%n)+.5)/f32(n)-1.;let v=2.*(f32((cell/n)%n)+.5)/f32(n)-1.;
 var dir=vec3(u,v,-1.);
 switch face {case 0u:{dir=vec3(1.,u,v);}case 1u:{dir=vec3(-1.,u,v);}case 2u:{dir=vec3(u,1.,v);}case 3u:{dir=vec3(u,-1.,v);}case 4u:{dir=vec3(u,v,1.);}default:{}}
 let region=vec3<u32>(floor((normalize(dir)+vec3(1.))*4.));
 let key=region.x+9u*region.y+81u*region.z;
 let period=(max(1u,p.dims.z)-1u)/max(12u,p.weather.z);
 let draw=f32(hash(key*7919u^period*104729u^p.dims.w)&65535u)/65536.;
 return select(1.,1.-bitcast<f32>(p.weather.y),draw<bitcast<f32>(p.weather.x));
}
@compute @workgroup_size(HISTORY_WORKGROUP_SIZE)
fn month(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=g.x+g.y*65535u*64u;if i>=p.dims.y{return;}var s=src[i];if p.options.x==2u {economies[i].production_probe=vec4(0.);weather_storage(i);}if s.stock.x<=0. {s.stock.z=0.;dst[i]=s;return;}
 var weather=(LEGACY_WEATHER_MIN_FACTOR+LEGACY_WEATHER_FACTOR_SPAN*f32(hash(p.dims.z^p.dims.w^u32(s.habitat.w)*7919u)&65535u)/65535.)*regional_weather(u32(s.habitat.z));
 if (p.options.w&2u)!=0u {let c=world[u32(s.habitat.z)];weather=clamp(c.climate.y/max(c.hydro.z,WEATHER_REFERENCE_RAIN_FLOOR),0.,MAX_REGIONAL_WEATHER_RATIO);}
 var cultivated=min(s.habitat.y,select(s.stock.x*select(LEGACY_CULTIVATED_HA_PER_PERSON,MANAGED_CULTIVATED_HA_PER_PERSON,p.options.x==2u),workers(i,s.stock.x)*worker_shares(economies[i],s.stock.x,workers(i,s.stock.x)*(1.-LAND_RECOVERY_WORK_PENALTY*select(0.,clamp(economies[i].soil.w,0.,1.),(p.options.w&2u)!=0u))).x*CULTIVATED_HECTARES_PER_WORKER_MONTH,(p.options.w&1u)==1u)); // ha, capped by available labor
 if p.options.x==2u && economies[i].farm_workers.x>.5 {
  cultivated=min(cultivated,max(0.,economies[i].farm_workers.y)*CULTIVATED_HECTARES_PER_WORKER_MONTH);
  economies[i].farm_workers.z=cultivated/CULTIVATED_HECTARES_PER_WORKER_MONTH;
 }
 if p.options.x==2u {economies[i].production_probe.y=cultivated;}
 var produced=cultivated*s.habitat.x/f32(CROP_CALENDAR_MONTHS)*select(weather,min(weather,LIVE_HARVEST_WEATHER_CAP),(p.options.w&2u)!=0u);
 if (p.options.w&2u)!=0u {let c=world[u32(s.habitat.z)];produced*=clamp((c.climate.x+SURVEY_COLD_OFFSET_C)/SURVEY_COLD_RAMP_C,0.,1.)*clamp((SURVEY_HEAT_CEILING_C-c.climate.x)/SURVEY_HEAT_RAMP_C,0.,1.)*clamp(1.-flood_depth(c),0.,1.); }
 if (p.options.w&1u)==1u {produced*=demography[i].crops.w*(1.+LEGACY_HARVEST_SEASON_AMPLITUDE*cos((f32(p.dims.z%CROP_CALENDAR_MONTHS)-demography[i].crops.z+LEGACY_HARVEST_PHASE_MONTHS)*LEGACY_HARVEST_RADIANS_PER_MONTH));}
 if economies[i].management.x>.5 {
  let t=world[u32(s.habitat.z)];
  let annual_warmth=clamp((t.hydro.y+SURVEY_COLD_OFFSET_C)/SURVEY_COLD_RAMP_C,0.,1.)*clamp((SURVEY_HEAT_CEILING_C-t.hydro.y)/SURVEY_HEAT_RAMP_C,0.,1.);
  let annual_moisture=clamp(t.hydro.z/SURVEY_RAIN_SATURATION_MM_YEAR,0.,1.)*clamp((SURVEY_RAIN_CEILING_MM_YEAR-t.hydro.z)/SURVEY_EXCESS_RAIN_RAMP_MM_YEAR,0.,1.);
  // Survey yield already embeds generic wheat climate. Recover its soil-limited
  // potential so crop-specific climate is applied once in managed_production.
  produced=cultivated*s.habitat.x/max(SURVEY_CLIMATE_DIVISOR_FLOOR,annual_warmth*annual_moisture)/f32(CROP_CALENDAR_MONTHS);
  if (p.options.w&2u)!=0u {produced*=clamp(1.-flood_depth(t),0.,1.);}
 }
 if p.options.x==2u {produced=ecological_production(i,produced*bitcast<f32>(p.options.z),weather);}
 let growth=produced;
 if (p.options.w&1u)==1u && economies[i].management.x<.5 {produced=crop_calendar(i,produced);}
 let storage=select(1.,1.-LEGACY_CONTAINER_MAX_SPOILAGE_REDUCTION*clamp(container_service(economies[i])/max(LEGACY_CONTAINER_POPULATION_FLOOR,s.stock.x*LEGACY_CONTAINER_KG_PER_PERSON),0.,1.),p.options.x==2u);
 // Basic granaries hold one harvest year; manufactured storage adds up to
 // another year. Unprotected overflow spoils after this month's consumption.
 let capacity=s.stock.x*LEGACY_RATION_KG_PER_PERSON_MONTH*(LEGACY_GRANARY_BASE_MONTHS+LEGACY_GRANARY_EXTENSION_MONTHS*(1.-storage));
 let ordinary_spoilage=s.stock.y*LEGACY_MONTHLY_FOOD_SPOILAGE*storage;
 let available=s.stock.y-ordinary_spoilage+produced;
 let need=select(s.stock.x*LEGACY_RATION_KG_PER_PERSON_MONTH,dot(demography[i].ages.xyz,vec3(CHILD_RATION_KG_PER_MONTH,ADULT_RATION_KG_PER_MONTH,ELDER_RATION_KG_PER_MONTH)),(p.options.w&1u)==1u);let entitlement=select(need,min(need,demography[i].household_food.x),demography[i].household_food.y>0.5);let eaten=min(available,entitlement);let shortage=clamp(1.-eaten/max(need,LEGACY_NEED_DIVISOR_FLOOR_KG),0.,1.);
 if (p.options.w&1u)==1u {demography[i].household_food.z=max(0.,available);}
 let overflow=max(0.,available-eaten-capacity);
 let spoilage=ordinary_spoilage+overflow;
 if p.options.x==2u {return_food(i,eaten+ordinary_spoilage);var e=economies[i];e.detritus+=vec4(overflow*LEGACY_FOOD_CNP_FRACTION,0.);economies[i]=e;}
 var births=s.stock.x*LEGACY_MONTHLY_BIRTH_RATE*(1.-shortage);var deaths=s.stock.x*(LEGACY_MONTHLY_DEATH_RATE+shortage*LEGACY_MONTHLY_STARVATION_DEATH_RATE);
 if (p.options.w&1u)==1u {let change=demographic_month(i,shortage,eaten);births=change.x;deaths=change.y;}
 s.stock.x=max(0.,s.stock.x+births-deaths);
 // Cohorts are authoritative; separately integrating their sum accumulates drift after collapse.
 if (p.options.w&1u)==1u {s.stock.x=dot(demography[i].ages.xyz,vec3(1.));}
 s.stock.y=max(0.,available-eaten-overflow);
 s.stock.z=produced;s.stock.w=shortage;
 s.ledger+=vec4(growth,eaten,spoilage,0.);s.people+=vec4(births,deaths,0.,0.);dst[i]=s;
}
