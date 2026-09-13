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
struct Params { dims:vec4<u32>, physical:vec4<f32>, counts:vec4<u32>, aux:vec4<u32>, tuning:vec4<f32> }
// Initial terrain, water and biological stocks.
const INITIAL_GREAT_LAKE_LEVEL_M:f32=120.;
const INITIAL_GREAT_LAKE_SALINITY:f32=.05;
const INITIAL_TERRAIN_NOISE_FREQUENCY:f32=9.;
const INITIAL_RIDGE_FREQUENCY:f32=24.;
const INITIAL_RIDGE_WARP_FREQUENCY:f32=8.;
const INITIAL_RIDGE_WARP_AMPLITUDE:f32=2.;
const INITIAL_RIDGE_EXPONENT:f32=3.;
const INITIAL_DETAIL_MAX_FREQUENCY:f32=100.;
const INITIAL_DETAIL_RESOLUTION_SCALE:f32=.4;
const INITIAL_LAND_BASE_M:f32=250.;
const INITIAL_LAND_RELIEF_M:f32=850.;
const INITIAL_MOUNTAIN_STRESS_EXPONENT:f32=2.;
const INITIAL_MOUNTAIN_BASE_M:f32=500.;
const INITIAL_MOUNTAIN_RIDGE_HEIGHT_M:f32=3300.;
const INITIAL_DETAIL_HEIGHT_M:f32=220.;
const INITIAL_OCEAN_BED_M:f32=-3800.;
const INITIAL_OCEAN_RELIEF_M:f32=1800.;
const INITIAL_LAKE_SHELF_WIDTH_RAD:f32=.16;
const INITIAL_LAKE_SHELF_BED_M:f32=60.;
const INITIAL_DEEP_LAKE_BED_M:f32=-6000.;
const INITIAL_LAKE_BED_RELIEF_M:f32=1200.;
const INITIAL_SOIL_BASE_M:f32=.2;
const INITIAL_CRUST_AGE_FREQUENCY:f32=7.;
const INITIAL_MAX_CRUST_AGE_MYR:f32=1500.;
const INITIAL_EQUATOR_TEMPERATURE_C:f32=30.;
const INITIAL_POLE_COOLING_C:f32=60.;
const INITIAL_RAIN_MM_YEAR:f32=800.;
const INITIAL_VAPOR_MM:f32=20.;
const INITIAL_WIND_M_S:f32=8.;
const INITIAL_GROUNDWATER_M:f32=.1;
const INITIAL_VEGETATION_COVER:f32=.2;
const INITIAL_FERTILITY:f32=.5;
const INITIAL_OCEAN_SALINITY:f32=35.;
const INITIAL_RUNOFF_M_YEAR:f32=.2;
const INITIAL_LAND_CRUST_KM:f32=35.;
const INITIAL_OCEAN_CRUST_KM:f32=7.;
const INITIAL_IGNEOUS_STRESS_THRESHOLD:f32=.5;
const INITIAL_METAMORPHIC_STRESS_THRESHOLD:f32=.8;
const INITIAL_TOP_STRATUM_BASE_M:f32=100.;
const INITIAL_TOP_STRATUM_SPAN_M:f32=400.;
const INITIAL_MIDDLE_STRATUM_BASE_M:f32=500.;
const INITIAL_STRATUM_NOISE_FREQUENCY:f32=11.;
const INITIAL_MIDDLE_STRATUM_SPAN_M:f32=2000.;

// Artistic continent masks and guaranteed lake separation.
const INNER_PLACEMENT_JITTER:f32=.18;
const INNER_PLACEMENT_PHASE_RAD:f32=.23;
const INNER_CENTER_MIN_ANGLE_RAD:f32=.32;
const INNER_CENTER_ANGLE_SPAN_RAD:f32=.12;
const INNER_BASE_RADIUS_CAP_RAD:f32=.195;
const INNER_RADIUS_COUNT_SCALE:f32=.82;
const INNER_SIZE_MIN_FACTOR:f32=.65;
const INNER_SIZE_MAX_FACTOR:f32=1.15;
const INNER_ASPECT_MIN:f32=.48;
const INNER_ASPECT_SPAN:f32=.44;
const INNER_ASPECT_RANK_SCALE:f32=1.618;
const INNER_ASPECT_PHASE:f32=.17;
const INNER_MIN_LOBES:f32=2.;
const INNER_LOBE_COUNT_SPAN:f32=3.;
const INNER_OUTLINE_BASE:f32=.84;
const INNER_OUTLINE_PRIMARY_AMPLITUDE:f32=.20;
const INNER_OUTLINE_PHASE_SPAN_RAD:f32=6.28;
const INNER_OUTLINE_LOBE_AMPLITUDE:f32=.10;
const INNER_OUTLINE_DETAIL_AMPLITUDE:f32=.05;
const INNER_OUTLINE_DETAIL_FREQUENCY:f32=5.;
const INNER_OUTLINE_NOISE_AMPLITUDE:f32=.16;
const INNER_OUTLINE_NOISE_FREQUENCY:f32=65.;
const INNER_RADIUS_CAP_RAD:f32=.21;
const INNER_CHANNEL_RADIUS_FACTOR:f32=.90;
const INNER_CHANNEL_SPACING_FACTOR:f32=.41;
const ENCLOSING_SHORE_ANGLE_RAD:f32=.92;
const ENCLOSING_SHORE_PRIMARY_AMPLITUDE_RAD:f32=.035;
const ENCLOSING_SHORE_PRIMARY_FREQUENCY:f32=5.;
const ENCLOSING_SHORE_DETAIL_AMPLITUDE_RAD:f32=.018;
const ENCLOSING_SHORE_DETAIL_FREQUENCY:f32=11.;
const EXTERIOR_SHORE_ANGLE_RAD:f32=1.67;
const EXTERIOR_SHORE_NOISE_AMPLITUDE_RAD:f32=.4;
const EXTERIOR_SHORE_NOISE_FREQUENCY:f32=5.;
const EXTERIOR_SHORE_WAVE_AMPLITUDE_RAD:f32=.055;
const EXTERIOR_SHORE_WAVE_FREQUENCY:f32=7.;

// Plate-field motion and geological activity; rates use the geological clock.
const PLATE_COUNT:u32=16u;
const PLATE_BASE_ANGULAR_SPEED_PER_MYR:f32=.002;
const PLATE_ANGULAR_SPEED_SPAN_PER_MYR:f32=.004;
const PLATE_WARP_FREQUENCY:f32=5.;
const PLATE_WARP_AMPLITUDE:f32=.7;
const PLATE_CONVERGENCE_SPEED_SCALE:f32=.008;
const PLATE_BOUNDARY_FALLOFF:f32=38.;
const OUTER_ACTIVITY_BASE:f32=.3;
const OUTER_ACTIVITY_BELT_GAIN:f32=.7;
const OUTER_ACTIVITY_BELT_START:f32=.28;
const OUTER_ACTIVITY_BELT_FULL:f32=.65;
const OUTER_ACTIVITY_NOISE_FREQUENCY:f32=13.;
const INNER_ACTIVITY_BASE:f32=.12;
const INNER_ACTIVITY_BELT_GAIN:f32=.88;
const INNER_ACTIVITY_STRESS_START:f32=.8;
const INNER_ACTIVITY_STRESS_FULL:f32=.97;
const TECTONIC_UPLIFT_M_PER_MYR:f32=1000.;
const KM_PER_METER:f32=.001;
const MIN_CRUST_THICKNESS_KM:f32=5.;
const MAX_CRUST_THICKNESS_KM:f32=80.;
const VOLCANIC_BOUNDARY_STRENGTH_MIN:f32=.85;
const VOLCANIC_CONVERGENCE_MIN:f32=.4;
const VOLCANIC_AGE_REDUCTION_PER_MYR:f32=100.;
const METAMORPHIC_BOUNDARY_STRENGTH_MIN:f32=.9;
const METAMORPHIC_DIVERGENCE_THRESHOLD:f32=-.3;
const TECTONIC_ROCK_PROVINCE_FREQUENCY:f32=9.;

