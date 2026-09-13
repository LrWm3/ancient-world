// Display scales and thresholds; these do not change simulation inventories.
const PI:f32=3.14159265359;
const PALETTE_MIDPOINT:f32=.48;
const PALETTE_AMPLITUDE:f32=.38;
const PALETTE_CYCLE_RADIANS:f32=6.28;
const CATEGORY_PALETTE_STRIDE:f32=.618;
const NATURAL_WATER_DEPTH_SCALE_M:f32=2500.;
const MAP_WET_DEPTH_M:f32=.25;
const BIOME_BASE_COLOR_WEIGHT:f32=.4;
const BIOME_COVER_COLOR_WEIGHT:f32=.6;
const BARE_ROCK_START_M:f32=1800.;
const BARE_ROCK_FULL_M:f32=6000.;
const VISIBLE_SNOW_DEPTH_M:f32=.1;
const VISIBLE_RIVER_DISCHARGE_M3_S:f32=2500.;
const RIVER_COLOR_WEIGHT:f32=.75;
const BATHYMETRY_OFFSET_M:f32=4000.;
const BATHYMETRY_RANGE_M:f32=4120.;
const ELEVATION_COLOR_SCALE_M:f32=6000.;
const RESOURCE_COLOR_GAIN:f32=2.;
const TEMPERATURE_COLOR_OFFSET_C:f32=30.;
const TEMPERATURE_COLOR_RANGE_C:f32=75.;
const PRECIPITATION_COLOR_SCALE_MM_YEAR:f32=3000.;
const WIND_COLOR_SCALE_M_S:f32=35.;
const SNOW_COLOR_SCALE_M:f32=2.;
const DRAINAGE_RANK_COLOR_SCALE:f32=300.;
const RIVER_LOG_COLOR_SCALE:f32=10.;
const LAKE_LOG_COLOR_SCALE:f32=6.;
const SOLAR_PRODUCTION_COLOR_GAIN:f32=10.;
const CHEMO_PRODUCTION_COLOR_GAIN:f32=100.;
const NITROGEN_COLOR_GAIN:f32=50.;
const PHOSPHORUS_COLOR_GAIN:f32=250.;
const PLANT_BIOMASS_COLOR_GAIN:f32=10.;
const ANIMAL_BIOMASS_COLOR_GAIN:f32=100.;
const CURRENT_DIRECTION_RADIANS:f32=6.283185;
const UPWELLING_COLOR_GAIN:f32=5.;
const DEEP_NUTRIENT_COLOR_GAIN:f32=10.;
const PLANKTON_COLOR_GAIN:f32=100.;
const ATMOSPHERE_GLOW_FALLOFF:f32=18.;
const ATMOSPHERE_GLOW_GAIN:f32=.12;
const GLOBE_AMBIENT_LIGHT:f32=.5;
const GLOBE_DIRECT_LIGHT:f32=.5;
const GLOBE_RIM_EXPONENT:f32=4.;
const REGIONAL_BLEND_START_ZOOM:f32=2.;
const REGIONAL_BLEND_FULL_ZOOM:f32=4.;
const GLOBAL_RELIEF_CONTRAST_PER_M:f32=.0004;
const REGIONAL_RELIEF_CONTRAST_PER_M:f32=.003;
const RELIEF_SAMPLE_MAX_ZOOM:f32=16.;
const MIN_RELIEF_SHADE:f32=.65;
const MAX_RELIEF_SHADE:f32=1.25;
const RIM_COLOR_WEIGHT:f32=.3;
const SELECTION_COLOR_WEIGHT:f32=.75;

