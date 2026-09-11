struct Economy {
 farm_workers:vec4<f32>, extraction_workers:vec4<f32>, construction_workers:vec4<f32>,
 production_probe:vec4<f32>, food_labor:vec4<f32>,
 tool_craft:vec4<f32>, tool_work:vec4<f32>, tool_orders:array<vec4<f32>,16>,
 residue:vec4<f32>,
 extraction:vec4<f32>,
 return_flow:vec4<f32>, land_return:vec4<f32>,
 targets:array<vec4<f32>,16>, orders:array<vec4<f32>,16>, logistics:vec4<f32>, storage:vec4<f32>, storage_plan:vec4<f32>, housing:vec4<f32>, housing_plan:vec4<f32>, waterworks:vec4<f32>, waterworks_plan:vec4<f32>, water_service:vec4<f32>, workshop:vec4<f32>, workshop_plan:vec4<f32>, workshop_types:array<vec4<f32>,4>,
 enterprise_lease:vec4<f32>, enterprise_plan:vec4<f32>, enterprise_used:vec4<f32>,
 management:vec4<f32>, crops:array<vec4<f32>,6>, herds:array<vec4<f32>,3>, agriculture:vec4<f32>,
 goods:array<vec4<f32>,16>, made:array<vec4<f32>,16>, used:array<vec4<f32>,16>, initial:array<vec4<f32>,16>, prices:array<vec4<f32>,16>,
 soil:vec4<f32>, detritus:vec4<f32>, forest:vec4<f32>, reserves:vec4<f32>, exchange:vec4<f32>, baseline:vec4<f32>, water:vec4<f32>, finance:vec4<f32>, labor:vec4<f32>, claim:vec4<f32>, diagnostics:vec4<f32>, policy:vec4<f32>,
 fishery:vec4<f32>, fishery_plan:vec4<f32>, fishery_config:vec4<f32>, fishery_stats:vec4<f32>, fishery_traps:vec4<f32>, fishery_choice:vec4<f32>, waterworks_recovery:vec4<f32>,
}
struct Eco {pools:array<vec4<f32>,38>}
struct Recipe {input:array<vec4<f32>,16>, output:array<vec4<f32>,16>, work:vec4<f32>}
@group(0) @binding(5) var<storage,read_write> economies:array<Economy>;
@group(0) @binding(6) var<storage,read_write> ecology:array<Eco>;
struct CraftCatalog { goods:array<vec4<f32>,64>, crops:array<vec4<f32>,12>, herds:array<vec4<f32>,3>, seasons:array<vec4<f32>,6>, methods:array<vec4<f32>,6>, recipes:array<Recipe> }
@group(0) @binding(7) var<storage,read> catalog:CraftCatalog;
// One invocation serializes overlapping coarse-cell reservations in stable site order.
@compute @workgroup_size(1)
fn claim_plots() {
 if p.options.x!=2u{return;}
 for(var i=0u;i<p.dims.y;i++){
  var e=economies[i];
  if e.claim.w>.5 && e.land_return.x>.5 {
   let cell=u32(e.claim.x);var eco=ecology[cell];let area=e.claim.z;
   if e.land_return.z>.5 && e.land_return.y<.5 {
    var litter=e.detritus.xyz;
    for(var j=0u;j<6u;j++){litter+=e.crops[j].y*catalog.goods[u32(catalog.crops[j*2u].x)].xyz;e.crops[j].y=0.;}
    for(var j=0u;j<3u;j++){litter+=e.herds[j].x*vec3(.25,.04,.003);e.herds[j].z+=e.herds[j].x;e.herds[j].x=0.;}
    let returned=e.soil.xyz+litter+e.forest.xyz+vec3(0.,0.,e.reserves.x);
    eco.pools[17]+=vec4(e.soil.xyz/area,0.);eco.pools[18]+=vec4(litter/area,0.);
    eco.pools[0]+=vec4(e.forest.xyz/area,0.);eco.pools[25].z+=e.reserves.x/area;
    eco.pools[27]+=vec4(returned/area,0.);e.exchange-=vec4(returned,0.);
    e.soil=vec4(0.,0.,0.,e.soil.w);e.detritus=vec4(0.);e.forest=vec4(0.);e.reserves.x=0.;
    e.return_flow.w+=e.water.x;e.water.w+=e.water.x;e.water.x=0.;
    e.land_return.y=1.;e.land_return.w+=e.claim.y/10000.;
   } else if e.land_return.z<.5 && e.land_return.y>.5 {
    let fraction=clamp(e.claim.y/area,0.,.05);
    e.soil=vec4(eco.pools[17].xyz*fraction*area,e.soil.w);eco.pools[17]=vec4(eco.pools[17].xyz*(1.-fraction),eco.pools[17].w);
    e.detritus=eco.pools[18]*fraction*area;eco.pools[18]*=1.-fraction;
    for(var k=0u;k<3u;k++){e.forest+=vec4(eco.pools[k].xyz*fraction*area,0.);eco.pools[k]=vec4(eco.pools[k].xyz*(1.-fraction),eco.pools[k].w);}
    e.reserves.x=eco.pools[25].z*fraction*area;eco.pools[25].z*=1.-fraction;
    let taken=e.soil.xyz+e.detritus.xyz+e.forest.xyz+vec3(0.,0.,e.reserves.x);
    e.exchange+=vec4(taken,0.);eco.pools[27]-=vec4(taken/area,0.);
    let water=eco.pools[25].w*fraction*area;eco.pools[25].w*=1.-fraction;eco.pools[27].w-=water/area;
    e.water.x+=water;e.water.z+=water;e.land_return.y=0.;
   }
   economies[i]=e;ecology[cell]=eco;
  }
  if e.claim.w>0.5{continue;}
  let cell=u32(e.claim.x);var eco=ecology[cell];let area=e.claim.z;let fraction=clamp(e.claim.y/area,0.,.05);
  e.soil=eco.pools[17]*fraction*area;eco.pools[17]*=1.-fraction;
  e.detritus=eco.pools[18]*fraction*area;eco.pools[18]*=1.-fraction;
  for(var k=0u;k<3u;k++){e.forest+=vec4(eco.pools[k].xyz*fraction*area,0.);eco.pools[k]=vec4(eco.pools[k].xyz*(1.-fraction),eco.pools[k].w);}
  e.reserves.x=eco.pools[25].z*fraction*area;eco.pools[25].z*=1.-fraction;
  let t=world[u32(src[i].habitat.z)];
  e.reserves.y=e.claim.y*.05*clamp(t.geology.z,0.,1.);
  e.reserves.z=e.claim.y*.2*clamp(t.terrain.z,.01,1.);
  let initial_water=eco.pools[25].w*fraction*area;
  e.water=vec4(initial_water,initial_water,0.,0.);eco.pools[25].w*=1.-fraction;eco.pools[27].w-=initial_water/area;
  e.baseline=vec4(e.soil.xyz+e.detritus.xyz+e.forest.xyz+vec3(0.,0.,e.reserves.x),0.);
  eco.pools[27]-=vec4(e.baseline.xyz/area,0.);
  e.claim.w=1.;economies[i]=e;ecology[cell]=eco;
 }
}
// Food granaries are separate; dry storage is shared by all nonfood goods.
fn dry_stock(e:Economy)->f32 {
 var mass=0.;for(var k=0u;k<63u;k++){if catalog.goods[k].w<=0.{mass+=e.goods[k/4u][k%4u];}}return mass;
}
fn order_room(e:Economy,k:u32)->f32 {
 if e.logistics.w<.5{return 1e30;}
 return min(max(0.,e.targets[k/4u][k%4u]-e.goods[k/4u][k%4u]),max(0.,e.logistics.x-e.logistics.y-dry_stock(e)));
}
fn warehouse_capacity(e:Economy)->f32 {
 let base=e.storage.z+min(e.storage.x/.02,e.storage.y/.03);
 return base+select(0.,min(base*.2,container_service(e)),catalog.methods[0].x>0.);
}
// Invoked even for empty sites: the physical shell persists and weathers.
fn weather_storage(i:u32) {
 var e=economies[i];e.storage_plan.z=0.;e.housing_plan.z=0.;e.waterworks_plan.z=0.;e.waterworks_plan.w=0.;e.water_service.y=0.;e.water_service.w=0.;
 e.waterworks_recovery.x=max(e.waterworks_recovery.x,min(e.waterworks.x/2.,e.waterworks.y/4.));e.waterworks_recovery.z=0.;
 let wear=.001+.02*clamp(e.soil.w,0.,1.);
 for(var j=0u;j<2u;j++) {
  let good=select(0u,5u,j==1u);let mass=e.storage[j]*wear;
  e.storage[j]-=mass;e.storage_plan.y+=mass;
  e.used[good/4u][good%4u]+=mass;
  e.detritus+=vec4(catalog.goods[good].xyz*mass,0.);e.reserves.w+=mass;
 }
 for(var j=0u;j<2u;j++) {
  let good=select(0u,5u,j==1u);let mass=e.housing[j]*wear;
  e.housing[j]-=mass;e.housing_plan.y+=mass;e.used[good/4u][good%4u]+=mass;
  e.detritus+=vec4(catalog.goods[good].xyz*mass,0.);e.reserves.w+=mass;
 }
 for(var j=0u;j<2u;j++) {
  let good=select(0u,5u,j==1u);let mass=e.waterworks[j]*wear;
  e.waterworks[j]-=mass;e.waterworks_plan.y+=mass;e.used[good/4u][good%4u]+=mass;
  e.detritus+=vec4(catalog.goods[good].xyz*mass,0.);e.reserves.w+=mass;
 }
 if e.storage_plan.w>.5 {e.logistics.x=warehouse_capacity(e);}
 economies[i]=e;
}
// Shared planner for the production dispatch and read-only participation forecasts.
fn production_labor(i:u32,e:Economy)->vec4<f32> {
 let recovery=select(0.,clamp(e.soil.w,0.,1.),(p.options.w&2u)!=0u);
 let available_workers=workers(i,src[i].stock.x)*(1.-.4*recovery);
 return available_workers*worker_shares(e,src[i].stock.x,available_workers);
}
@compute @workgroup_size(64)
fn forecast_labor(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=g.x;if i>=p.dims.y{return;}
 // Use only the output scratch buffer. No production, claims, fishing, wages,
 // population transitions or weather-storage wear occur in this pass.
 var row=Site(vec4(0.),vec4(0.),vec4(0.),vec4(0.));
 if src[i].stock.x>0. {
  let e=economies[i];
  row.stock=production_labor(i,e);
  {
   // Use opening stocks, not hypothetical new extraction or retail purchases.
   var preview=e;preview.labor=row.stock;preview.construction_workers.x=1.;
   preview.construction_workers.y=.2*max(0.,row.stock.w-e.exchange.w-dot(e.enterprise_plan,vec4(1.)));
   let recovery=select(0.,clamp(e.soil.w,0.,1.),(p.options.w&2u)!=0u);
   let built=building_work(preview,src[i],workers(i,src[i].stock.x)*(1.-.4*recovery));
   row.ledger.x=built.economy.construction_workers.z;
  }
  row.habitat=vec4(e.exchange.w,dot(e.enterprise_plan,vec4(1.)),e.fishery_plan.y+e.fishery_plan.z,workers(i,src[i].stock.x));
 }
 dst[i]=row;
}
// Pure local transaction shared by forecast and execution. The caller decides
// whether to persist its updated inventory; forecasting discards it.
struct BuildingResult {
 economy:Economy, labor:f32, industrial:f32, household:f32,
 types:vec4<f32>, firms:vec4<f32>
}
fn building_work(input:Economy,s:Site,available_workers:f32)->BuildingResult {
 var e=input;
 // Research workshops reserve staff before dispatch; no double-counted craft labor.
 var labor=max(0.,e.labor.w-e.exchange.w);
 if e.waterworks.w>.5 {
  let served=min(s.stock.x,min(e.waterworks.x/2.,e.waterworks.y/4.))*(1.-e.water_service.y);
  let operated=min(served,labor/.001);
  let work=min(labor,operated*.001);labor=max(0.,labor-work);
  e.water_service.z+=work;e.water_service.w=operated;
  e.waterworks_plan.w=clamp(operated/max(s.stock.x,1.),0.,1.);
 }
 // Builders cannot consume already contracted workshop shifts.
 let building_start=labor;
 var building_budget=1e30;
 if e.construction_workers.x>.5 {
  building_budget=min(e.construction_workers.y,max(0.,labor-dot(e.enterprise_plan,vec4(1.))));
  e.construction_workers.z=0.;
 }
 var asset_work=min(labor*.1,building_budget);
 // Under recovery priority, meet current shelter need before spending on headroom.
 // Reuse the same finite asset-work pool; all construction still consumes stocks.
 if e.waterworks.w>.5 && e.waterworks_recovery.y>.5 && e.housing_plan.w>.5 {
  let existing=min(e.housing.x/2.,e.housing.y/3.);
  let urgent_target=min(e.housing_plan.x,max(0.,s.stock.x-e.housing.z));
  var build=min(max(0.,urgent_target-existing),asset_work/.2);
  build=min(build,min(e.goods[0].x/2.,e.goods[1].y/3.));
  e.goods[0].x=max(0.,e.goods[0].x-build*2.);e.housing.x+=build*2.;
  e.goods[1].y=max(0.,e.goods[1].y-build*3.);e.housing.y+=build*3.;
  labor=max(0.,labor-build*.2);asset_work=max(0.,asset_work-build*.2);
  e.housing.w+=build*.2;e.housing_plan.z+=build;
 }
 // Repair previously installed service before optional housing headroom. Never
 // displace urgent shelter or reserve work when water/materials are unavailable.
 let shelter=e.housing.z+min(e.housing.x/2.,e.housing.y/3.);
 if e.waterworks.w>.5 && e.waterworks_recovery.y>.5 && (e.housing_plan.w<.5 || shelter>=s.stock.x) && e.water_service.y<.01 {
  let existing=min(e.waterworks.x/2.,e.waterworks.y/4.);
  let historical=e.waterworks_recovery.x;
  let repair_target=min(e.waterworks_plan.x,historical);
  var repair=min(max(0.,repair_target-existing),asset_work/.2);
  repair=min(repair,min(e.goods[0].x/2.,e.goods[1].y/4.));
  e.goods[0].x=max(0.,e.goods[0].x-repair*2.);e.waterworks.x+=repair*2.;
  e.goods[1].y=max(0.,e.goods[1].y-repair*4.);e.waterworks.y+=repair*4.;
  labor=max(0.,labor-repair*.2);asset_work=max(0.,asset_work-repair*.2);
  e.waterworks.z+=repair*.2;e.waterworks_plan.z+=repair;
  e.waterworks_recovery.z=repair*.2;e.waterworks_recovery.w+=repair*.2;
 }

 if e.housing_plan.w>.5 {
  let existing=min(e.housing.x/2.,e.housing.y/3.);
  var build=min(max(0.,e.housing_plan.x-existing),asset_work/.2);
  build=min(build,min(e.goods[0].x/2.,e.goods[1].y/3.));
  for(var j=0u;j<2u;j++){
   let good=select(0u,5u,j==1u);let cost=select(2.,3.,j==1u);let mass=build*cost;
   e.goods[good/4u][good%4u]=max(0.,e.goods[good/4u][good%4u]-mass);e.housing[j]+=mass;
  }
  labor-=build*.2;asset_work=max(0.,asset_work-build*.2);
  e.housing.w+=build*.2;e.housing_plan.z+=build;
 }
 if e.waterworks.w>.5 {
  let existing=min(e.waterworks.x/2.,e.waterworks.y/4.);
  var build=min(max(0.,e.waterworks_plan.x-existing),asset_work/.2);
  build=min(build,min(e.goods[0].x/2.,e.goods[1].y/4.));
  for(var j=0u;j<2u;j++){
   let good=select(0u,5u,j==1u);let cost=select(2.,4.,j==1u);let mass=build*cost;
   e.goods[good/4u][good%4u]=max(0.,e.goods[good/4u][good%4u]-mass);e.waterworks[j]+=mass;
  }
  labor-=build*.2;asset_work=max(0.,asset_work-build*.2);
  e.waterworks.z+=build*.2;e.waterworks_plan.z+=build;
 }
 if e.storage_plan.w>.5 {
  let existing=min(e.storage.x/.02,e.storage.y/.03);
  var build=min(max(0.,e.storage_plan.x-existing),asset_work/.002);
  build=min(build,min(e.goods[0].x/.02,e.goods[1].y/.03));
  for(var j=0u;j<2u;j++){
   let good=select(0u,5u,j==1u);let cost=select(.02,.03,j==1u);let mass=build*cost;
   e.goods[good/4u][good%4u]=max(0.,e.goods[good/4u][good%4u]-mass);e.storage[j]+=mass;
  }
  labor-=build*.002;e.storage.w+=build*.002;e.storage_plan.z=build;
  e.logistics.x=warehouse_capacity(e);
 }
 building_budget=max(0.,building_budget-(building_start-labor));
 e.workshop_plan.z=0.;e.workshop_plan.w=0.;
 var industrial_capacity=1e30;
 let specialized=e.workshop_types[0].w>.5;
 var household_capacity=max(1.,available_workers*.025);
 var type_capacity=vec4(0.);
 var firm_capacity=vec4(0.);e.enterprise_used=vec4(0.);
 for(var j=0u;j<4u;j++){e.workshop_types[j].z=0.;}
 if e.workshop.w>.5 {
  // Assets remain physical inventory. Wear returns organics and recoverable metal.
  for(var j=0u;j<3u;j++) {
   let good=select(select(0u,5u,j==1u),3u,j==2u);
   let worn=e.workshop[j]*.002;e.workshop[j]-=worn;
   e.used[good/4u][good%4u]+=worn;e.workshop_plan.y+=worn;
   let recovered=select(0.,worn*.9,j==2u&&catalog.herds[0].w>.5);
  e.goods[7].y+=recovered;e.made[7].y+=recovered;
   e.detritus+=vec4(catalog.goods[good].xyz*(worn-recovered),0.);
   e.reserves.w+=worn-recovered;
  }
  for(var j=0u;j<4u;j++){e.workshop_types[j].x*=.998;}
  // Construction and repair spend actual craft labor and stocked materials.
  let costs=vec3(20.,30.,2.);
  let units=min(e.workshop.x/20.,min(e.workshop.y/30.,e.workshop.z/2.));
  var build=min(max(0.,e.workshop_plan.x-units),min(labor*.1,building_budget)/2.);
  build=min(build,min(e.goods[0].x/20.,min(e.goods[1].y/30.,max(0.,e.goods[0].w-s.stock.x*.3)/2.)));
  for(var j=0u;j<3u;j++) {
   let good=select(select(0u,5u,j==1u),3u,j==2u);let mass=build*costs[j];
   e.goods[good/4u][good%4u]=max(0.,e.goods[good/4u][good%4u]-mass);e.workshop[j]+=mass;
  }
  labor-=build*2.;building_budget=max(0.,building_budget-build*2.);e.workshop_plan.z=build;
  industrial_capacity=household_capacity+4.*(units+build);
  if specialized {
   var assigned=0.;for(var j=0u;j<4u;j++){assigned+=e.workshop_types[j].x;}
   var unassigned=max(0.,units+build-assigned);
   // Existing generic assets require fitting labor; new builds include fitting.
   var fitting=build+min(labor*.1,building_budget)/.5;
   for(var step=0u;step<4u;step++){
    let j=(step+p.dims.z)%4u;
    let fitted=min(unassigned,min(fitting,max(0.,e.workshop_types[j].y-e.workshop_types[j].x)));
    e.workshop_types[j].x+=fitted;unassigned-=fitted;fitting-=fitted;
    type_capacity[j]=4.*e.workshop_types[j].x;
   }
   let fitted=units+build-assigned-unassigned;
   labor=max(0.,labor-max(0.,fitted-build)*.5);
  }
 }

 if e.construction_workers.x>.5 {e.construction_workers.z=max(0.,building_start-labor);}
 if specialized {
  for(var j=0u;j<4u;j++) {
   let leased=min(type_capacity[j],4.*e.enterprise_lease[j]*.998);
   type_capacity[j]-=leased;
   firm_capacity[j]=min(leased,e.enterprise_plan[j]);
  }
 }

 return BuildingResult(e,labor,industrial_capacity,household_capacity,type_capacity,firm_capacity);
}
fn ecological_production(i:u32,potential:f32,weather:f32)->f32 {
 var e=economies[i];let s=src[i];let t=world[u32(s.habitat.z)];let area=e.claim.y;
 let recovery=select(0.,clamp(e.soil.w,0.,1.),(p.options.w&2u)!=0u);
 let available_workers=workers(i,s.stock.x)*(1.-.4*recovery);e.labor=production_labor(i,e);
 if e.farm_workers.x>.5 {e.labor.x=min(e.labor.x,e.farm_workers.y);}
 if e.farm_workers.x>1.5 {e.labor.y=min(e.labor.y,e.extraction_workers.x);e.labor.z=min(e.labor.z,e.extraction_workers.y);e.extraction_workers.z=0.;e.extraction_workers.w=0.; }
 if e.logistics.w>3.5 {e.food_labor.x=mix(e.food_labor.x,e.food_labor.y,.25);}
 let rain=max(0.,t.hydro.z)*area/12000.*weather;
 e.water.z+=rain;e.water.x+=rain;
 let capacity=area*(.1+.4*e.policy.z)+select(0.,min(e.waterworks.x/2.,e.waterworks.y/4.),e.waterworks.w>.5);let runoff=max(0.,e.water.x-capacity);e.water.x-=runoff;e.water.w+=runoff;
 if e.land_return.x>.5 {
  // Dissolved nutrient export is bounded by actual runoff and soil inventories.
  let fraction=min(.05,runoff/max(area*.2,1.));let nutrients=e.soil.xyz*fraction;
  e.soil-=vec4(nutrients,0.);e.exchange-=vec4(nutrients,0.);
  e.return_flow+=vec4(nutrients,runoff);
 }
 if e.waterworks.w>.5 {
  let need=s.stock.x*.09;let supplied=min(e.water.x,need);
  e.water.x-=supplied;e.water.w+=supplied;e.water_service.x+=supplied;
  e.water_service.y=clamp(1.-supplied/max(need,1e-6),0.,1.);
 }
 let release=min(e.reserves.x,e.reserves.x*.0000001);e.reserves.x-=release;e.soil.z+=release;
 let decay=e.detritus.xyz*.08;e.detritus-=vec4(decay,0.);e.soil+=vec4(0.,decay.yz,0.);e.exchange.x-=decay.x;
 let fixation_cost=max(1.,catalog.herds[0].z);
 let fixed=min(potential*.05/fixation_cost,area*.002/12.*e.policy.x*clamp((t.hydro.y+5.)/20.,0.,1.));
 e.soil.y+=fixed;e.exchange.y+=fixed;
 let tool_factor=.75+.25*clamp((e.goods[0].w+select(0.,e.goods[10].y+.6*e.goods[10].w,e.extraction.y>.5))/max(1.,s.stock.x*.5),0.,1.);
 e.production_probe.x=tool_factor;e.production_probe.z=e.goods[0].w+select(0.,e.goods[10].y+.6*e.goods[10].w,e.extraction.y>.5);e.production_probe.w=s.stock.x;
 var output=max(0.,potential*(1.-e.policy.x)*tool_factor*(1.-.5*recovery)-fixed*fixation_cost);
 if e.management.x>.5{output=0.;}
 let n=e.soil.y/.02;let ph=e.soil.z/.003;let water=e.water.x/.5;
 e.diagnostics.x=0.;if n<output{e.diagnostics.x=1.;}output=min(output,n);if ph<output{e.diagnostics.x=2.;}output=min(output,ph);if water<output{e.diagnostics.x=3.;}output=min(output,water);
 e.soil.y-=output*.02;e.soil.z-=output*.003;e.exchange.x+=output*.45;e.water.x-=output*.5;e.water.w+=output*.5;e.diagnostics.y=output;
 // Finite timber harvest; a regional woodland stock, not unlimited yield from cover.
 let wood_potential=min(e.labor.y*extraction_rate(e,0u),min(e.forest.x/.5,min(e.forest.y/.002,e.forest.z/.0002)));
 let wood=min(wood_potential,order_room(e,0u));
 if e.farm_workers.x>1.5 {e.extraction_workers.z=wood/extraction_rate(e,0u); }
 e.forest-=vec4(wood*vec3(.5,.002,.0002),0.);e.goods[0].x+=wood;e.made[0].x+=wood;
 // One mining workforce serves ore and clay. Rotate priority to avoid starving
 // either industry when both have orders; all policies obey the physical budget.
 var mining=e.labor.z;
 for(var extract=0u;extract<2u;extract++) {
 let mineral=(extract+p.dims.z)%2u;let good=select(ore_good(e),4u,mineral==1u);
 let rate=extraction_rate(e,mineral+1u);
 let quantity=min(min(e.reserves[mineral+1u],mining*rate),order_room(e,good));
 e.reserves[mineral+1u]-=quantity;e.goods[good/4u][good%4u]+=quantity;e.made[good/4u][good%4u]+=quantity;mining=max(0.,mining-quantity/rate);
 }
 if e.farm_workers.x>1.5 {e.extraction_workers.w=max(0.,e.labor.z-mining); }
 let building=building_work(e,s,available_workers);
 e=building.economy;var labor=building.labor;
 var industrial_capacity=building.industrial;var household_capacity=building.household;
 var type_capacity=building.types;var firm_capacity=building.firms;
 let specialized=e.workshop_types[0].w>.5;

 // The priority pass borrows from the same finite labor and capacity pools.
 // Blocked reservations expire within this dispatch; the ordinary pass gets all remaining work.
 e.tool_work=vec4(0.);e.tool_work.x=select(0.,labor*.2,e.tool_craft.y>.5);
 var completed:array<f32,64>;
 for(var wave=0u;wave<2u;wave++){
 if wave==0u && e.tool_craft.y<.5 {continue;}
 for(var step=0u;step<p.options.y;step++){
  let r=select(step,(step+p.dims.z)%p.options.y,e.logistics.w>.5);
  let recipe=catalog.recipes[r];if recipe.work.y>0. && (u32(e.management.w)&(1u<<u32(recipe.work.y-1.)))==0u{continue;}let tool_output=recipe.output[0].w+recipe.output[10].y+recipe.output[10].w;
  var unit_work=recipe.work.x;
  if tool_output>0. && e.tool_craft.z>.5 {unit_work*=1.-.2*e.tool_craft.x;}
  var batches=labor/max(unit_work,.001)/min(5.,f32(p.options.y-step));
  if wave==0u {batches=min(e.tool_orders[r/4u][r%4u],max(0.,e.tool_work.x-e.tool_work.y)/max(unit_work,.001));}
  var household=false;
  for(var k=0u;k<64u;k++){if recipe.output[k/4u][k%4u]>0. && catalog.goods[k].w>0.{household=true;}}
  let industry=u32(recipe.work.z);
  if !household{
   var capacity=industrial_capacity;
   if specialized{capacity=type_capacity[industry]+firm_capacity[industry]+household_capacity;}
   batches=min(batches,capacity/max(unit_work,.001));
  }
  if e.logistics.w>.5{batches=min(batches,max(0.,e.orders[r/4u][r%4u]-completed[r]));}
  for(var k=0u;k<64u;k++){let input=recipe.input[k/4u][k%4u];if input>0.{batches=min(batches,e.goods[k/4u][k%4u]/input);}}
  for(var k=0u;k<64u;k++){let quantity=recipe.output[k/4u][k%4u];if quantity>0.{batches=min(batches,max(0.,select(s.stock.x*8.,e.targets[k/4u][k%4u],e.logistics.w>.5)-e.goods[k/4u][k%4u])/quantity);}}
  if e.extraction.y>.5 && recipe.work.w>0. {
   let room=max(0.,e.residue.z-e.residue.x)/recipe.work.w;
   if batches>room {e.residue.w+=1.;}batches=min(batches,room);
  }
  // Conversions that shrink dry storage remain possible even in an overfull legacy yard.
  var expansion=0.;for(var k=0u;k<63u;k++){if catalog.goods[k].w<=0.{expansion+=recipe.output[k/4u][k%4u]-recipe.input[k/4u][k%4u];}}
  if e.logistics.w>.5 && expansion>0.{batches=min(batches,max(0.,e.logistics.x-e.logistics.y-dry_stock(e))/expansion);}
  // Exhausted f32 labor can leave a tiny negative remainder. Never reverse a recipe.
  batches=max(0.,batches);
  var inputs=vec3(0.);var outputs=vec3(0.);
  for(var k=0u;k<64u;k++){inputs+=catalog.goods[k].xyz*recipe.input[k/4u][k%4u]*batches;outputs+=catalog.goods[k].xyz*recipe.output[k/4u][k%4u]*batches;}
  let lost=max(vec3(0.),inputs-outputs);e.exchange.x-=lost.x;e.detritus+=vec4(0.,lost.yz,0.);
  for(var k=0u;k<64u;k++){let input=recipe.input[k/4u][k%4u]*batches;let output=recipe.output[k/4u][k%4u]*batches;e.goods[k/4u][k%4u]=max(0.,e.goods[k/4u][k%4u]-input)+output;e.used[k/4u][k%4u]+=input;e.made[k/4u][k%4u]+=output;e.reserves.w+=input-output;}
  let deposited=select(0.,batches*recipe.work.w,e.extraction.y>.5);
  e.residue.x+=deposited;e.residue.y+=deposited;e.reserves.w-=deposited;
  completed[r]+=batches;
  labor=max(0.,labor-batches*unit_work);
  if wave==0u {e.tool_work.y+=batches*unit_work;}
  if tool_output>0. {e.tool_work.z+=batches*unit_work;e.tool_work.w+=batches*tool_output;}
  if !household{
   let work=batches*unit_work;
   industrial_capacity=max(0.,industrial_capacity-work);e.workshop_plan.w+=work;
   e.workshop_types[industry].z+=work;
   if specialized {
    let installed=type_capacity[industry]+firm_capacity[industry];
    let equipped=min(work,installed);
    // Prepaid operator shifts execute first; ordinary communal capacity supplies remaining work.
    let contracted=min(firm_capacity[industry],equipped);
    firm_capacity[industry]=max(0.,firm_capacity[industry]-contracted);
    type_capacity[industry]=max(0.,type_capacity[industry]-(equipped-contracted));
    e.enterprise_used[industry]+=contracted;
    household_capacity=max(0.,household_capacity-(work-equipped));
   }
  }
 }
 }
 if e.tool_craft.z>.5 {
  let practice=clamp(e.tool_work.z/max(available_workers,.001),0.,1.);
  // Skill cannot arise from queued/blocked work; inactivity gradually erodes it.
  e.tool_craft.x=clamp(e.tool_craft.x+.04*practice*(1.-e.tool_craft.x)-.002*(1.-practice)*e.tool_craft.x,0.,1.);
  e.tool_craft.w+=e.tool_work.z;
 }
 e.logistics.z=max(0.,labor);
 // Legacy stored wear; planned economies wear only the useful household/working quantity.
 for(var k=0u;k<64u;k++){
  var in_use=0.;var rate=.005;
  if e.logistics.w<.5{if k==3u||k==5u||k==7u||((k==41u||k==43u)&&e.extraction.y>.5){in_use=e.goods[k/4u][k%4u];}}
  else {
   if k==3u{in_use=min(e.goods[k/4u][k%4u],s.stock.x*.5);}
   if k==41u&&e.extraction.y>.5 {in_use=min(e.goods[10].y,max(0.,s.stock.x*.5-e.goods[0].w));}
   if k==43u&&e.extraction.y>.5 {in_use=min(e.goods[10].w,max(0.,s.stock.x*.5-e.goods[0].w-e.goods[10].y)/.6);}
   if k==7u{in_use=min(e.goods[k/4u][k%4u],s.stock.x*2.);}
   if k==18u{in_use=min(e.goods[k/4u][k%4u],s.stock.x*.6);rate=.025;}
   if k==20u{in_use=min(e.goods[k/4u][k%4u],s.stock.x*.1);rate=.01;}
   if k==22u||k==23u{in_use=min(e.goods[k/4u][k%4u],s.stock.x*.1);rate=.002;}
  }
  if k==43u&&e.extraction.y>.5 {rate=.01;}
  if k>=45u && k<=50u && catalog.methods[k-45u].x>0. && catalog.methods[k-45u].x<5. {in_use=min(e.goods[k/4u][k%4u],s.stock.x*.2);rate=catalog.methods[k-45u].z;}
  let worn=in_use*rate;e.goods[k/4u][k%4u]-=worn;e.used[k/4u][k%4u]+=worn;
  var recovered=select(0.,worn*.9,(k==3u||k==22u||k==23u) && catalog.herds[0].w>.5);
  if k>=45u && k<=50u {recovered=worn*catalog.methods[k-45u].w*.9;}
   if (k==41u||k==43u) && e.extraction.y>.5 {
   let scrap_good=select(42u,44u,k==43u);let scrap=worn*.9;e.goods[scrap_good/4u][scrap_good%4u]+=scrap;e.made[scrap_good/4u][scrap_good%4u]+=scrap;e.reserves.w-=scrap;
  }
  e.goods[7].y+=recovered;e.made[7].y+=recovered;e.reserves.w+=worn-recovered;
  // Organic worn material is a recorded detrital transfer, not a missing C/N/P sink.
  e.detritus+=vec4(select(worn-recovered,worn,k>=45u&&k<=50u)*catalog.goods[k].xyz,0.);
 }
 if e.management.x>.5 { e=managed_production(i,e,max(0.,potential*(1.-e.policy.x)*tool_factor*(1.-.5*recovery)-fixed*fixation_cost),weather);output=e.diagnostics.y; }
 // Retain a year's approximate memory of observed food handling returns. This
 // is a lagged average, not a perfect forecast or a claimed marginal farm yield.
 let fish_chem=catalog.goods[28];let fish_energy=min(fish_chem.w,min(fish_chem.x/.45,min(fish_chem.y/.02,fish_chem.z/.003)));
 let alternative=max(0.,output-e.fishery_plan.w*fish_energy)/max(dot(e.labor,vec4(1.)),1.);
 e.fishery_choice.x=mix(e.fishery_choice.x,alternative,1./12.);
 e.reserves.w=max(0.,e.reserves.w);e.soil=max(e.soil,vec4(0.));e.forest=max(e.forest,vec4(0.));e.water.x=max(0.,e.water.x);economies[i]=e;return max(0.,output);
}
fn return_food(i:u32,amount:f32){
 var e=economies[i];let nutrients=amount*vec2(.02,.003);let returned=nutrients*e.policy.y;
 e.detritus+=vec4(0.,returned,0.);e.exchange-=vec4(amount*.45,nutrients-returned,0.);e.diagnostics.z=returned.x;e.diagnostics.w=returned.y;economies[i]=e;
}