// Erosion and terrain constraints, after the shared Cell ABI prefix.
const METERS_PER_KM:f32=1000.;
const MIN_EROSION_DISTANCE_M:f32=1.;
const THERMAL_EROSION_MIN_SLOPE:f32=.025;
const THERMAL_MOBILE_SEDIMENT_CAP_M:f32=.1;
const THERMAL_EROSION_RATE:f32=.03;
const EROSION_VEGETATION_PROTECTION:f32=.7;
const FLUVIAL_STANDING_WATER_CUTOFF_M:f32=.1;
const MAX_FLUVIAL_EROSION_M:f32=.08;
const FLUVIAL_DISCHARGE_EROSION_GAIN:f32=.002;
const MIN_EROSION_ROCK_HARDNESS:f32=1.;
const MIN_EXTERIOR_OCEAN_ELEVATION_M:f32=-9000.;
const MAX_EXTERIOR_OCEAN_ELEVATION_M:f32=-20.;
const MIN_GREAT_LAKE_BED_ELEVATION_M:f32=-6000.;
const MAX_GREAT_LAKE_BED_ELEVATION_M:f32=80.;
const MIN_CONSTRAINED_LAND_ELEVATION_M:f32=250.;
const MAX_CONSTRAINED_LAND_ELEVATION_M:f32=9500.;
const BEDROCK_WEATHERING_GAIN:f32=.003;
const FROZEN_WEATHERING_REDUCTION:f32=.8;
const MAX_WEATHERED_SOIL_DEPTH_M:f32=20.;
const LITHIFICATION_SEDIMENT_RESERVE_M:f32=5.;
const MAX_LITHIFICATION_SHARE:f32=.1;
const LITHIFICATION_SHARE_PER_MYR:f32=.01;
const SEDIMENTARY_PROVINCE_NOISE_FREQUENCY:f32=9.;
const SEDIMENTARY_PROVINCE_NOISE_OFFSET:f32=.1;

// Climate and hydrology parameters, after the shared Cell ABI prefix.
const CLIMATE_WET_DEPTH_M:f32=.1;
const CLIMATE_BASE_TEMPERATURE_C:f32=-20.;
const CLIMATE_SOLAR_WARMING_C:f32=55.;
const CLIMATE_LAPSE_C_PER_M:f32=.006;
const MIN_CLIMATE_TEMPERATURE_C:f32=-90.;
const MAX_CLIMATE_TEMPERATURE_C:f32=55.;
const LAND_TEMPERATURE_RESPONSE:f32=.35;
const WATER_TEMPERATURE_RESPONSE:f32=.12;
const WIND_AXIS_EPSILON:f32=.00001;
const WIND_REVERSAL_Y:f32=.5;
const CONVECTION_BASE:f32=.015;
const CONVECTION_SOLAR_GAIN:f32=.08;
const CONVECTION_SOLAR_EXPONENT:f32=4.;
const UPSTREAM_VAPOR_SHARE:f32=.65;
const LAND_VAPOR_BASE:f32=.15;
const VEGETATION_VAPOR_GAIN:f32=.7;
const GROUNDWATER_VAPOR_GAIN:f32=2.;
const GROUNDWATER_VAPOR_HALF_SATURATION_M:f32=.1;
const WATER_VAPOR_BASE:f32=1.5;
const WATER_VAPOR_TEMPERATURE_GAIN:f32=.05;
const OROGRAPHIC_PRECIPITATION_PER_M:f32=.00012;
const MAX_PRECIPITATION_PER_PASS:f32=30.;
const MAX_VAPOR_STORAGE:f32=80.;
const PRECIPITATION_ANNUALIZATION:f32=365.;
const PREVAILING_WIND_BASE_M_S:f32=5.;
const PREVAILING_WIND_AMPLITUDE_M_S:f32=10.;
const WIND_LATITUDE_FREQUENCY:f32=3.;
const SEASONAL_WIND_AMPLITUDE_M_S:f32=2.;
const SEASONAL_WIND_RADIANS:f32=6.28;
const WIND_EPOCH_PHASE_RADIANS:f32=.3;
const METERS_PER_MM:f32=.001;
const SECONDS_PER_RUNOFF_YEAR:f32=31557600.;
const SNOW_MELT_M_PER_C:f32=.025;
const EVAPORATION_TEMPERATURE_OFFSET_C:f32=5.;
const EVAPORATION_M_PER_C:f32=.015;
const GROUNDWATER_CAPACITY_M:f32=100.;
const INFILTRATION_PERMEABILITY_SHARE:f32=.6;
const GROUNDWATER_RELEASE_SHARE:f32=.04;
const CLIMATE_TEMPERATURE_TOLERANCE_C:f32=.25;
const CLIMATE_RAIN_TOLERANCE_MM_YEAR:f32=10.;
const CLIMATE_VAPOR_TOLERANCE:f32=.1;
const SECONDARY_LAKE_MIN_DEPRESSION_M:f32=.05;
const POOL_AREA_WEIGHT_SCALE:f32=4096.;
const POOL_OVERFLOW_TOLERANCE_M:f32=.0001;
const POOL_HEAD_ABSOLUTE_TOLERANCE_M:f32=.0005;
const POOL_HEAD_RELATIVE_TOLERANCE:f32=.0000005;
const POOL_MAX_EDGE_OUTFLOW_SHARE:f32=.24;
const POOL_HEAD_TRANSFER_SHARE:f32=.48;
const LAKE_LEDGER_SCALE:f32=1e-12;
const LAKE_RIVER_INPUT_SALINITY:f32=.05;
const LAKE_DIVISION_FLOOR:f32=1e-10;
const MIN_GREAT_LAKE_LEVEL_M:f32=85.;
const MAX_GREAT_LAKE_LEVEL_M:f32=240.;
const MAX_GREAT_LAKE_SALINITY:f32=200.;