struct View { dims:vec4<u32>, camera:vec4<f32>, pan:vec4<f32>, counts:vec4<u32>, extra:vec4<u32> }
struct Entry { a:vec4<f32>,b:vec4<f32>,c:vec4<f32>,d:vec4<f32>,ids:vec4<u32> }
@group(0) @binding(0) var<storage,read> cells:array<Cell>;
@group(0) @binding(1) var<storage,read> catalog:array<Entry>;
@group(0) @binding(2) var<uniform> v:View;
@group(0) @binding(3) var output:texture_storage_2d<rgba8unorm,write>;
struct Eco { pools:array<vec4<f32>,41> }
@group(0) @binding(4) var<storage,read> ecology:array<Eco>;
fn index(d:vec3<f32>)->u32 {
 let a=abs(d);var f=0u;var uv=vec2(0.);
 if a.x>=a.y && a.x>=a.z {f=select(1u,0u,d.x>=0.);uv=d.yz/a.x;}else if a.y>=a.z {f=select(3u,2u,d.y>=0.);uv=d.xz/a.y;}else{f=select(5u,4u,d.z>=0.);uv=d.xy/a.z;}
 let xy=vec2<u32>(clamp(floor((uv+1.)*.5*f32(v.dims.x)),vec2(0.),vec2(f32(v.dims.x-1u))));return f*v.dims.x*v.dims.x+xy.y*v.dims.x+xy.x;
}
fn palette(x:f32)->vec3<f32> {return PALETTE_MIDPOINT+PALETTE_AMPLITUDE*cos(vec3(0.,2.,4.)+x*PALETTE_CYCLE_RADIANS);}
fn terrain(c:Cell)->vec3<f32> {
 if c.tags.x<2u {return mix(vec3(.025,.10,.18),vec3(.08,.36,.43),clamp(1.-c.water.x/NATURAL_WATER_DEPTH_SCALE_M,0.,1.));}
 if c.water.x>MAP_WET_DEPTH_M {return vec3(.08,.36,.52);}
 let bio=v.counts.x+v.counts.y+v.counts.z+v.counts.w;
 var color=mix(vec3(.49,.43,.28),vec3(.12,.32,.19),c.life.x);
 if c.ids.w<v.extra.x {color=mix(vec3(.49,.43,.28),catalog[bio+c.ids.w].c.xyz,BIOME_BASE_COLOR_WEIGHT+c.life.x*BIOME_COVER_COLOR_WEIGHT);}
 color=mix(color,vec3(.62,.59,.52),smoothstep(BARE_ROCK_START_M,BARE_ROCK_FULL_M,c.terrain.x));
 if c.water.z>VISIBLE_SNOW_DEPTH_M {color=mix(color,vec3(.89,.94,.95),min(c.water.z,1.));}
 if c.water.w>VISIBLE_RIVER_DISCHARGE_M3_S {color=mix(color,vec3(.15,.55,.68),RIVER_COLOR_WEIGHT);}
 return color;
}
fn color_for(c:Cell)->vec3<f32> {
 let water=c.tags.x<2u;let layer=v.dims.w;
 switch layer {
  case 0u:{return terrain(c);}
  case 1u:{if water{return mix(vec3(.02,.06,.2),vec3(.1,.5,.62),clamp((c.terrain.x+BATHYMETRY_OFFSET_M)/BATHYMETRY_RANGE_M,0.,1.));}return mix(vec3(.2,.4,.22),vec3(.95,.9,.77),clamp(c.terrain.x/ELEVATION_COLOR_SCALE_M,0.,1.));}
  case 2u:{return palette(f32(c.routing.w)*CATEGORY_PALETTE_STRIDE);}
  case 3u:{if water || c.water.x>MAP_WET_DEPTH_M {return terrain(c); }return catalog[c.ids.x].c.xyz;}
  case 4u:{return mix(vec3(.035,.045,.065),vec3(.95,.65,.2),clamp(c.geology.z*RESOURCE_COLOR_GAIN,0.,1.));}
  case 5u:{return palette(f32(c.ids.y)*CATEGORY_PALETTE_STRIDE);}
  case 6u:{return mix(vec3(.16,.35,.8),vec3(.98,.3,.12),clamp((c.hydro.y+TEMPERATURE_COLOR_OFFSET_C)/TEMPERATURE_COLOR_RANGE_C,0.,1.));}
  case 7u:{return mix(vec3(.68,.42,.2),vec3(.13,.65,.8),clamp(c.hydro.z/PRECIPITATION_COLOR_SCALE_MM_YEAR,0.,1.));}
  case 8u:{return mix(vec3(.2,.45,.8),vec3(.9,.55,.25),clamp(c.climate.w/WIND_COLOR_SCALE_M_S+.5,0.,1.));}
  case 9u:{return mix(vec3(.08,.12,.16),vec3(.92,.97,1.),clamp(c.water.z/SNOW_COLOR_SCALE_M,0.,1.));}
  case 10u:{if c.routing.z!=0xffffffffu {return palette(f32(c.routing.z)*CATEGORY_PALETTE_STRIDE);}return mix(vec3(.08,.12,.18),vec3(.8,.72,.45),clamp(f32(c.routing.y)/DRAINAGE_RANK_COLOR_SCALE,0.,1.));}
  case 11u:{if water{return vec3(.025,.12,.2);}return mix(vec3(.12,.16,.14),vec3(.22,.8,1.),clamp(log(1.+c.water.w)/RIVER_LOG_COLOR_SCALE,0.,1.));}
  case 12u:{return mix(vec3(.14,.18,.16),vec3(.15,.65,.9),clamp(log(1.+c.water.x)/LAKE_LOG_COLOR_SCALE,0.,1.));}
  case 13u:{if water{return vec3(.025,.12,.2);}if c.ids.w<v.extra.x{return catalog[v.counts.x+v.counts.y+v.counts.z+v.counts.w+c.ids.w].c.xyz;}return vec3(.5);}
  default:{return mix(vec3(.4,.3,.2),vec3(.1,.65,.28),c.life.x);}
 }
}
fn eco_color(id:u32,layer:u32)->vec3<f32> {
 let n=v.dims.x;let m=v.extra.z;let r=n/m;let j=id/(n*n)*m*m+(id/n%n)/r*m+(id%n)/r;let e=ecology[j];var value=0.;
 switch layer {
 case 15u:{value=e.pools[28].x*SOLAR_PRODUCTION_COLOR_GAIN;}case 16u:{value=e.pools[28].y*CHEMO_PRODUCTION_COLOR_GAIN;}
 case 17u:{value=e.pools[17].y*NITROGEN_COLOR_GAIN;}case 18u:{value=e.pools[17].z*PHOSPHORUS_COLOR_GAIN;}
 case 19u:{return select(select(vec3(.2,.6,.3),vec3(.9,.5,.1),e.pools[29].w==1.),vec3(.7,.2,.8),e.pools[29].w==2.);}
 case 20u,21u,22u,23u,24u:{value=log(1.+e.pools[layer-20u].x*PLANT_BIOMASS_COLOR_GAIN);}
 case 25u:{for(var k=5u;k<17u;k++){value+=e.pools[k].x;}value=log(1.+value*ANIMAL_BIOMASS_COLOR_GAIN);}
 case 26u:{return palette(atan2(e.pools[29].y,e.pools[29].x)/CURRENT_DIRECTION_RADIANS);}
 case 27u:{value=e.pools[29].z*UPWELLING_COLOR_GAIN;}case 28u:{value=e.pools[21].z*DEEP_NUTRIENT_COLOR_GAIN;}
 case 29u:{value=e.pools[23].x*PLANKTON_COLOR_GAIN;}default:{value=e.pools[18].x;}
 }
 return mix(vec3(.025,.04,.08),vec3(.25,.9,.65),clamp(value,0.,1.));
}
@compute @workgroup_size(MAP_WORKGROUP_EDGE,MAP_WORKGROUP_EDGE)
fn render(@builtin(global_invocation_id) g:vec3<u32>) {
 if g.x>=v.dims.y || g.y>=v.dims.z {return;}
 let size=vec2<f32>(v.dims.yz);let uv=(vec2<f32>(g.xy)+.5)/size;
 var d=vec3(0.);var shade=1.;var rim=0.;
 if v.camera.w>.5 {
  let xy=(uv-.5)*2.*vec2(size.x/size.y,1.)/v.camera.z;
  let r=dot(xy,xy);
  if r>1. {let glow=exp(-(sqrt(r)-1.)*ATMOSPHERE_GLOW_FALLOFF)*ATMOSPHERE_GLOW_GAIN;textureStore(output,vec2<i32>(g.xy),vec4(vec3(.023,.036,.055)+vec3(.1,.3,.4)*glow,1.));return;}
  let local=vec3(xy.x,-xy.y,sqrt(1.-r));
  let pitched=vec3(local.x,local.y*cos(v.camera.y)+local.z*sin(v.camera.y),-local.y*sin(v.camera.y)+local.z*cos(v.camera.y));
  d=vec3(pitched.x*cos(v.camera.x)+pitched.z*sin(v.camera.x),pitched.y,-pitched.x*sin(v.camera.x)+pitched.z*cos(v.camera.x));
  shade=GLOBE_AMBIENT_LIGHT+GLOBE_DIRECT_LIGHT*max(0.,dot(local,normalize(vec3(-.4,.5,1.))));rim=pow(1.-local.z,GLOBE_RIM_EXPONENT);
 } else {
  let q=(uv-.5)/v.camera.z+.5+v.pan.xy;let lon=(q.x-.5)*2.*PI;let lat=(.5-q.y)*PI;
  if abs(lat)>PI*.5 {textureStore(output,vec2<i32>(g.xy),vec4(.023,.036,.055,1.));return;}
  d=vec3(cos(lat)*sin(lon),sin(lat),cos(lat)*cos(lon));
 }
 let id=index(d);var c=cells[id];
 if v.camera.z>REGIONAL_BLEND_START_ZOOM && (v.dims.w==0u||v.dims.w==1u) {c.terrain.x=mix(c.terrain.x,regional_height(d),smoothstep(REGIONAL_BLEND_START_ZOOM,REGIONAL_BLEND_FULL_ZOOM,v.camera.z));}
 var color=color_for(c)*shade;
 if v.camera.z>REGIONAL_BLEND_START_ZOOM && (v.dims.w==0u||v.dims.w==1u) {color=mix(color,regional_color(d)*shade,smoothstep(REGIONAL_BLEND_START_ZOOM,REGIONAL_BLEND_FULL_ZOOM,v.camera.z));}
 if v.dims.w>=15u {color=eco_color(id,v.dims.w)*shade;}
 if v.dims.w==0u || v.dims.w==1u || v.dims.w==3u {
  let tangent=normalize(cross(d,vec3(.01,1.,.01)));let other=cells[index(normalize(d+tangent*2./f32(v.dims.x)))];
  var contrast=(c.terrain.x-other.terrain.x)*GLOBAL_RELIEF_CONTRAST_PER_M;
  if v.camera.z>REGIONAL_BLEND_START_ZOOM && v.dims.w!=3u {
   let step=2./f32(v.dims.x)/min(v.camera.z,RELIEF_SAMPLE_MAX_ZOOM);
   contrast=(regional_height(d)-regional_height(normalize(d+tangent*step)))*REGIONAL_RELIEF_CONTRAST_PER_M;
  }
  color*=clamp(1.+contrast,MIN_RELIEF_SHADE,MAX_RELIEF_SHADE);
 }
 color=mix(color,vec3(.24,.55,.7),rim*RIM_COLOR_WEIGHT);
 if id==v.extra.y {color=mix(color,vec3(1.,.83,.36),SELECTION_COLOR_WEIGHT);}
 textureStore(output,vec2<i32>(g.xy),vec4(color,1.));
}