// Canopy proxy across sowing, expansion, flowering, filling and senescence.
fn crop_canopy(phase:u32)->f32 {
 if phase>6u {return 0.;}
 return array<f32,7>(.2,.6,1.,1.,.8,.4,.1)[phase];
}
// Managed growth and husbandry share finite land, water, feed and nutrients.
fn farm_attendance(e:Economy)->f32 {
 if e.farm_workers.x<.5 {return 1.;}
 return clamp(e.farm_workers.y/max(e.farm_workers.w,.000001),0.,1.);
}
fn managed_production(i:u32,input:Economy,potential:f32,weather:f32)->Economy {
 var e=input;let s=src[i];let t=world[u32(s.habitat.z)];let month=p.dims.z%12u;
 let temp=select(t.hydro.y,t.climate.x,(p.options.w&2u)!=0u);
 let moisture=max(0.,t.hydro.z)*weather;
 var demands:array<f32,6>;
 var total_need=vec3(0.); // N, P and water, measured before any crop uptake.
 for(var j=0u;j<6u;j++){
  let params=catalog.crops[j*2u];let growth_params=catalog.crops[j*2u+1u];let good=u32(params.x);let chemistry=catalog.goods[good].xyz;var c=e.crops[j];
  let seasonal=catalog.seasons[j];let harvest=(u32(demography[i].crops.z)+u32(growth_params.w))%12u;
  let phase=(month+12u-(harvest+6u)%12u)%12u;
  if seasonal.x>0. && phase==0u {let planted=min(c.z,max(2.,s.stock.x*.02))*farm_attendance(e);c.z-=planted;c.y+=planted;}
  let habitat=clamp((temp-params.y)/10.,0.,1.)*clamp((params.z-temp)/10.,0.,1.)*clamp(moisture/params.w,0.,1.);
  // Every crop shares the same bounded total potential and 5% of land is pasture.
  var growth=potential*.95*c.x*growth_params.x*habitat*select(0.,1.,c.z>0.001||c.y>0.001);
  if seasonal.x>0. {
   // Crop-equivalent standing mass, normalized to a common 0.45 kg-C budget.
   // Stored seed is dormant. Monthly phenology is a calendar proxy, not thermal time.
   // The survey supplies annual harvest potential, not whole-plant biomass.
   // Distribute it over the canopy calendar and include the biomass required
   // for residues. All of that biomass still consumes finite N/P/water.
   let canopy=crop_canopy(phase)*(12./4.1)/max(seasonal.y,.05);
   let thermal=clamp((temp-params.y)/10.,0.,1.)*clamp((params.z-temp)/10.,0.,1.);
   growth=potential*.95*c.x*growth_params.x*(.45/max(chemistry.x,.01))*thermal*canopy*f32(c.y>.001);
   let frost=c.y*seasonal.w*clamp((params.y-temp)/10.,0.,1.);
   c.y-=frost;e.detritus+=vec4(frost*chemistry,0.);
  }
  // Industrial crops stop growing once standing crop plus stores cover orders.
  if e.logistics.w>.5 && catalog.goods[good].w<=0.{growth=min(growth,max(0.,order_room(e,good)-c.y));}
  demands[j]=max(0.,growth);
  total_need+=demands[j]*vec3(chemistry.yz,growth_params.y);
  e.crops[j]=c;
 }
 // Proportional rationing: all crops face the same resource snapshot. This bounded
 // single round may leave other resources unused when one resource is limiting.
 let supply=max(vec3(e.soil.yz,e.water.x),vec3(0.));
 let fractions=select(vec3(1.),min(vec3(1.),supply/max(total_need,vec3(.000001))),total_need>vec3(0.));
 let fulfilled=min(fractions.x,min(fractions.y,fractions.z));
 if fulfilled<1. {e.diagnostics.x=select(select(3.,2.,fractions.y<=fractions.z),1.,fractions.x<=min(fractions.y,fractions.z));}
 for(var j=0u;j<6u;j++){
  let params=catalog.crops[j*2u];let growth_params=catalog.crops[j*2u+1u];let good=u32(params.x);let chemistry=catalog.goods[good].xyz;var c=e.crops[j];
  let seasonal=catalog.seasons[j];let harvest=(u32(demography[i].crops.z)+u32(growth_params.w))%12u;
  let phase=(month+12u-(harvest+6u)%12u)%12u;
  if seasonal.x>0. && phase>=3u && phase<=4u && demands[j]>.00001 {
   let damage=c.y*seasonal.z*(1.-fractions.z);
   c.y-=damage;e.detritus+=vec4(damage*chemistry,0.);
  }
  // Guard only against final float32 subtraction error, not sequential allocation.
  let capacity=min(e.soil.y/max(chemistry.y,.00001),min(e.soil.z/max(chemistry.z,.00001),e.water.x/growth_params.y));
  let growth=max(0.,min(demands[j]*fulfilled,capacity));
  e.soil.y=max(0.,e.soil.y-growth*chemistry.y);e.soil.z=max(0.,e.soil.z-growth*chemistry.z);e.exchange.x+=growth*chemistry.x;e.water.x-=growth*growth_params.y;e.water.w+=growth*growth_params.y;c.y+=growth;e.agriculture.x+=growth;
  if month==harvest {
   let lost=c.y*(1.-farm_attendance(e));c.y-=lost;e.detritus+=vec4(lost*chemistry,0.);
   if seasonal.x>0. {let residue=c.y*(1.-seasonal.y);c.y-=residue;e.detritus+=vec4(residue*chemistry,0.);}
   let seed=min(c.y*.05,max(2.,s.stock.x*.02));let harvested=max(0.,c.y-seed);e.goods[good/4u][good%4u]+=harvested;e.made[good/4u][good%4u]+=harvested;c.w+=harvested;c.z+=seed;c.y=0.;}
  if seasonal.x==0. && month==(harvest+6u)%12u {let planted=min(c.z,max(2.,s.stock.x*.02))*farm_attendance(e);c.z-=planted;c.y+=planted;}
  // Purchased seed can establish a new daughter farm; no spontaneous imports.
  if c.z+c.y<.001{let seed=min(2.,e.goods[good/4u][good%4u])*farm_attendance(e);e.goods[good/4u][good%4u]-=seed;e.used[good/4u][good%4u]+=seed;c.z+=seed;}
  e.crops[j]=c;
 }
 for(var j=0u;j<3u;j++){
  var a=e.herds[j];if a.x<=0.{continue;}
  let body=vec3(.25,.04,.003);let feed_good=u32(catalog.herds[j].y);let feed_chem=catalog.goods[feed_good].xyz;
  let need=a.x*.08;let taken=min(need,e.goods[feed_good/4u][feed_good%4u]);e.goods[feed_good/4u][feed_good%4u]-=taken;e.used[feed_good/4u][feed_good%4u]+=taken;e.agriculture.y+=taken;
  let fed=taken/max(need,.001);let carrying=max(1.,e.claim.y/10000.*.1*20.);
  let gain=min(a.x*.015*fed*clamp(1.-a.x/carrying,0.,1.),min(taken*feed_chem.x/body.x,min(taken*feed_chem.y/body.y,taken*feed_chem.z/body.z))*.4);
  var leftover=max(vec3(0.),taken*feed_chem-gain*body);a.x+=gain;a.y+=gain;
  let dead=min(a.x,a.x*(.002+(1.-fed)*.04));a.x-=dead;a.z+=dead;e.detritus+=vec4(dead*body,0.);
  let product=u32(catalog.herds[j].x);let chemistry=catalog.goods[product].xyz;
  let made=min(a.x*.015*fed,min(leftover.x/max(chemistry.x,.0001),min(leftover.y/max(chemistry.y,.0001),leftover.z/max(chemistry.z,.0001))));
  leftover-=made*chemistry;e.goods[product/4u][product%4u]+=made;e.made[product/4u][product%4u]+=made;a.w+=made;
  let slaughter=min(a.x,a.x*select(.01,.03,fed<.5)*select(0.,1.,fed<.5||a.x>carrying*.75));
  a.x-=slaughter;a.z+=slaughter;
  let meat=slaughter*.6;let hides=slaughter*.1;e.goods[6].x+=meat;e.made[6].x+=meat;e.goods[4].w+=hides;e.made[4].w+=hides;
  e.detritus+=vec4(max(vec3(0.),slaughter*body-meat*catalog.goods[24].xyz-hides*catalog.goods[19].xyz),0.);
  e.exchange.x-=leftover.x*.6;e.detritus+=vec4(leftover*vec3(.4,1.,1.),0.);e.herds[j]=a;
 }
 // Food processing conserves elements: calorie-equivalent output is also limited
 // by its embodied C/N/P; remaining material becomes compost, not extra food.
 var food=0.;
 for(var good=8u;good<63u;good++){
  let energy=catalog.goods[good].w;if energy<=0.{continue;}
  let wanted=max(0.,s.stock.x*18.*18.-s.stock.y-food);if wanted<=0.{break;}
  let quantity=min(e.goods[good/4u][good%4u],wanted/energy);let matter=quantity*catalog.goods[good].xyz;
  let equivalent=min(quantity*energy,min(matter.x/.45,min(matter.y/.02,matter.z/.003)));
  e.goods[good/4u][good%4u]-=quantity;e.used[good/4u][good%4u]+=quantity;
  e.detritus+=vec4(max(vec3(0.),matter-equivalent*vec3(.45,.02,.003)),0.);food+=equivalent;e.agriculture.w+=quantity-equivalent;
 }
 // One shared six-month raw-food capacity, plus the existing cooked-food granary.
 var raw_energy=0.;for(var good=8u;good<63u;good++){raw_energy+=e.goods[good/4u][good%4u]*catalog.goods[good].w;}
 let excess=clamp(1.-s.stock.x*18.*6./max(raw_energy,.001),0.,1.);
 for(var good=8u;good<63u;good++){if catalog.goods[good].w<=0.{continue;}let loss=e.goods[good/4u][good%4u]*(excess+(1.-excess)*.005);e.goods[good/4u][good%4u]-=loss;e.used[good/4u][good%4u]+=loss;e.detritus+=vec4(loss*catalog.goods[good].xyz,0.);e.agriculture.w+=loss;}
 if e.logistics.w>.5{
  for(var k=19u;k<=27u;k++){if k!=19u&&k!=27u{continue;}let excess=max(0.,e.goods[k/4u][k%4u]-max(s.stock.x,e.targets[k/4u][k%4u]));e.goods[k/4u][k%4u]-=excess;e.used[k/4u][k%4u]+=excess;e.detritus+=vec4(excess*catalog.goods[k].xyz,0.);}
 }
 e.diagnostics.y=food;return e;
}