struct Entry { a:vec4<f32>, b:vec4<f32>, c:vec4<f32>, d:vec4<f32>, ids:vec4<u32> }
struct Flags { changed:atomic<u32>, invalid:atomic<u32>, reserved:atomic<u32>, pad:atomic<u32> }
@group(0) @binding(0) var<storage,read> src:array<Cell>;
@group(0) @binding(1) var<storage,read_write> dst:array<Cell>;
@group(0) @binding(2) var<uniform> p:Params;
@group(0) @binding(3) var<storage,read_write> flags:Flags;
@group(0) @binding(4) var<storage,read> catalog:array<Entry>;
@group(0) @binding(5) var<storage,read_write> scratch:array<vec4<f32>>;
@group(0) @binding(6) var<storage,read_write> planet:array<vec4<f32>>;
const NONE:u32=0xffffffffu;
const PI:f32=3.14159265359;
fn cell_id(g:vec3<u32>)->u32 { return g.z*p.dims.x*p.dims.x+g.y*p.dims.x+g.x; }
fn hash(x:u32)->u32 { var v=x; v=(v^(v>>16u))*0x7feb352du; v=(v^(v>>15u))*0x846ca68bu; return v^(v>>16u); }
fn rand(x:u32)->f32 { return f32(hash(x^p.dims.y)&0xffffffu)/16777216.; }
fn lattice(q:vec3<i32>)->f32 { return rand(bitcast<u32>(q.x)*73856093u^bitcast<u32>(q.y)*19349663u^bitcast<u32>(q.z)*83492791u); }
fn noise(q:vec3<f32>)->f32 {
 let i=vec3<i32>(floor(q)); let a=fract(q); let t=a*a*(3.-2.*a);
 return mix(mix(mix(lattice(i),lattice(i+vec3(1,0,0)),t.x),mix(lattice(i+vec3(0,1,0)),lattice(i+vec3(1,1,0)),t.x),t.y),mix(mix(lattice(i+vec3(0,0,1)),lattice(i+vec3(1,0,1)),t.x),mix(lattice(i+vec3(0,1,1)),lattice(i+vec3(1,1,1)),t.x),t.y),t.z);
}
fn fbm(q:vec3<f32>)->f32 { return noise(q)*.55+noise(q*2.03)*.27+noise(q*4.11)*.13+noise(q*8.23)*.05; }
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
fn area(i:u32)->f32 {
 // Four-point quadrature avoids catastrophic cancellation in corner-angle differences at 1024².
 let n=f32(p.dims.x);let uv=2.*(vec2<f32>(vec2<u32>(i%p.dims.x,(i/p.dims.x)%p.dims.x))+.5)/n-1.;
 let t=1./(n*sqrt(3.));var sum=0.;
 for(var k=0u;k<4u;k++){let q=uv+vec2(select(-t,t,(k&1u)!=0u),select(-t,t,(k&2u)!=0u));sum+=pow(1.+dot(q,q),-1.5);}
 return sum/(n*n)*p.physical.x*p.physical.x*1e6;
}
fn inner_distance(d:vec3<f32>,j:u32)->f32 {

  // Bounded placement leaves a broad open-water belt inside the enclosing shore.
  let key=j*131u+1701u;
  let spacing=2.*PI/f32(p.dims.w);
  let a=spacing*(f32(j)+INNER_PLACEMENT_JITTER*(rand(key)-.5))+INNER_PLACEMENT_PHASE_RAD;
  let r=INNER_CENTER_MIN_ANGLE_RAD+INNER_CENTER_ANGLE_SPAN_RAD*rand(key+1u);
  let center=vec3(sin(r)*cos(a),sin(r)*sin(a),cos(r));
  let dist=acos(clamp(dot(d,center),-1.,1.));
  let bearing=atan2(dot(d,vec3(-sin(a),cos(a),0.)),dot(d,vec3(cos(r)*cos(a),cos(r)*sin(a),-sin(r))));
  // A permuted size sequence guarantees variety even when random samples agree.
  let rank=f32((j+p.dims.y%p.dims.w)%p.dims.w)/f32(p.dims.w-1u);
  let size=min(INNER_BASE_RADIUS_CAP_RAD,INNER_RADIUS_COUNT_SCALE/f32(p.dims.w))*mix(INNER_SIZE_MIN_FACTOR,INNER_SIZE_MAX_FACTOR,rank);
  let orientation=rand(key+2u)*2.*PI;
  let theta=bearing-orientation;
  let aspect=INNER_ASPECT_MIN+INNER_ASPECT_SPAN*fract(rank*INNER_ASPECT_RANK_SCALE+INNER_ASPECT_PHASE);
  let ellipse=inverseSqrt(cos(theta)*cos(theta)+sin(theta)*sin(theta)/(aspect*aspect));
  let lobes=INNER_MIN_LOBES+floor(rand(key+4u)*INNER_LOBE_COUNT_SPAN);
  let outline=INNER_OUTLINE_BASE+INNER_OUTLINE_PRIMARY_AMPLITUDE*sin(theta+rand(key+3u)*INNER_OUTLINE_PHASE_SPAN_RAD)
      +INNER_OUTLINE_LOBE_AMPLITUDE*sin(theta*lobes+rand(key+5u)*INNER_OUTLINE_PHASE_SPAN_RAD)+INNER_OUTLINE_DETAIL_AMPLITUDE*sin(theta*INNER_OUTLINE_DETAIL_FREQUENCY+rand(key+6u)*INNER_OUTLINE_PHASE_SPAN_RAD)+INNER_OUTLINE_NOISE_AMPLITUDE*(fbm(d*INNER_OUTLINE_NOISE_FREQUENCY+f32(j)*17.)-.5);
  // Explicit cap also protects channels at the maximum continent count.
  let radius=min(size*ellipse*outline,min(INNER_RADIUS_CAP_RAD,INNER_CHANNEL_RADIUS_FACTOR*sin(spacing*INNER_CHANNEL_SPACING_FACTOR)*sin(INNER_CENTER_MIN_ANGLE_RAD)));
  return dist-radius;
}
fn enclosing_shore(d:vec3<f32>)->f32 {
 let az=atan2(d.y,d.x);return ENCLOSING_SHORE_ANGLE_RAD+ENCLOSING_SHORE_PRIMARY_AMPLITUDE_RAD*sin(az*ENCLOSING_SHORE_PRIMARY_FREQUENCY+f32(p.dims.y%99u))+ENCLOSING_SHORE_DETAIL_AMPLITUDE_RAD*sin(az*ENCLOSING_SHORE_DETAIL_FREQUENCY);
}
fn region(d:vec3<f32>)->u32 {
 let angle=acos(clamp(d.z,-1.,1.));let az=atan2(d.y,d.x);
 if angle>EXTERIOR_SHORE_ANGLE_RAD+EXTERIOR_SHORE_NOISE_AMPLITUDE_RAD*(fbm(d*EXTERIOR_SHORE_NOISE_FREQUENCY)-.5)+EXTERIOR_SHORE_WAVE_AMPLITUDE_RAD*sin(az*EXTERIOR_SHORE_WAVE_FREQUENCY) {return 0u;}
 if angle>=enclosing_shore(d) {return 3u;}
 for(var j=0u;j<p.dims.w;j++){if inner_distance(d,j)<0. {return 2u;}}
 return 1u;
}
fn lake_shore_distance(d:vec3<f32>)->f32 {
 var distance=enclosing_shore(d)-acos(clamp(d.z,-1.,1.));
 for(var j=0u;j<p.dims.w;j++){distance=min(distance,inner_distance(d,j));}
 return max(0.,distance);
}

