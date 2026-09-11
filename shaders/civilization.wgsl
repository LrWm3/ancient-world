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
@compute @workgroup_size(64)
fn survey(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=g.x+g.y*65535u*64u;if i>=p.dims.x{return;}let c=world[i];
 let warmth=clamp((c.hydro.y+5.)/20.,0.,1.)*clamp((45.-c.hydro.y)/15.,0.,1.);
 let moisture=clamp(c.hydro.z/800.,0.,1.)*clamp((5000.-c.hydro.z)/2000.,0.,1.);
 let soil=clamp(c.life.y,.1,1.);let dry=c.tags.x==2u&&flood_depth(c)<.25&&c.terrain.x<3000.;
 let harvest=select(0.,2200.*warmth*moisture*soil,dry);
 // Reserve a bounded, non-overlapping farm footprint within this world cell.
 let score=harvest*(1.+min(c.water.y,.5)) /(1.+max(0.,c.terrain.x-800.)*.001);
 prospects[i]=vec4(harvest,score,c.hydro.y,f32(c.tags.x));
}
fn flood_depth(c:Cell)->f32 {return c.water.x/select(1.,.05,c.routing.x!=0xffffffffu&&c.water.w>1.&&c.hydro.x-c.terrain.x<.01);}
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
@compute @workgroup_size(64)
fn month(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=g.x+g.y*65535u*64u;if i>=p.dims.y{return;}var s=src[i];if p.options.x==2u {economies[i].production_probe=vec4(0.);weather_storage(i);}if s.stock.x<=0. {s.stock.z=0.;dst[i]=s;return;}
 var weather=(.7+.6*f32(hash(p.dims.z^p.dims.w^u32(s.habitat.w)*7919u)&65535u)/65535.)*regional_weather(u32(s.habitat.z));
 if (p.options.w&2u)!=0u {let c=world[u32(s.habitat.z)];weather=clamp(c.climate.y/max(c.hydro.z,.001),0.,10.);}
 var cultivated=min(s.habitat.y,select(s.stock.x*select(.45,.465,p.options.x==2u),workers(i,s.stock.x)*worker_shares(economies[i],s.stock.x,workers(i,s.stock.x)*(1.-.4*select(0.,clamp(economies[i].soil.w,0.,1.),(p.options.w&2u)!=0u))).x*1.5,(p.options.w&1u)==1u)); // ha, capped by available labor
 if p.options.x==2u && economies[i].farm_workers.x>.5 {
  cultivated=min(cultivated,max(0.,economies[i].farm_workers.y)*1.5);
  economies[i].farm_workers.z=cultivated/1.5;
 }
 if p.options.x==2u {economies[i].production_probe.y=cultivated;}
 var produced=cultivated*s.habitat.x/12.*select(weather,min(weather,1.25),(p.options.w&2u)!=0u);
 if (p.options.w&2u)!=0u {let c=world[u32(s.habitat.z)];produced*=clamp((c.climate.x+5.)/20.,0.,1.)*clamp((45.-c.climate.x)/15.,0.,1.)*clamp(1.-flood_depth(c),0.,1.); }
 if (p.options.w&1u)==1u {produced*=demography[i].crops.w*(1.+.6*cos((f32(p.dims.z%12u)-demography[i].crops.z+2.)*.5235988));}
 if economies[i].management.x>.5 {
  let t=world[u32(s.habitat.z)];
  let annual_warmth=clamp((t.hydro.y+5.)/20.,0.,1.)*clamp((45.-t.hydro.y)/15.,0.,1.);
  let annual_moisture=clamp(t.hydro.z/800.,0.,1.)*clamp((5000.-t.hydro.z)/2000.,0.,1.);
  // Survey yield already embeds generic wheat climate. Recover its soil-limited
  // potential so crop-specific climate is applied once in managed_production.
  produced=cultivated*s.habitat.x/max(.1,annual_warmth*annual_moisture)/12.;
  if (p.options.w&2u)!=0u {produced*=clamp(1.-flood_depth(t),0.,1.);}
 }
 if p.options.x==2u {produced=ecological_production(i,produced*bitcast<f32>(p.options.z),weather);}
 let growth=produced;
 if (p.options.w&1u)==1u && economies[i].management.x<.5 {produced=crop_calendar(i,produced);}
 let storage=select(1.,1.-.5*clamp(container_service(economies[i])/max(1.,s.stock.x*2.),0.,1.),p.options.x==2u);
 // Basic granaries hold one harvest year; manufactured storage adds up to
 // another year. Unprotected overflow spoils after this month's consumption.
 let capacity=s.stock.x*18.*(12.+24.*(1.-storage));
 let ordinary_spoilage=s.stock.y*.01*storage;
 let available=s.stock.y-ordinary_spoilage+produced;
 let need=select(s.stock.x*18.,dot(demography[i].ages.xyz,vec3(10.,18.,14.)),(p.options.w&1u)==1u);let entitlement=select(need,min(need,demography[i].household_food.x),demography[i].household_food.y>0.5);let eaten=min(available,entitlement);let shortage=clamp(1.-eaten/max(need,.01),0.,1.);
 if (p.options.w&1u)==1u {demography[i].household_food.z=max(0.,available);}
 let overflow=max(0.,available-eaten-capacity);
 let spoilage=ordinary_spoilage+overflow;
 if p.options.x==2u {return_food(i,eaten+ordinary_spoilage);var e=economies[i];e.detritus+=vec4(overflow*vec3(.45,.02,.003),0.);economies[i]=e;}
 var births=s.stock.x*.0025*(1.-shortage);var deaths=s.stock.x*(.0012+shortage*.04);
 if (p.options.w&1u)==1u {let change=demographic_month(i,shortage,eaten);births=change.x;deaths=change.y;}
 s.stock.x=max(0.,s.stock.x+births-deaths);
 // Cohorts are authoritative; separately integrating their sum accumulates drift after collapse.
 if (p.options.w&1u)==1u {s.stock.x=dot(demography[i].ages.xyz,vec3(1.));}
 s.stock.y=max(0.,available-eaten-overflow);
 s.stock.z=produced;s.stock.w=shortage;
 s.ledger+=vec4(growth,eaten,spoilage,0.);s.people+=vec4(births,deaths,0.,0.);dst[i]=s;
}