// Equipment is a finite material inventory. Retirement/closure preserves assets;
// passive wear remains a recorded transfer even when the fishery is idle.
// Serial site order prevents two fisheries from double-withdrawing a coarse cell.
@compute @workgroup_size(1)
// Expand the three fixed material/guild operations: dynamic vector-component
// stores in these nested loops crash the NVIDIA 595.84 SPIR-V compiler.
fn adaptive_fish_plots(){
 for(var i=0u;i<p.dims.y;i++){
 if economies[i].fishery.w<.5 && dot(economies[i].fishery.xyz,vec3(1.))+economies[i].fishery_traps.x<=0.{continue;}
 let s=src[i];let materials=array<u32,3>(0u,3u,16u);let cost=vec3(40.,1.,4.);
 {let j=0u;
  let good=materials[j];let worn=economies[i].fishery[j]*.003;economies[i].fishery[j]-=worn;
  economies[i].used[good/4u][good%4u]+=worn;economies[i].detritus+=vec4(worn*catalog.goods[good].xyz,0.);economies[i].fishery_stats.x+=worn;
 }
{let j=1u;
  let good=materials[j];let worn=economies[i].fishery[j]*.003;economies[i].fishery[j]-=worn;
  economies[i].used[good/4u][good%4u]+=worn;economies[i].detritus+=vec4(worn*catalog.goods[good].xyz,0.);economies[i].fishery_stats.x+=worn;
 }
{let j=2u;
  let good=materials[j];let worn=economies[i].fishery[j]*.003;economies[i].fishery[j]-=worn;
  economies[i].used[good/4u][good%4u]+=worn;economies[i].detritus+=vec4(worn*catalog.goods[good].xyz,0.);economies[i].fishery_stats.x+=worn;
 }
 let trap_wear=economies[i].fishery_traps.x*.003;
 economies[i].fishery_traps.x-=trap_wear;economies[i].fishery_traps.z+=trap_wear;
 economies[i].used[0].x+=trap_wear;economies[i].detritus+=vec4(trap_wear*catalog.goods[0].xyz,0.);
 economies[i].fishery_plan=vec4(0.);economies[i].fishery_stats.w=0.;economies[i].fishery_choice.y=0.;economies[i].fishery_choice.z=0.;
 if economies[i].fishery.w<.5||economies[i].management.x<.5||economies[i].management.y<1.||s.stock.x<1.{continue;}
 let cell=u32(economies[i].management.y)-1u;var eco=ecology[cell];let area=economies[i].management.z;
 let slots=array<u32,3>(13u,15u,14u);let capture=array<f32,3>(1.,.25,.1);
 var density=0.;{let j=0u;density+=eco.pools[slots[j]].x*capture[j];}
{let j=1u;density+=eco.pools[slots[j]].x*capture[j];}
{let j=2u;density+=eco.pools[slots[j]].x*capture[j];}
 let response=density/(density+economies[i].fishery_config.z);economies[i].fishery_stats.w=response;
 let recovery=select(0.,clamp(economies[i].soil.w,0.,1.),(p.options.w&2u)!=0u);
 let workforce=workers(i,s.stock.x)*(1.-.4*recovery);
 var stored=s.stock.y;for(var good=8u;good<63u;good++){stored+=economies[i].goods[good/4u][good%4u]*catalog.goods[good].w;}
 let chem=catalog.goods[28].xyz;
 let energy=min(catalog.goods[28].w,min(chem.x/.45,min(chem.y/.02,chem.z/.003)));
 let deficit=max(0.,s.stock.x*18.*economies[i].fishery_config.w-stored);
 let wanted=min(deficit/max(energy,.001),max(0.,s.stock.x*18.*6./max(energy,.001)-economies[i].goods[7].x));
 let rate=economies[i].fishery_config.y*response;
 var desired=min(workforce*economies[i].fishery_config.x*response,wanted/max(rate,.001));
 let capacity=min(economies[i].fishery.x/cost.x,min(economies[i].fishery.y/cost.y,economies[i].fishery.z/cost.z));
 let trap_capacity=select(0.,economies[i].fishery_traps.x/20.,economies[i].fishery_traps.w>.5);
 var factor=1.;
 if economies[i].fishery_choice.w>.5 {
  // Expected mix uses installed outfits; new primitive crews have lower returns.
  let efficiency=select(1.,.35+.65*clamp(capacity/max(desired,.001),0.,1.),economies[i].fishery_traps.w>.5);
  // Amortize two construction worker-months over twelve months for new capacity.
  let investment=2./12.*clamp((desired-capacity-trap_capacity)/max(desired,.001),0.,1.);
  let expected=rate*energy*efficiency/(1.+investment);
  economies[i].fishery_choice.y=expected;
  factor=clamp(1.-economies[i].fishery_choice.x/max(expected,.001),0.,1.);
 }
 economies[i].fishery_choice.z=factor;desired*=factor;
 economies[i].fishery_plan.x=desired;
 var build=min(max(0.,desired-capacity-trap_capacity),desired*.25/2.);
 {let j=0u;
  let good=materials[j];let reserved=select(0.,s.stock.x*.15,good==3u);
  build=min(build,max(0.,economies[i].goods[good/4u][good%4u]-reserved)/cost[j]);
 }
{let j=1u;
  let good=materials[j];let reserved=select(0.,s.stock.x*.15,good==3u);
  build=min(build,max(0.,economies[i].goods[good/4u][good%4u]-reserved)/cost[j]);
 }
{let j=2u;
  let good=materials[j];let reserved=select(0.,s.stock.x*.15,good==3u);
  build=min(build,max(0.,economies[i].goods[good/4u][good%4u]-reserved)/cost[j]);
 }
 {let j=0u;
  let good=materials[j];let mass=build*cost[j];economies[i].goods[good/4u][good%4u]-=mass;economies[i].fishery[j]+=mass;economies[i].fishery_stats.z+=mass;
 }
{let j=1u;
  let good=materials[j];let mass=build*cost[j];economies[i].goods[good/4u][good%4u]-=mass;economies[i].fishery[j]+=mass;economies[i].fishery_stats.z+=mass;
 }
{let j=2u;
  let good=materials[j];let mass=build*cost[j];economies[i].goods[good/4u][good%4u]-=mass;economies[i].fishery[j]+=mass;economies[i].fishery_stats.z+=mass;
 }
 // A timber trap alternative can establish a shore fishery without metal tools
 // or spun fiber. It is slower, still consumes timber and the same construction work.
 var trap_build=0.;
 if economies[i].fishery_traps.w>.5 {
  trap_build=min(max(0.,desired-capacity-build-trap_capacity),min(max(0.,desired*.25/2.-build),economies[i].goods[0].x/20.));
 }
 // Clamp the transferred mass, not just units: (stock/20)*20 can round above stock.
 let trap_mass=min(economies[i].goods[0].x,trap_build*20.);trap_build=trap_mass/20.;
 economies[i].goods[0].x-=trap_mass;
 economies[i].fishery_traps.x+=trap_mass;economies[i].fishery_traps.y+=trap_mass;
 economies[i].fishery_plan.z=(build+trap_build)*2.;
 let crew=min(max(0.,desired-economies[i].fishery_plan.z),capacity+build);
 let trap_crew=min(max(0.,desired-economies[i].fishery_plan.z-crew),trap_capacity+trap_build);
 let potential=(crew+trap_crew*.35)*rate;
 let realized_rate=select(rate,potential/max(crew+trap_crew,.001),trap_crew>0.);
 var remaining=min(wanted,potential);
 var catch_total=0.;
 {let j=0u;
  let slot=slots[j];let available=eco.pools[slot].xyz*area;
  let caught=min(remaining,min(available.x/max(chem.x,1e-9),min(available.y/max(chem.y,1e-9),available.z/max(chem.z,1e-9)))*.001*capture[j]);
  eco.pools[slot]-=vec4(caught*chem/area,0.);eco.pools[27]-=vec4(caught*chem/area,0.);
  economies[i].exchange+=vec4(caught*chem,0.);economies[i].goods[7].x+=caught;economies[i].made[7].x+=caught;
  remaining=max(0.,remaining-caught);catch_total+=caught;
 }
{let j=1u;
  let slot=slots[j];let available=eco.pools[slot].xyz*area;
  let caught=min(remaining,min(available.x/max(chem.x,1e-9),min(available.y/max(chem.y,1e-9),available.z/max(chem.z,1e-9)))*.001*capture[j]);
  eco.pools[slot]-=vec4(caught*chem/area,0.);eco.pools[27]-=vec4(caught*chem/area,0.);
  economies[i].exchange+=vec4(caught*chem,0.);economies[i].goods[7].x+=caught;economies[i].made[7].x+=caught;
  remaining=max(0.,remaining-caught);catch_total+=caught;
 }
{let j=2u;
  let slot=slots[j];let available=eco.pools[slot].xyz*area;
  let caught=min(remaining,min(available.x/max(chem.x,1e-9),min(available.y/max(chem.y,1e-9),available.z/max(chem.z,1e-9)))*.001*capture[j]);
  eco.pools[slot]-=vec4(caught*chem/area,0.);eco.pools[27]-=vec4(caught*chem/area,0.);
  economies[i].exchange+=vec4(caught*chem,0.);economies[i].goods[7].x+=caught;economies[i].made[7].x+=caught;
  remaining=max(0.,remaining-caught);catch_total+=caught;
 }
 economies[i].fishery_plan.y=catch_total/max(realized_rate,.001);economies[i].fishery_plan.w=catch_total;economies[i].agriculture.z+=catch_total;
 economies[i].fishery_stats.y+=economies[i].fishery_plan.y+economies[i].fishery_plan.z;
 ecology[cell]=eco;continue;
 }
}

