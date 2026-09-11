struct Demography {ages:vec4<f32>, crops:vec4<f32>, health:vec4<f32>, ration_priority:vec4<f32>, ration_need:vec4<f32>, ration_eaten:vec4<f32>, household_food:vec4<f32>}
@group(0) @binding(8) var<storage,read_write> demography:array<Demography>;
// Opening-month illness reduces effective work, not population. The burden is
// an abstract index: at its ordinary cap of 0.5, work is reduced by 25%.
fn workers(i:u32,pop:f32)->f32 {
 if (p.options.w&1u)==0u {return pop*.5;}
 return demography[i].ages.y*.8*(1.-.5*clamp(demography[i].health.x,0.,.5));
}
// All cohorts receive half the common sufficiency first. Remaining food is
// weighted by entitlement, capped at need, and redistributed in at most 3 passes.
fn allocate_rations(need:vec3<f32>, food:f32, priority:vec3<f32>)->vec3<f32> {
 let total=dot(need,vec3(1.));
 if total<=0. {return vec3(0.);}
 let available=min(max(food,0.),total);
 if available>=total {return need;}
 if all(priority==vec3(0.)) {return need*(available/total);}
 var eaten=need*(.5*available/total);
 var remaining=available-dot(eaten,vec3(1.));
 for(var iteration=0u;iteration<3u;iteration++) {
  let unmet=max(need-eaten,vec3(0.));
  let weight=unmet*(vec3(1.)+clamp(priority,vec3(0.),vec3(3.)));
  let amount=min(unmet,weight*(remaining/max(dot(weight,vec3(1.)),1e-20)));
  eaten+=amount;remaining=max(0.,remaining-dot(amount,vec3(1.)));
 }
 return eaten;
}
fn demographic_month(i:u32,shortage:f32,food:f32)->vec2<f32> {
 var d=demography[i];let old=d.ages.xyz;
 let need=old*vec3(10.,18.,14.);
 let eaten=allocate_rations(need,food,d.ration_priority.xyz);
 let hunger=select(vec3(0.),clamp(vec3(1.)-eaten/max(need,vec3(1e-20)),vec3(0.),vec3(1.)),need>vec3(0.));
 d.ration_need=vec4(need,dot(need,vec3(1.)));
 d.ration_eaten=vec4(eaten,dot(eaten,vec3(1.)));
 let mature=old.x/180.;let retire=old.y/540.;
 // Waterlogged settlements accumulate illness even when relief prevents hunger.
 // Recovery follows the environmental cleanup clock; remedies can meet this need.
 var contamination=select(0.,economies[i].soil.w*.004,(p.options.w&2u)!=0u);
 let e=economies[i];
 if e.waterworks.w>.5 {
  let crowding=select(0.,clamp(1.-(e.housing.z+min(e.housing.x/2.,e.housing.y/3.))/max(dot(old,vec3(1.)),1.),0.,1.),e.housing_plan.w>.5);
  contamination=(contamination+.001*crowding)*(1.-.75*e.waterworks_plan.w)+.003*e.water_service.y;
 }
 let disease=clamp(d.health.x*.95+shortage*.02+contamination,0.,.5);
 let losses=old*min(vec3(.9),vec3(.0005,.0006,.003)+hunger*vec3(.06,.025,.05)+vec3(disease*.01));
 let born=old.y*.004*(1.-hunger.y)*(1.-disease);
 var weather=regional_weather(u32(src[i].habitat.z));if (p.options.w&2u)!=0u {let c=world[u32(src[i].habitat.z)];weather=clamp(c.climate.y/max(c.hydro.z,.001),0.,10.);}
 // Identity-owned mode retains GPU ration/weather/disease work, but commits
 // demographic transitions once on the CPU from actual resident identities.
 if (p.options.w&4u)!=0u {d.ages=vec4(old,weather);d.health.x=disease;d.health.y=select(0.,d.health.y+1.,shortage>.02);demography[i]=d;return vec2(0.);}
 d.ages=vec4(max(vec3(0.),old-losses+vec3(born-mature,mature-retire,retire)),weather);
 d.health.x=disease;d.health.y=select(0.,d.health.y+1.,shortage>.02);d.health.z+=dot(losses,vec3(1.));demography[i]=d;
 return vec2(born,dot(losses,vec3(1.)));
}
fn crop_calendar(i:u32,growth:f32)->f32 {
 var d=demography[i];d.crops.x+=growth;d.health.w=growth;var harvested=0.;
 let season=p.dims.z%12u;let harvest=u32(d.crops.z);
 if season==harvest {harvested=d.crops.x*farm_attendance(economies[i]);d.crops.x-=harvested;let reserved=min(harvested*.05,src[i].stock.x);d.crops.y+=reserved;harvested-=reserved;}
 if season==(harvest+6u)%12u {
  let planted=min(d.crops.y,src[i].stock.x)*farm_attendance(economies[i]);d.crops.y-=planted;d.crops.w=clamp(planted/max(1.,src[i].stock.x),0.,1.);
  // Seed remains living crop inventory; growth converts it to standing biomass without creating matter.
  d.crops.x+=planted;
 }
 demography[i]=d;return harvested;
}