fn constrain(h:f32,r:u32)->f32 { switch r { case 0u:{return clamp(h,MIN_EXTERIOR_OCEAN_ELEVATION_M,MAX_EXTERIOR_OCEAN_ELEVATION_M);} case 1u:{return clamp(h,MIN_GREAT_LAKE_BED_ELEVATION_M,MAX_GREAT_LAKE_BED_ELEVATION_M);} default:{return clamp(h,MIN_CONSTRAINED_LAND_ELEVATION_M,MAX_CONSTRAINED_LAND_ELEVATION_M);} } }
fn plate_axis(j:u32)->vec3<f32> {
 return normalize(vec3(rand(j*43u+811u),rand(j*71u+813u),rand(j*97u+817u))-.5);
}
fn plate_seed(j:u32)->vec3<f32> {
 let z=rand(j*31u+919u)*2.-1.;let a=rand(j*79u+33u)*2.*PI;
 let base=vec3(sqrt(1.-z*z)*cos(a),z,sqrt(1.-z*z)*sin(a));
 let axis=plate_axis(j);let t=p.tuning.z*(PLATE_BASE_ANGULAR_SPEED_PER_MYR+PLATE_ANGULAR_SPEED_SPAN_PER_MYR*rand(j+91u));
 return base*cos(t)+cross(axis,base)*sin(t)+axis*dot(axis,base)*(1.-cos(t));
}
fn plate(point:vec3<f32>)->vec3<f32> {
 let warp=vec3(fbm(point*PLATE_WARP_FREQUENCY+17.),fbm(point*PLATE_WARP_FREQUENCY+39.),fbm(point*PLATE_WARP_FREQUENCY+71.))-.5;
 let d=normalize(point+warp*PLATE_WARP_AMPLITUDE);
 var best=-2.;var second=-2.;var id=0u;var other=0u;
 for(var j=0u;j<PLATE_COUNT;j++){let s=dot(d,plate_seed(j));if s>best {second=best;other=id;best=s;id=j;} else if s>second {second=s;other=j;}}
 let boundary=normalize(plate_seed(other)-plate_seed(id));
 let velocity=cross(plate_axis(id)*(PLATE_BASE_ANGULAR_SPEED_PER_MYR+PLATE_ANGULAR_SPEED_SPAN_PER_MYR*rand(id+91u)),point);
 let adjacent=cross(plate_axis(other)*(PLATE_BASE_ANGULAR_SPEED_PER_MYR+PLATE_ANGULAR_SPEED_SPAN_PER_MYR*rand(other+91u)),point);
 let convergence=clamp(dot(velocity-adjacent,boundary)/PLATE_CONVERGENCE_SPEED_SCALE,-1.,1.);
 return vec3(f32(id),exp(-(best-second)*PLATE_BOUNDARY_FALLOFF),convergence);
}
fn geological_activity(d:vec3<f32>,stress:f32,r:u32)->f32 {
 // Regional source distributions: old outer plateaus coexist with rejuvenated belts;
 // inner continents keep uncommon but fully active geothermal exceptions.
 if r==3u {return stress*(OUTER_ACTIVITY_BASE+OUTER_ACTIVITY_BELT_GAIN*smoothstep(OUTER_ACTIVITY_BELT_START,OUTER_ACTIVITY_BELT_FULL,fbm(d*OUTER_ACTIVITY_NOISE_FREQUENCY+71.)));}
 if r==2u {return stress*(INNER_ACTIVITY_BASE+INNER_ACTIVITY_BELT_GAIN*smoothstep(INNER_ACTIVITY_STRESS_START,INNER_ACTIVITY_STRESS_FULL,stress));}
 return stress;
}
fn rock_for(f:u32,r:f32)->u32 {
 var id=0u;var score=-1.;for(var j=0u;j<p.counts.x;j++){if catalog[j].ids.x==f {let s=rand(j*117u+u32(r*100000.));if s>score {id=j;score=s;}}}return id;
}
// Continuous world-space fields do not depend on cube-face indexing or resolution.
fn province_rock(f:u32,d:vec3<f32>,legacy:f32)->u32 {
 if p.tuning.w==0. {return rock_for(f,legacy);}
 var id=0u;var best=-1.;
 for(var j=0u;j<p.counts.x;j++) {if catalog[j].ids.x!=f {continue;}
 let salt=catalog[j].ids.y;let offset=vec3(rand(salt),rand(salt+1u),rand(salt+2u))*200.;
 let score=noise(d*24.+offset);if score>best {best=score;id=j;}}
 return id;
}
// Broad depositional/exposure settings; these are regional proxies, not a basin solver.
fn geological_setting(d:vec3<f32>,pl:vec3<f32>,h:f32,r:u32)->u32 {
 let basin=noise(d*3.+vec3(19.,37.,71.));
 let arc=pl.y*smoothstep(.02,.45,pl.z);
 if r==0u {return 1u;}
 if arc>.48 {return 1u;}
 if pl.y>.66 && pl.z>.12 && h>700. {return 6u;}
 if pl.y>.55 && pl.z<-.22 && h>1100. {return 8u;}
 if h>1050. && basin<.5 {return 2u;}
 if arc>.26 && h>800. {return 7u;}
 if r==1u || basin>.56 {
  if r>=2u && abs(d.y)>.2 && abs(d.y)<.6 && basin>.76 && pl.y<.25 {return 5u;}
  return 3u;
 }
 return 4u;
}
fn setting_rock(setting:u32,d:vec3<f32>)->u32 {
 var formation=0u;if setting>=3u && setting<=5u {formation=1u;}
 if setting==6u || setting==7u {formation=2u;}
 let warp=vec3(noise(d*3.+11.),noise(d*3.+37.),noise(d*3.+73.))-.5;
 let q=d*5.+warp*.7;
 var id=NONE;var best=-1.;
 for(var j=0u;j<p.counts.x;j++) {
  if catalog[j].ids.x!=formation || (catalog[j].ids.z!=0u && catalog[j].ids.z!=setting) {continue;}
  let salt=catalog[j].ids.y;
  let offset=vec3(rand(salt),rand(salt+1u),rand(salt+2u))*200.;
  let score=noise(q+offset);if score>best {best=score;id=j;}
 }
 if id==NONE {return province_rock(formation,d,noise(d*5.));}return id;
}
fn deposit_environment(c:Cell,setting:u32)->f32 {
 let activity=clamp(c.geology.x,0.,1.);let young=exp(-c.terrain.w/500.);
 let wet=clamp(c.hydro.z/2000.,0.,1.);let warmth=clamp((c.hydro.y+5.)/30.,0.,1.);
 let cover=clamp(c.terrain.y/20.,0.,1.);
 switch setting {
 case 1u: {return .15+.85*max(activity,young);}
 case 2u: {return (.15+.85*activity)*(.35+.65*max(wet,clamp(c.water.y,0.,1.)));}
 case 3u: {return .35+.65*max(cover,1.-activity);}
 case 4u: {return warmth*wet*(.2+.8*(1.-young));}
 case 5u: {return .2+.8*max(activity,clamp((c.geology.y-25.)/35.,0.,1.));}
 case 6u: {return (1.-wet)*(.25+.75*max(cover,select(0.,1.,c.water.x>0.)));}
 default: {return 1.;}
 }
}
fn deposit_potential(c:Cell,d:vec3<f32>,e:Entry)->f32 {
 let salt=e.ids.w;let offset=vec3(rand(salt),rand(salt+1u),rand(salt+2u))*200.;
 let field=noise(d*(p.physical.x/e.b.x)+offset);
 let province=smoothstep(.3,.75,field);
 return e.a.x*province*deposit_environment(c,e.ids.z);
}
fn column_present(c:Cell)->bool {return dot(c.strata.xyz,vec3(1.))+c.strata.w>0.;}
// Remove finite bedrock in order; retain the deepest identity if fully exhausted.
fn cut_column(input:Cell,amount:f32)->Cell {
 var c=input;var remaining=min(amount,dot(c.strata.xyz,vec3(1.)));
 c.strata.w+=remaining;
 for(var k=0u;k<3u;k++) {
  let cut=min(remaining,c.strata.x);c.strata.x-=cut;remaining-=cut;
  if c.strata.x<=0. && c.strata.y+c.strata.z>0. {
   c.ids.x=c.tags.z;c.tags.z=c.tags.w;c.strata.x=c.strata.y;c.strata.y=c.strata.z;c.strata.z=0.;
  }
 }
 return c;
}
fn lithify_column(input:Cell,amount:f32,rock:u32)->Cell {
 var c=input;if amount<=0. {return c;}
 if c.ids.x!=rock {
  // Bounded compression: combine the deepest two units under the middle unit's ID.
  c.strata.z+=c.strata.y;c.strata.y=c.strata.x;c.strata.x=0.;
  c.tags.w=c.tags.z;c.tags.z=c.ids.x;c.ids.x=rock;
 }
 c.strata.x+=amount;c.terrain.y-=amount;return c;
}
@compute @workgroup_size(TERRAIN_WORKGROUP_EDGE,TERRAIN_WORKGROUP_EDGE)
fn initialize(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=cell_id(g);if i==0u {planet[0]=vec4(INITIAL_GREAT_LAKE_LEVEL_M,INITIAL_GREAT_LAKE_SALINITY,0.,0.);}
 let d=pos(i);let r=region(d);let pl=plate(d);let f=fbm(d*INITIAL_TERRAIN_NOISE_FREQUENCY);
 // Broad mountain belts contain branching ridges and lower intervening valleys.
 let warped=d*INITIAL_RIDGE_FREQUENCY+vec3(fbm(d*INITIAL_RIDGE_WARP_FREQUENCY),fbm(d*INITIAL_RIDGE_WARP_FREQUENCY+19.),fbm(d*INITIAL_RIDGE_WARP_FREQUENCY+47.))*INITIAL_RIDGE_WARP_AMPLITUDE;
 let ridge=pow(1.-abs(noise(warped)*2.-1.),INITIAL_RIDGE_EXPONENT);
 let detail=fbm(d*min(INITIAL_DETAIL_MAX_FREQUENCY,f32(p.dims.x)*INITIAL_DETAIL_RESOLUTION_SCALE));
 var h=INITIAL_LAND_BASE_M+f*INITIAL_LAND_RELIEF_M+pow(pl.y,INITIAL_MOUNTAIN_STRESS_EXPONENT)*(INITIAL_MOUNTAIN_BASE_M+INITIAL_MOUNTAIN_RIDGE_HEIGHT_M*ridge)+detail*INITIAL_DETAIL_HEIGHT_M;
 if r==0u {h=INITIAL_OCEAN_BED_M+f*INITIAL_OCEAN_RELIEF_M;} if r==1u {
  let shelf=smoothstep(0.,INITIAL_LAKE_SHELF_WIDTH_RAD,lake_shore_distance(d));
  h=mix(INITIAL_LAKE_SHELF_BED_M,INITIAL_DEEP_LAKE_BED_M+f*INITIAL_LAKE_BED_RELIEF_M,shelf);
 }
 var c:Cell;c.terrain=vec4(constrain(h,r),0.,INITIAL_SOIL_BASE_M+f,select(rand(i+7u),fbm(d*INITIAL_CRUST_AGE_FREQUENCY+113.),p.tuning.w>0.)*INITIAL_MAX_CRUST_AGE_MYR);
 c.climate=vec4(INITIAL_EQUATOR_TEMPERATURE_C-INITIAL_POLE_COOLING_C*abs(d.y)-max(h,0.)*CLIMATE_LAPSE_C_PER_M,INITIAL_RAIN_MM_YEAR,INITIAL_VAPOR_MM,INITIAL_WIND_M_S);
 c.water=vec4(select(0.,max(0.,select(0.,INITIAL_GREAT_LAKE_LEVEL_M,r==1u)-c.terrain.x),r<2u),INITIAL_GROUNDWATER_M,0.,0.);
 c.life=vec4(INITIAL_VEGETATION_COVER,INITIAL_FERTILITY,select(0.,INITIAL_OCEAN_SALINITY,r==0u),INITIAL_RUNOFF_M_YEAR);
 c.geology=vec4(geological_activity(d,pl.y,r),select(INITIAL_LAND_CRUST_KM,INITIAL_OCEAN_CRUST_KM,r==0u),0.,0.);
 c.hydro=vec4(0.,c.climate.x,INITIAL_RAIN_MM_YEAR,0.);
 let formation=select(select(1u,0u,pl.y>INITIAL_IGNEOUS_STRESS_THRESHOLD),2u,pl.y>INITIAL_METAMORPHIC_STRESS_THRESHOLD);
 c.ids=vec4(province_rock(formation,d,f),0u,NONE,0u);
 if p.tuning.w>1. {c.ids.x=setting_rock(geological_setting(d,pl,h,r),d);}
 c.routing=vec4(NONE,NONE,NONE,u32(pl.x));c.tags=vec4(r,NONE,province_rock(1u,d,f+.1),province_rock(0u,d,f+.2));
 if p.tuning.w>1. {c.tags.z=setting_rock(select(4u,3u,r<2u),d);c.tags.w=setting_rock(select(2u,1u,r==0u),d);}c.budget=vec4(0.);
 c.strata=vec4(INITIAL_TOP_STRATUM_BASE_M+f*INITIAL_TOP_STRATUM_SPAN_M,INITIAL_MIDDLE_STRATUM_BASE_M+fbm(d*INITIAL_STRATUM_NOISE_FREQUENCY+29.)*INITIAL_MIDDLE_STRATUM_SPAN_M,0.,0.);c.strata.z=max(0.,c.geology.y*METERS_PER_KM-c.strata.x-c.strata.y);
 dst[i]=c;
}
@compute @workgroup_size(TERRAIN_WORKGROUP_EDGE,TERRAIN_WORKGROUP_EDGE)
fn tectonics(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=cell_id(g);var c=src[i];let d=pos(i);let pl=plate(d);let dt=p.physical.z;
 let uplift=pl.y*pl.z*TECTONIC_UPLIFT_M_PER_MYR*dt; c.terrain.x=constrain(c.terrain.x+uplift,c.tags.x);
 let old_crust=c.geology.y;
 c.terrain.w+=dt;c.routing.w=u32(pl.x);c.geology.x=geological_activity(d,pl.y,c.tags.x);c.geology.y=clamp(c.geology.y+uplift*KM_PER_METER,MIN_CRUST_THICKNESS_KM,MAX_CRUST_THICKNESS_KM);
 if column_present(c) {c.strata.z=max(0.,c.strata.z+(c.geology.y-old_crust)*METERS_PER_KM);}
 if pl.y>VOLCANIC_BOUNDARY_STRENGTH_MIN && pl.z>VOLCANIC_CONVERGENCE_MIN {c.ids.x=province_rock(0u,d,fbm(d*TECTONIC_ROCK_PROVINCE_FREQUENCY));if p.tuning.w>1. {c.ids.x=setting_rock(1u,d);}c.terrain.w=max(0.,c.terrain.w-dt*VOLCANIC_AGE_REDUCTION_PER_MYR);}
 if pl.y>METAMORPHIC_BOUNDARY_STRENGTH_MIN && select(pl.z<METAMORPHIC_DIVERGENCE_THRESHOLD,geological_setting(d,pl,c.terrain.x,c.tags.x)==6u,p.tuning.w>1.) {c.ids.x=province_rock(2u,d,fbm(d*TECTONIC_ROCK_PROVINCE_FREQUENCY));if p.tuning.w>1. {c.ids.x=setting_rock(6u,d);}}
 dst[i]=c;
}
@compute @workgroup_size(TERRAIN_WORKGROUP_EDGE,TERRAIN_WORKGROUP_EDGE)
fn drain_init(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=cell_id(g);var c=src[i];c.routing.x=NONE;c.routing.y=NONE;c.routing.z=NONE;c.hydro.x=1e20;
 if c.tags.x<2u {c.hydro.x=select(0.,planet[0].x,c.tags.x==1u);c.routing.y=0u;c.routing.z=c.tags.x;}
 scratch[i]=vec4(c.hydro.x,bitcast<vec3<f32>>(c.routing.xyz));dst[i]=c;
}
// Only the 16-byte drainage tuple moves during relaxation, not all 160 bytes
// of terrain, climate and ecology. Both sides reuse the existing reduction scratch.
fn drain_compact(i:u32,side:u32) {
 let count=6u*p.dims.x*p.dims.x;let offset=side*count;
 let old=scratch[offset+i];var value=old;var route=bitcast<vec3<u32>>(old.yzw);
 if src[i].tags.x>=2u {
  for(var k=0u;k<4u;k++){let j=neighbor(i,k);let b=scratch[offset+j];let r=bitcast<vec3<u32>>(b.yzw);if r.y==NONE {continue;}
   let spill=max(src[i].terrain.x,b.x);let rank=r.y+1u;
   if spill<value.x || (spill==value.x && (rank<route.y || (rank==route.y && j<route.x))) {
    value.x=spill;route=vec3(j,rank,r.z);
   }
  }
 }
 if value.x!=old.x || any(route!=bitcast<vec3<u32>>(old.yzw)) {atomicAdd(&flags.changed,1u);}
 scratch[(1u-side)*count+i]=vec4(value.x,bitcast<vec3<f32>>(route));
}
@compute @workgroup_size(TERRAIN_WORKGROUP_EDGE,TERRAIN_WORKGROUP_EDGE)
fn drain_even(@builtin(global_invocation_id) g:vec3<u32>) {drain_compact(cell_id(g),0u);}
@compute @workgroup_size(TERRAIN_WORKGROUP_EDGE,TERRAIN_WORKGROUP_EDGE)
fn drain_odd(@builtin(global_invocation_id) g:vec3<u32>) {drain_compact(cell_id(g),1u);}
@compute @workgroup_size(TERRAIN_WORKGROUP_EDGE,TERRAIN_WORKGROUP_EDGE)
fn drain_scatter(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=cell_id(g);var c=src[i];let value=scratch[(p.aux.y%2u)*6u*p.dims.x*p.dims.x+i];
 c.hydro.x=value.x;c.routing=vec4(bitcast<vec3<u32>>(value.yzw),c.routing.w);dst[i]=c;
}
@compute @workgroup_size(TERRAIN_WORKGROUP_EDGE,TERRAIN_WORKGROUP_EDGE)
fn basin_init(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=cell_id(g);var c=src[i];if c.tags.x>=2u {c.routing.z=select(NONE,i+2u,c.hydro.x>c.terrain.x+SECONDARY_LAKE_MIN_DEPRESSION_M);}dst[i]=c;
}
@compute @workgroup_size(TERRAIN_WORKGROUP_EDGE,TERRAIN_WORKGROUP_EDGE)
fn basin_relax(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=cell_id(g);var c=src[i];let old=c.routing.z;
 if c.tags.x>=2u && c.hydro.x>c.terrain.x+SECONDARY_LAKE_MIN_DEPRESSION_M {for(var k=0u;k<4u;k++) {let b=src[neighbor(i,k)];if b.tags.x>=2u && b.hydro.x>b.terrain.x+.05 && abs(b.hydro.x-c.hydro.x)<.01 {c.routing.z=min(c.routing.z,b.routing.z);}}}
 // Follow the representative's representative to accelerate long connected basins.
 if c.routing.z>=2u && c.routing.z!=NONE {c.routing.z=min(c.routing.z,src[c.routing.z-2u].routing.z);}
 if old!=c.routing.z {atomicAdd(&flags.changed,1u);}dst[i]=c;
}
@compute @workgroup_size(TERRAIN_WORKGROUP_EDGE,TERRAIN_WORKGROUP_EDGE)
fn climate(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=cell_id(g);var c=src[i];let d=pos(i);let season=p.physical.w;
 let latitude=asin(d.y);let solar=cos(latitude-p.physical.y*PI/180.*sin(season*2.*PI));
 let ocean=c.tags.x<2u || c.water.x>CLIMATE_WET_DEPTH_M;let temp=clamp(CLIMATE_BASE_TEMPERATURE_C+CLIMATE_SOLAR_WARMING_C*solar-max(c.terrain.x,0.)*CLIMATE_LAPSE_C_PER_M,MIN_CLIMATE_TEMPERATURE_C,MAX_CLIMATE_TEMPERATURE_C);
 c.climate.x=mix(c.climate.x,temp,select(LAND_TEMPERATURE_RESPONSE,WATER_TEMPERATURE_RESPONSE,ocean));
 let east=normalize(vec3(-d.z,0.,d.x)+vec3(WIND_AXIS_EPSILON,0.,0.));let signwind=select(-1.,1.,abs(d.y)>WIND_REVERSAL_Y);
 let upstream=index(normalize(d-east*signwind*2./f32(p.dims.x)));
 let b=src[upstream];let uplift=max(c.terrain.x-b.terrain.x,0.);
 let convection=CONVECTION_BASE+CONVECTION_SOLAR_GAIN*pow(max(0.,solar),CONVECTION_SOLAR_EXPONENT);let vapor=mix(c.climate.z,b.climate.z,UPSTREAM_VAPOR_SHARE);
 let recycling=LAND_VAPOR_BASE+c.life.x*VEGETATION_VAPOR_GAIN+GROUNDWATER_VAPOR_GAIN*c.water.y/(c.water.y+GROUNDWATER_VAPOR_HALF_SATURATION_M);
 let supply=select(recycling,WATER_VAPOR_BASE+max(temp,0.)*WATER_VAPOR_TEMPERATURE_GAIN,ocean);
 let precip=clamp(vapor*(convection+uplift*OROGRAPHIC_PRECIPITATION_PER_M),0.,MAX_PRECIPITATION_PER_PASS);
 c.climate.z=clamp(vapor+supply-precip,0.,MAX_VAPOR_STORAGE);
 c.climate.y=precip*PRECIPITATION_ANNUALIZATION;c.climate.w=signwind*(PREVAILING_WIND_BASE_M_S+PREVAILING_WIND_AMPLITUDE_M_S*abs(sin(latitude*WIND_LATITUDE_FREQUENCY)))+SEASONAL_WIND_AMPLITUDE_M_S*sin(season*SEASONAL_WIND_RADIANS+f32(p.dims.z)*WIND_EPOCH_PHASE_RADIANS);
 let weight=1./f32(p.aux.y+1u);c.hydro.y=mix(c.hydro.y,c.climate.x,weight);c.hydro.z=mix(c.hydro.z,c.climate.y,weight);
 dst[i]=c;
}
@compute @workgroup_size(TERRAIN_WORKGROUP_EDGE,TERRAIN_WORKGROUP_EDGE)
fn flow_init(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=cell_id(g);var c=src[i];let permeability=catalog[c.ids.x].a.y;
 let precipitation=c.hydro.z*METERS_PER_MM;let rain=select(0.,precipitation,c.hydro.y>=0.);
 let melt=min(c.water.z,max(c.hydro.y,0.)*SNOW_MELT_M_PER_C);let available=rain+melt;
 let evap=min(available,max(c.hydro.y+EVAPORATION_TEMPERATURE_OFFSET_C,0.)*EVAPORATION_M_PER_C);
 let infiltrate=min(max(0.,GROUNDWATER_CAPACITY_M-c.water.y),(available-evap)*permeability*INFILTRATION_PERMEABILITY_SHARE);
 let release=c.water.y*GROUNDWATER_RELEASE_SHARE;
 c.life.w=max(0.,available-evap-infiltrate)+release;
 if c.tags.x<2u {c.life.w=0.;}
 let retention=select(max(0.,c.hydro.x-c.terrain.x-c.water.x)*area(i)/SECONDS_PER_RUNOFF_YEAR,0.,c.tags.x<2u);
 c.water.w=max(0.,c.life.w*area(i)/SECONDS_PER_RUNOFF_YEAR-retention);c.hydro.w=infiltrate-release;
 c.budget.x=precipitation;c.budget.y=evap;dst[i]=c;
}
@compute @workgroup_size(TERRAIN_WORKGROUP_EDGE,TERRAIN_WORKGROUP_EDGE)
fn flow_relax(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=cell_id(g);var c=src[i];var total=c.life.w*area(i)/SECONDS_PER_RUNOFF_YEAR;
 for(var k=0u;k<4u;k++){let j=neighbor(i,k);if src[j].routing.x==i {total+=src[j].water.w;}}
 let retention=select(max(0.,c.hydro.x-c.terrain.x-c.water.x)*area(i)/SECONDS_PER_RUNOFF_YEAR,0.,c.tags.x<2u);total=max(0.,total-retention);
 if total!=c.water.w {atomicAdd(&flags.changed,1u);}c.water.w=total;dst[i]=c;
}
fn erosion_flux(a:Cell,b:Cell,i:u32,j:u32)->f32 {
 let distance=max(MIN_EROSION_DISTANCE_M,acos(clamp(dot(pos(i),pos(j)),-1.,1.))*p.physical.x*METERS_PER_KM);
 let slope=(a.terrain.x-b.terrain.x)/distance;
 return max(0.,slope-THERMAL_EROSION_MIN_SLOPE)*min(a.terrain.y,THERMAL_MOBILE_SEDIMENT_CAP_M)*THERMAL_EROSION_RATE*(1.-a.life.x*EROSION_VEGETATION_PROTECTION);
}
fn fluvial_flux(i:u32)->f32 {
 let c=src[i];if c.routing.x==NONE || c.water.x>FLUVIAL_STANDING_WATER_CUTOFF_M || c.tags.x<2u {return 0.;}
 let b=src[c.routing.x];let slope=max(0.,c.terrain.x-b.terrain.x)/max(MIN_EROSION_DISTANCE_M,acos(clamp(dot(pos(i),pos(c.routing.x)),-1.,1.))*p.physical.x*METERS_PER_KM);
 let rate=min(MAX_FLUVIAL_EROSION_M,sqrt(max(0.,c.water.w))*FLUVIAL_DISCHARGE_EROSION_GAIN*slope)*(1.-c.life.x*EROSION_VEGETATION_PROTECTION)/max(catalog[c.ids.x].a.x,MIN_EROSION_ROCK_HARDNESS);
 var available=max(0.,c.terrain.x-MIN_CONSTRAINED_LAND_ELEVATION_M);if column_present(c) {available=min(available,c.terrain.y+dot(c.strata.xyz,vec3(1.)));}
 return min(rate,available);
}
@compute @workgroup_size(TERRAIN_WORKGROUP_EDGE,TERRAIN_WORKGROUP_EDGE)
fn water_erosion(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=cell_id(g);var c=src[i];let rock=catalog[c.ids.x];let annual=max(c.hydro.z*METERS_PER_MM,0.);
 let cold=c.hydro.y<0.;let snowfall=select(0.,c.budget.x,cold);let melt=min(c.water.z,max(c.hydro.y,0.)*SNOW_MELT_M_PER_C);
 c.water.z=max(0.,c.water.z+snowfall-melt);c.water.y=clamp(c.water.y+c.hydro.w,0.,GROUNDWATER_CAPACITY_M);
 let capacity=max(0.,c.hydro.x-c.terrain.x);
 // Stored runoff fills secondary depressions; discharge includes the full catchment.
 var incoming=c.life.w*area(i)/SECONDS_PER_RUNOFF_YEAR;
 for(var k=0u;k<4u;k++){let j=neighbor(i,k);if src[j].routing.x==i {incoming+=src[j].water.w;}}
 let inflow=incoming*SECONDS_PER_RUNOFF_YEAR/area(i);
 c.water.x=clamp(c.water.x+inflow,0.,capacity);
 if c.tags.x<2u {c.water.x=max(0.,select(0.,planet[0].x,c.tags.x==1u)-c.terrain.x);}
 var weather=rock.a.z*BEDROCK_WEATHERING_GAIN*(1.+annual)*(1.-select(0.,FROZEN_WEATHERING_REDUCTION,cold));
 if column_present(c) {weather=min(weather,dot(c.strata.xyz,vec3(1.)));c=cut_column(c,weather);}
 c.terrain.z=clamp(c.terrain.z+weather,0.,MAX_WEATHERED_SOIL_DEPTH_M);c.terrain.y+=weather;
 var lost=0.;var gained=0.;
 for(var k=0u;k<4u;k++){let j=neighbor(i,k);lost+=erosion_flux(src[i],src[j],i,j);gained+=erosion_flux(src[j],src[i],j,i)*area(j)/area(i);}
 let erode=fluvial_flux(i);var deposit=0.;
 for(var k=0u;k<4u;k++){let j=neighbor(i,k);if src[j].routing.x==i {deposit+=fluvial_flux(j)*area(j)/area(i);}}
 let bedrock_cut=max(0.,erode-max(0.,c.terrain.y-lost));
 if column_present(c) {c=cut_column(c,bedrock_cut);}
 c.terrain.y=max(0.,c.terrain.y-lost+gained-min(erode,c.terrain.y)+deposit);
 // Bedrock weathering becomes local sediment; rivers and slopes transport material between cells.
 c.terrain.x=constrain(c.terrain.x-lost+gained-erode+deposit,c.tags.x);
 c.budget.z=lost+erode+weather;c.budget.w=gained+deposit+weather;
 if column_present(c) {
  let lithified=min(c.terrain.y,max(0.,c.terrain.y-LITHIFICATION_SEDIMENT_RESERVE_M)*min(MAX_LITHIFICATION_SHARE,p.physical.z*LITHIFICATION_SHARE_PER_MYR));
  var rock_id=province_rock(1u,pos(i),fbm(pos(i)*SEDIMENTARY_PROVINCE_NOISE_FREQUENCY)+SEDIMENTARY_PROVINCE_NOISE_OFFSET);if p.tuning.w>1. {rock_id=setting_rock(select(4u,3u,c.tags.x<2u),pos(i));}
  c=lithify_column(c,lithified,rock_id);
 }
 if c.tags.x==1u {c.life.z=planet[0].y;}
 dst[i]=c;
}
@compute @workgroup_size(TERRAIN_WORKGROUP_EDGE,TERRAIN_WORKGROUP_EDGE)
fn ecology(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=cell_id(g);var c=src[i];let f=catalog[c.ids.x].ids.x;
 let soil_offset=p.counts.x+p.counts.y;var best=-1.;var soil=0u;
 for(var j=0u;j<p.counts.z;j++){let e=catalog[soil_offset+j];let score=select(.1,1.,e.ids.x==f)+rand(i+j*37u)*.2;if score>best {best=score;soil=j;}}
 c.ids.y=soil;let soilentry=catalog[soil_offset+soil];
 let biome_offset=soil_offset+p.counts.z+p.counts.w;var biome=NONE;var density=0.;
 for(var j=0u;j<p.aux.x;j++){let b=catalog[biome_offset+j];if b.ids.x!=0u {continue;}if c.hydro.y>=b.a.x && c.hydro.y<b.a.y && c.hydro.z>=b.a.z && c.hydro.z<b.a.w && c.terrain.x>=b.b.x && c.terrain.x<b.b.y {biome=j;density=b.b.z;break;}}
 c.ids.w=biome;var plant=NONE;best=-1.;var growth=.03;
 for(var j=0u;j<p.counts.w;j++){let e=catalog[soil_offset+p.counts.z+j];
  if c.hydro.y<e.a.x || c.hydro.y>e.a.y || c.hydro.z<e.a.z || c.hydro.z>e.a.w || c.life.y<e.b.x || (e.ids.y!=3u && e.ids.y!=f) {continue;}
  if e.ids.x==1u && c.tags.x!=3u {continue;}
  if u32(e.c.x)>2u {continue;}
  let score=rand(i+j*199u);if score>best {best=score;plant=j;growth=e.b.y;}
 }
 c.ids.z=plant;let disturbance=select(0.,.05,rand(i+p.dims.z*91u)<.015);
 // Cover and fertility are maintained by ecological inventories.
 if plant==NONE || c.tags.x<2u || c.water.x>1. {c.life.x=0.;c.ids.z=NONE;}
 var mineral=NONE;var potential=0.;c.geology.w=0.;
 for(var j=0u;j<p.counts.y;j++){let e=catalog[p.counts.x+j];if (e.ids.x&(1u<<c.ids.x))==0u {continue;}
 var score=e.a.x*rand(i/7u+j*43u)*(1.+c.geology.x*.5);
 if p.tuning.w>0.&&e.ids.z!=0u {score=deposit_potential(c,pos(i),e);}
 if score>potential {potential=score;mineral=j;c.geology.w=e.a.z+max(0.,c.terrain.y);}}
 c.tags.y=mineral;c.geology.z=potential;dst[i]=c;
}