@compute @workgroup_size(1)
fn fish_plots(){
 for(var i=0u;i<p.dims.y;i++){
 var e=economies[i];
 if e.fishery.w>.5{continue;}
 if e.management.x<.5||e.management.y<1.||src[i].stock.x<1.{continue;}
 let cell=u32(e.management.y)-1u;let area=e.management.z;var eco=ecology[cell];let fish=28u;let chemistry=catalog.goods[fish].xyz;
 // One shared labor budget; larger river animals and predators are less catchable.
 // Guild changes must not make a populated fishery invisible to the economy.
 var remaining=workers(i,src[i].stock.x)*.02;
 let stocks=array<u32,3>(13u,15u,14u);let catchability=array<f32,3>(1.,.25,.1);
 for(var prey=0u;prey<3u;prey++){
 let slot=stocks[prey];let available=eco.pools[slot].xyz*area;
 let caught=min(remaining,min(available.x/chemistry.x,min(available.y/chemistry.y,available.z/chemistry.z))*.001*catchability[prey]);
 eco.pools[slot]-=vec4(caught*chemistry/area,0.);eco.pools[27]-=vec4(caught*chemistry/area,0.);e.exchange+=vec4(caught*chemistry,0.);e.goods[fish/4u][fish%4u]+=caught;e.made[fish/4u][fish%4u]+=caught;e.agriculture.z+=caught;
 remaining=max(0.,remaining-caught);
 }
 economies[i]=e;ecology[cell]=eco;
 }
}

// Keep the disabled-policy arithmetic isolated for exact continuation of existing worlds.
fn worker_shares(e:Economy,pop:f32,available_workers:f32)->vec4<f32>{
 var shares:vec4<f32>;
 if e.logistics.w>3.5 {shares=food_worker_shares(e,pop,available_workers);} else {shares=baseline_worker_shares(e,pop,available_workers);}
 if e.fishery.w>.5 {
  // Helpers reserve legacy fishing only for legacy sites. An idle adaptive
  // fishery must preserve exactly the same ordinary shares as a closed one.
  let used=clamp((e.fishery_plan.y+e.fishery_plan.z)/max(available_workers,.001),0.,.25);
  if used<=0.{return shares;}
  let reserved_craft=min(shares.w,e.exchange.w/max(available_workers,.001));
  shares.w-=reserved_craft;
  shares*=max(0.,1.-used-reserved_craft)/max(1.-reserved_craft,.001);
  shares.w+=reserved_craft;
 }
 return shares;
}
fn baseline_worker_shares(e:Economy,pop:f32,available_workers:f32)->vec4<f32>{
 // Explicit diagnostic ablation; never creates workers or bypasses resource costs.
 if e.logistics.w>2.5 {return vec4(.62,.08-select(0.,.001,e.management.x>.5&&e.management.y>=1.&&e.fishery.w<.5),.1,.2);}

 if e.logistics.w>1.5 {
 let workforce=max(available_workers,.001);
 let fish=select(0.,.001,e.management.x>.5&&e.management.y>=1.&&e.fishery.w<.5);
 // Forecast only work that can use existing inputs or this month's finite
 // extraction. Cargo suppresses orders in the CPU planner; it is not stock here.
 var stock=e.goods;
 let wood=min(order_room(e,0u),min(e.forest.x/.5,min(e.forest.y/.002,e.forest.z/.0002)));
 let wood_rate=extraction_rate(e,0u);let forestry=min(workforce*.12,wood/wood_rate);
 let room=max(0.,e.logistics.x-e.logistics.y-dry_stock(e)-forestry*wood_rate);
 let ore=min(room,min(order_room(e,ore_good(e)),e.reserves.y));let clay=min(max(0.,room-min(ore,workforce*.16*5.)),min(order_room(e,4u),e.reserves.z));
 let mining=min(workforce*.16,(ore+clay)/5.);
 stock[0].x+=forestry*wood_rate;
 var extraction=mining;
 for(var k=0u;k<2u;k++){let mineral=(k+p.dims.z)%2u;let good=select(ore_good(e),4u,mineral==1u);let rate=extraction_rate(e,mineral+1u);let mined=min(extraction*rate,select(ore,clay,mineral==1u));stock[good/4u][good%4u]+=mined;extraction=max(0.,extraction-mined/rate);}
 var craft=0.;var residue_held=e.residue.x;
 for(var step=0u;step<p.options.y;step++){
 let r=(step+p.dims.z)%p.options.y;let recipe=catalog.recipes[r];
 if recipe.work.y>0.&&(u32(e.management.w)&(1u<<u32(recipe.work.y-1.)))==0u{continue;}
 var batches=e.orders[r/4u][r%4u];
 for(var k=0u;k<64u;k++){let input=recipe.input[k/4u][k%4u];if input>0.{batches=min(batches,stock[k/4u][k%4u]/input);}}
 var stored=0.;var expansion=0.;
 for(var k=0u;k<63u;k++){
 let output=recipe.output[k/4u][k%4u];if output>0.{batches=min(batches,max(0.,e.targets[k/4u][k%4u]-stock[k/4u][k%4u])/output);}
 if catalog.goods[k].w<=0.{stored+=stock[k/4u][k%4u];expansion+=output-recipe.input[k/4u][k%4u];}}
 if expansion>0.{batches=min(batches,max(0.,e.logistics.x-e.logistics.y-stored)/expansion);}
 // Reserve shared inputs once; later jobs can use completed intermediate goods.
 if e.extraction.y>.5 && recipe.work.w>0. {batches=min(batches,max(0.,e.residue.z-residue_held)/recipe.work.w);residue_held+=batches*recipe.work.w;}
 for(var k=0u;k<64u;k++){stock[k/4u][k%4u]=max(0.,stock[k/4u][k%4u]-recipe.input[k/4u][k%4u]*batches)+recipe.output[k/4u][k%4u]*batches;}
 craft+=batches*recipe.work.x;
 }
 // Operating infrastructure is recurring work even when recipe orders are empty.
 // Reserve actual operating work; construction separately uses its 10% allowance.
 let service_craft=select(0.,min(pop,min(e.waterworks.x/2.,e.waterworks.y/4.))*.001,e.waterworks.w>.5);
 let reserved=min(e.exchange.w+service_craft,workforce*.2);
 var demand=vec3(forestry,mining,min(workforce*.30,craft)+reserved)/workforce;
 let discretionary=max(0.,.38-fish-reserved/workforce);
 demand.z=max(0.,demand.z-reserved/workforce);
 demand*=min(1.,discretionary/max(demand.x+demand.y+demand.z,.00001));demand.z+=reserved/workforce;
 let desired=vec4(1.-fish-demand.x-demand.y-demand.z,demand);
 var old=vec4(.62,.08,.1,.2);let previous=dot(e.labor,vec4(1.));if previous>0.{old=e.labor/previous*(1.-fish);}
 // Gradual reassignment retains skills and prevents monthly occupation swings.
 var shares=mix(old,desired,.25);shares.x=clamp(shares.x,.62,1.-fish);
 let nonfarm=shares.y+shares.z+shares.w;
 shares=vec4(shares.x,shares.yzw*min(1.,max(0.,1.-fish-shares.x)/max(nonfarm,.00001)));
 // Previously promised research/cultural labor is protected within the craft pool.
 let needed=max(0.,reserved/workforce-shares.w);let transfer=min(needed,max(0.,shares.x-.62));shares.x-=transfer;shares.w+=transfer;
 return shares;
 }

 var shares=vec4(.62,.08,.1,.2);
 if e.reserves.y<1.{shares.x+=.06;shares.z-=.06;}
 if e.logistics.w<.5 && e.reserves.y>1000.&&e.goods[0].y<pop*2.&&e.goods[0].w>pop*.3{shares.x-=.12;shares.z+=.12;}
 if e.goods[0].w<pop*.3 && (e.reserves.y>0.||e.goods[0].y+e.goods[0].z+e.goods[7].y>0.){shares.x-=.08;shares.w+=.08;}
 // Reserve 0.1% of workers for a shore fishery (20 kg per worker-month).
 if e.management.x>.5 && e.management.y>=1. && e.fishery.w<.5{shares.y-=.001;}
 return shares;
}