// Deterministic tree reduction: no floating-point atomic accumulation.
@compute @workgroup_size(TERRAIN_WORKGROUP_EDGE,TERRAIN_WORKGROUP_EDGE)
fn lake_collect(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=cell_id(g);let c=src[i];var value=vec4(0.);
 if c.tags.x==1u {
  let a=area(i)*LAKE_LEDGER_SCALE;let river=c.water.w*SECONDS_PER_RUNOFF_YEAR*LAKE_LEDGER_SCALE;
  let evaporation=max(c.hydro.y+EVAPORATION_TEMPERATURE_OFFSET_C,0.)*EVAPORATION_M_PER_C;
  let capacity=max(0.,planet[0].x-c.terrain.x);
  value=vec4(a,(c.budget.x-evaporation+c.water.x-capacity)*a+river,capacity*a,planet[0].y*c.water.x*a+river*LAKE_RIVER_INPUT_SALINITY);
 }
 scratch[i]=value;dst[i]=c;
}
var<workgroup> sums:array<vec4<f32>,LAKE_REDUCTION_WORKGROUP_SIZE>;
@compute @workgroup_size(LAKE_REDUCTION_WORKGROUP_SIZE)
fn lake_reduce(@builtin(global_invocation_id) g:vec3<u32>,@builtin(local_invocation_index) local:u32,@builtin(workgroup_id) group:vec3<u32>) {
 let total=6u*p.dims.x*p.dims.x;let input_offset=p.aux.w*total;let output_offset=(1u-p.aux.w)*total;
 var value=vec4(0.);if g.x<p.aux.z {value=scratch[input_offset+g.x];}sums[local]=value;workgroupBarrier();
 var stride=LAKE_REDUCTION_WORKGROUP_SIZE/2u;loop {if local<stride {sums[local]+=sums[local+stride];}workgroupBarrier();if stride==1u {break;}stride/=2u;}
 if local==0u {scratch[output_offset+group.x]=sums[0];}
}
@compute @workgroup_size(TERRAIN_WORKGROUP_EDGE,TERRAIN_WORKGROUP_EDGE)
fn lake_update(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=cell_id(g);dst[i]=src[i];if i!=0u{return;}
 let total=6u*p.dims.x*p.dims.x;let t=scratch[p.aux.w*total];let old=planet[0];
 let raw=old.x+t.y/max(t.x,LAKE_DIVISION_FLOOR);let level=clamp(raw,MIN_GREAT_LAKE_LEVEL_M,MAX_GREAT_LAKE_LEVEL_M);
 let volume=max(LAKE_DIVISION_FLOOR,t.z+(level-old.x)*t.x);
 let salinity=clamp(t.w/volume,0.,MAX_GREAT_LAKE_SALINITY);
 planet[0]=vec4(level,salinity,old.z+(raw-level)*t.x,volume);
}