fn food_worker_shares(e:Economy,pop:f32,available_workers:f32)->vec4<f32>{
 // Explicit diagnostic ablation; never creates workers or bypasses resource costs.
 if e.logistics.w>2.5 && e.logistics.w<3.5 {return vec4(.62,.08-select(0.,.001,e.management.x>.5&&e.management.y>=1.&&e.fishery.w<.5),.1,.2);}

 if e.logistics.w>1.5 {
 let workforce=max(available_workers,.001);
 let fish=select(0.,.001,e.management.x>.5&&e.management.y>=1.&&e.fishery.w<.5);
 // Forecast only work that can use existing inputs or this month's finite
 // extraction. Cargo suppresses orders in the CPU planner; it is not stock here.
 var stock=e.goods;
 let wood=min(order_room(e,0u),min(e.forest.x/.5,min(e.forest.y/.002,e.forest.z/.0002)));
 let wood_rate=extraction_rate(e,0u);let forestry=min(workforce*.12,wood/wood_rate);
 let room=max(0.,e.logistics.x-e.logistics.y-dry_stock(e)-forestry*wood_rate);
 let ore=min(room,min(order_room(e,ore_good(e)),e.reserves.y));let clay=min(max(0.,room-min(ore,workforce*.16*5.)),min(order_room(e,4u),e.reserves.z));
 let mining=min(workforce*.16,(ore+clay)/5.);
 stock[0].x+=forestry*wood_rate;
 var extraction=mining;
 for(var k=0u;k<2u;k++){let mineral=(k+p.dims.z)%2u;let good=select(ore_good(e),4u,mineral==1u);let rate=extraction_rate(e,mineral+1u);let mined=min(extraction*rate,select(ore,clay,mineral==1u));stock[good/4u][good%4u]+=mined;extraction=max(0.,extraction-mined/rate);}
 var craft=0.;var tool_work=0.;var residue_held=e.residue.x;
 for(var step=0u;step<p.options.y;step++){
 let r=(step+p.dims.z)%p.options.y;let recipe=catalog.recipes[r];
 if recipe.work.y>0.&&(u32(e.management.w)&(1u<<u32(recipe.work.y-1.)))==0u{continue;}
 var batches=e.orders[r/4u][r%4u];
 for(var k=0u;k<64u;k++){let input=recipe.input[k/4u][k%4u];if input>0.{batches=min(batches,stock[k/4u][k%4u]/input);}}
 var stored=0.;var expansion=0.;
 for(var k=0u;k<63u;k++){
 let output=recipe.output[k/4u][k%4u];if output>0.{batches=min(batches,max(0.,e.targets[k/4u][k%4u]-stock[k/4u][k%4u])/output);}
 if catalog.goods[k].w<=0.{stored+=stock[k/4u][k%4u];expansion+=output-recipe.input[k/4u][k%4u];}}
 if expansion>0.{batches=min(batches,max(0.,e.logistics.x-e.logistics.y-stored)/expansion);}
 // Reserve shared inputs once; later jobs can use completed intermediate goods.
 if e.extraction.y>.5 && recipe.work.w>0. {batches=min(batches,max(0.,e.residue.z-residue_held)/recipe.work.w);residue_held+=batches*recipe.work.w;}
 for(var k=0u;k<64u;k++){stock[k/4u][k%4u]=max(0.,stock[k/4u][k%4u]-recipe.input[k/4u][k%4u]*batches)+recipe.output[k/4u][k%4u]*batches;}
 craft+=batches*recipe.work.x;
 if recipe.output[0].z+recipe.output[0].w+recipe.output[9].w+dot(recipe.output[10],vec4(1.))>0. {tool_work+=batches*recipe.work.x;}
 }
 // Operating infrastructure is recurring work even when recipe orders are empty.
 // Reserve actual operating work; construction separately uses its 10% allowance.
 let service_craft=select(0.,min(pop,min(e.waterworks.x/2.,e.waterworks.y/4.))*.001,e.waterworks.w>.5);
 let reserved=min(e.exchange.w+service_craft,workforce*.2);
 var demand=vec3(forestry,mining,min(workforce*.30,craft)+reserved)/workforce;
 let food_policy=e.logistics.w>3.5;
 let pressure=select(0.,mix(e.food_labor.x,e.food_labor.y,.25),food_policy);
 let farm_floor=.62+.20*pressure;
 // Preserve at most 8% of finite workers for feasible industry when tools are scarce.
 // This protects capacity, not output; ordinary recipe inputs, orders and costs still apply.
 let feasible=max(vec3(0.),demand-vec3(0.,0.,reserved/workforce));
 let maintenance_feasible=vec3(select(0.,forestry/workforce,tool_work>0.||ore>0.),min(mining,ore/5.)/workforce,min(tool_work/workforce,feasible.z));
 var maintenance=maintenance_feasible*min(1.,select(0.,.08*e.food_labor.w,food_policy)/max(dot(maintenance_feasible,vec3(1.)),.00001));
 if e.logistics.w>4.5 {maintenance=vec3(0.);}
 let discretionary=max(0.,select(.38,1.-farm_floor,food_policy)-fish-reserved/workforce);
 demand.z=max(0.,demand.z-reserved/workforce);
 demand*=min(1.,discretionary/max(demand.x+demand.y+demand.z,.00001));demand.z+=reserved/workforce;
 let desired=vec4(1.-fish-demand.x-demand.y-demand.z,demand);
 var old=vec4(.62,.08,.1,.2);let previous=dot(e.labor,vec4(1.));if previous>0.{old=e.labor/previous*(1.-fish);}
 // Gradual reassignment retains skills and prevents monthly occupation swings.
 var shares=mix(old,desired,.25);shares.x=clamp(shares.x,farm_floor,1.-fish);
 let nonfarm=shares.y+shares.z+shares.w;
 shares=vec4(shares.x,shares.yzw*min(1.,max(0.,1.-fish-shares.x)/max(nonfarm,.00001)));
 // Previously promised research/cultural labor is protected within the craft pool.
 if food_policy {
  let minimum=maintenance+vec3(0.,0.,reserved/workforce);
  let deficit=max(vec3(0.),minimum-shares.yzw);
  let moved=deficit*min(1.,max(0.,shares.x-.62)/max(dot(deficit,vec3(1.)),.00001));
  shares=vec4(shares.x-dot(moved,vec3(1.)),shares.yzw+moved);
 }
 let needed=max(0.,reserved/workforce-shares.w);let transfer=min(needed,max(0.,shares.x-.62));shares.x-=transfer;shares.w+=transfer;
 return shares;
 }

 var shares=vec4(.62,.08,.1,.2);
 if e.reserves.y<1.{shares.x+=.06;shares.z-=.06;}
 if e.logistics.w<.5 && e.reserves.y>1000.&&e.goods[0].y<pop*2.&&e.goods[0].w>pop*.3{shares.x-=.12;shares.z+=.12;}
 if e.goods[0].w<pop*.3 && (e.reserves.y>0.||e.goods[0].y+e.goods[0].z+e.goods[7].y>0.){shares.x-=.08;shares.w+=.08;}
 // Reserve 0.1% of workers for a shore fishery (20 kg per worker-month).
 if e.management.x>.5 && e.management.y>=1. && e.fishery.w<.5{shares.y-=.001;}
 return shares;
}