// Remove one-cell coastal slivers introduced by sampling continuous masks.
@compute @workgroup_size(TERRAIN_WORKGROUP_EDGE,TERRAIN_WORKGROUP_EDGE)
fn coast_cleanup(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=cell_id(g);var c=src[i];var votes=array<u32,4>(0u,0u,0u,0u);
 for(var k=0u;k<4u;k++){votes[src[neighbor(i,k)].tags.x]+=1u;}
 if votes[c.tags.x]<=1u {
  for(var r=0u;r<4u;r++){if votes[r]>=3u {
   c.tags.x=r;c.terrain.x=constrain(c.terrain.x,r);
   c.water.x=select(0.,max(0.,select(0.,120.,r==1u)-c.terrain.x),r<2u);
   c.life.z=select(select(0.,.05,r==1u),35.,r==0u);
  }}
 }
 dst[i]=c;
}

@compute @workgroup_size(TERRAIN_WORKGROUP_EDGE,TERRAIN_WORKGROUP_EDGE)
fn climate_start(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=cell_id(g);let c=src[i];scratch[i]=vec4(c.hydro.yz,c.climate.xz);dst[i]=c;
}
@compute @workgroup_size(TERRAIN_WORKGROUP_EDGE,TERRAIN_WORKGROUP_EDGE)
fn climate_check(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=cell_id(g);let c=src[i];let previous=scratch[i];
 // Compare both annual means and the same seasonal phase. Stable means alone
 // can conceal a moisture reservoir that is still warming up.
 if abs(c.hydro.y-previous.x)>CLIMATE_TEMPERATURE_TOLERANCE_C || abs(c.hydro.z-previous.y)>CLIMATE_RAIN_TOLERANCE_MM_YEAR || abs(c.climate.x-previous.z)>CLIMATE_TEMPERATURE_TOLERANCE_C || abs(c.climate.z-previous.w)>CLIMATE_VAPOR_TOLERANCE {atomicAdd(&flags.changed,1u);}
 dst[i]=c;
}
fn secondary(c:Cell)->bool { return c.tags.x>=2u && c.routing.z>=2u && c.routing.z!=NONE && c.hydro.x>c.terrain.x+SECONDARY_LAKE_MIN_DEPRESSION_M; }
fn lake_weight(i:u32)->f32 { return area(i)/(4.*PI*p.physical.x*p.physical.x*1e6/f32(6u*p.dims.x*p.dims.x))*POOL_AREA_WEIGHT_SCALE; }
// Reuse the drainage/reduction scratch: water depth, terrain, area weight, spill.
// Static geometry is prepared once; relaxation never copies the full Cell.
@compute @workgroup_size(TERRAIN_WORKGROUP_EDGE,TERRAIN_WORKGROUP_EDGE)
fn pool_init(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=cell_id(g);let c=src[i];scratch[i]=vec4(c.water.x,c.terrain.x,lake_weight(i),c.hydro.x);
}
// Frozen edge fluxes conserve area-weighted water. Four outflows consume at most
// 96% of the source inventory. Dry saddles remain physical barriers.
fn pool_transfer(i:u32,j:u32,offset:u32)->f32 {
 let a=scratch[offset+i];let b=scratch[offset+j];if a.x<=0.||src[i].tags.x<2u{return 0.;}
 let same_pool=secondary(src[i])&&secondary(src[j])&&src[i].routing.z==src[j].routing.z;
 let overflow=src[i].routing.x==j && a.y+a.x>a.w+POOL_OVERFLOW_TOLERANCE_M;
 if !same_pool&&!overflow{return 0.;}
 var head=a.y+a.x-max(a.y,b.y+b.x);
 if !same_pool {head=min(head,a.y+a.x-a.w);}
 if head<=max(POOL_HEAD_ABSOLUTE_TOLERANCE_M,max(abs(a.y),abs(b.y))*POOL_HEAD_RELATIVE_TOLERANCE){return 0.;}
 return min(a.x*a.z*POOL_MAX_EDGE_OUTFLOW_SHARE,head*a.z*b.z/(a.z+b.z)*POOL_HEAD_TRANSFER_SHARE);
}
fn pool_compact(i:u32,side:u32) {
 let count=6u*p.dims.x*p.dims.x;let offset=side*count;
 let old=scratch[offset+i];var c=old;var net=0.;
 for(var k=0u;k<4u;k++){let j=neighbor(i,k);net+=pool_transfer(j,i,offset)-pool_transfer(i,j,offset);}
 c.x=max(0.,c.x+net/c.z);
 let change=abs(c.x-old.x);
 if change>max(POOL_HEAD_ABSOLUTE_TOLERANCE_M,max(abs(c.y),abs(c.x))*POOL_HEAD_RELATIVE_TOLERANCE) {
  atomicAdd(&flags.changed,1u);atomicMax(&flags.invalid,bitcast<u32>(change));
 }
 scratch[(1u-side)*count+i]=c;
}
@compute @workgroup_size(TERRAIN_WORKGROUP_EDGE,TERRAIN_WORKGROUP_EDGE)
fn pool_even(@builtin(global_invocation_id) g:vec3<u32>) {pool_compact(cell_id(g),0u);}
@compute @workgroup_size(TERRAIN_WORKGROUP_EDGE,TERRAIN_WORKGROUP_EDGE)
fn pool_odd(@builtin(global_invocation_id) g:vec3<u32>) {pool_compact(cell_id(g),1u);}
@compute @workgroup_size(TERRAIN_WORKGROUP_EDGE,TERRAIN_WORKGROUP_EDGE)
fn pool_scatter(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=cell_id(g);var c=src[i];c.water.x=scratch[i].x;dst[i]=c;
}