fn ore_good(e:Economy)->u32 {return select(1u,u32(e.extraction.x),e.extraction.x>=1.);}

// Capability is task-specific. Labor improves access rate, never source inventory.
fn extraction_rate(e:Economy,task:u32)->f32 {
 let base=select(5.,20.,task==0u);if catalog.methods[0].x==0. {return base;}
 let role=select(select(4.,2.,task==2u),3.,task==0u);
 let workers=max(1.,select(e.labor.z,e.labor.y,task==0u));
 var tools=e.goods[0].w*.25;
 if e.extraction.y>.5 {tools+=e.goods[10].y*.25+e.goods[10].w*.15;}
 for(var j=0u;j<6u;j++){if catalog.methods[j].x==role {let k=j+45u;tools+=e.goods[k/4u][k%4u]*catalog.methods[j].y;}}
 let capability=clamp(tools/workers,0.,2.);
 let difficulty=select(select(max(.1,e.extraction.z)+e.extraction.w*2.,.2,task==2u),.4+clamp(e.forest.x/max(e.claim.y,1.),0.,1.),task==0u);
 return base/(1.+difficulty/(.2+2.*capability));
}

fn container_service(e:Economy)->f32 {
 var vessels=e.goods[1].w;
 for(var j=0u;j<6u;j++){if catalog.methods[j].x==1. {let k=j+45u;vessels+=e.goods[k/4u][k%4u]*catalog.methods[j].y;}}
 return vessels;
}
