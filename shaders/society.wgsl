// Monthly population, ration and crop-calendar parameters.
const NUTRITION_DECLINE_MONTHS:f32=6.;
const NUTRITION_RECOVERY_MONTHS:f32=3.;
const RATION_COMMON_SUFFICIENCY_SHARE:f32=.5;
const RATION_REDISTRIBUTION_PASSES:u32=3u;
const DEMOGRAPHY_DIVISION_FLOOR:f32=1e-20;
const WATERLOGGING_MONTHLY_ILLNESS:f32=.004;
const CROWDING_MONTHLY_ILLNESS:f32=.001;
const SANITATION_MAX_ILLNESS_REDUCTION:f32=.75;
const UNSAFE_WATER_MONTHLY_ILLNESS:f32=.003;
const MONTHLY_ILLNESS_RETENTION:f32=.95;
const SHORTAGE_MONTHLY_ILLNESS:f32=.02;
// Shared by the civilization shader's current-weather path.
const WEATHER_REFERENCE_RAIN_FLOOR:f32=.001;
const MAX_REGIONAL_WEATHER_RATIO:f32=10.;
const SHORTAGE_DURATION_THRESHOLD:f32=.02;
const PLANTING_LEAD_MONTHS:u32=6u;
const HARVEST_SEED_RESERVE_SHARE:f32=.05;
const EXPOSURE_POPULATION_FLOOR:f32=1.;
const SEED_RATIO_POPULATION_FLOOR:f32=1.;

struct Demography {ages:vec4<f32>, crops:vec4<f32>, health:vec4<f32>, ration_priority:vec4<f32>, ration_need:vec4<f32>, ration_eaten:vec4<f32>, household_food:vec4<f32>, nutrition:vec4<f32>}
@group(0) @binding(8) var<storage,read_write> demography:array<Demography>;
// Opening-month illness reduces effective work, not population. The burden is
// an abstract index: at its ordinary cap of 0.5, work is reduced by 25%.
fn workers(i:u32,pop:f32)->f32 {
 if (p.options.w&1u)==0u {return pop*LEGACY_WORKER_SHARE;}
 return demography[i].ages.y*ADULT_WORKER_MONTHS*(1.-ILLNESS_WORK_PENALTY*clamp(demography[i].health.x,0.,MAX_WORK_ILLNESS_BURDEN));
}
// All cohorts receive half the common sufficiency first. Remaining food is
// weighted by entitlement, capped at need, and redistributed in at most 3 passes.
fn allocate_rations(need:vec3<f32>, food:f32, priority:vec3<f32>)->vec3<f32> {
 let total=dot(need,vec3(1.));
 if total<=0. {return vec3(0.);}
 let available=min(max(food,0.),total);
 if available>=total {return need;}
 if all(priority==vec3(0.)) {return need*(available/total);}
 var eaten=need*(RATION_COMMON_SUFFICIENCY_SHARE*available/total);
 var remaining=available-dot(eaten,vec3(1.));
 for(var iteration=0u;iteration<RATION_REDISTRIBUTION_PASSES;iteration++) {
  let unmet=max(need-eaten,vec3(0.));
  let weight=unmet*(vec3(1.)+clamp(priority,vec3(0.),vec3(MAX_RATION_PRIORITY)));
  let amount=min(unmet,weight*(remaining/max(dot(weight,vec3(1.)),DEMOGRAPHY_DIVISION_FLOOR)));
  eaten+=amount;remaining=max(0.,remaining-dot(amount,vec3(1.)));
 }
 return eaten;
}
fn demographic_month(i:u32,shortage:f32,food:f32)->vec2<f32> {
 var d=demography[i];let old=d.ages.xyz;
 let need=old*vec3(CHILD_RATION_KG_PER_MONTH,ADULT_RATION_KG_PER_MONTH,ELDER_RATION_KG_PER_MONTH);
 let eaten=allocate_rations(need,food,d.ration_priority.xyz);
 let hunger=select(vec3(0.),clamp(vec3(1.)-eaten/max(need,vec3(DEMOGRAPHY_DIVISION_FLOOR)),vec3(0.),vec3(1.)),need>vec3(0.));
 d.ration_need=vec4(need,dot(need,vec3(1.)));
 d.ration_eaten=vec4(eaten,dot(eaten,vec3(1.)));
 let mature=old.x/CHILD_COHORT_MONTHS;let retire=old.y/ADULT_COHORT_MONTHS;
 // Waterlogged settlements accumulate illness even when relief prevents hunger.
 // Recovery follows the environmental cleanup clock; remedies can meet this need.
 var contamination=select(0.,economies[i].soil.w*WATERLOGGING_MONTHLY_ILLNESS,(p.options.w&2u)!=0u);
 let e=economies[i];
 if e.waterworks.w>.5 {
  let crowding=select(0.,clamp(1.-(e.housing.z+min(e.housing.x/HOUSING_WOOD_KG_PER_PERSON,e.housing.y/HOUSING_BRICKS_KG_PER_PERSON))/max(dot(old,vec3(1.)),EXPOSURE_POPULATION_FLOOR),0.,1.),e.housing_plan.w>.5);
  contamination=(contamination+CROWDING_MONTHLY_ILLNESS*crowding)*(1.-SANITATION_MAX_ILLNESS_REDUCTION*e.waterworks_plan.w)+UNSAFE_WATER_MONTHLY_ILLNESS*e.water_service.y;
 }
 var mortality_hunger=hunger;
 var illness_shortage=shortage;
 if d.nutrition.w>.5 {
  let previous=d.nutrition.xyz;
  let timescale=select(vec3(NUTRITION_DECLINE_MONTHS),vec3(NUTRITION_RECOVERY_MONTHS),hunger<previous);
  let memory=clamp(previous+(hunger-previous)/timescale,vec3(0.),vec3(1.));
  d.nutrition=vec4(memory,1.);
  mortality_hunger=max(memory*memory,max(vec3(0.),(hunger-vec3(ACUTE_HUNGER_THRESHOLD))*ACUTE_HUNGER_SCALE));
  illness_shortage=dot(mortality_hunger,need)/max(dot(need,vec3(1.)),DEMOGRAPHY_DIVISION_FLOOR);
 }
 let disease=clamp(d.health.x*MONTHLY_ILLNESS_RETENTION+illness_shortage*SHORTAGE_MONTHLY_ILLNESS+contamination,0.,MAX_DISEASE_BURDEN);
 let losses=old*min(vec3(MAX_MONTHLY_MORTALITY),vec3(CHILD_BASE_MONTHLY_MORTALITY,ADULT_BASE_MONTHLY_MORTALITY,ELDER_BASE_MONTHLY_MORTALITY)+mortality_hunger*vec3(CHILD_HUNGER_MORTALITY,ADULT_HUNGER_MORTALITY,ELDER_HUNGER_MORTALITY)+vec3(disease*DISEASE_MORTALITY));
 let born=old.y*MONTHLY_BIRTH_RATE_PER_ADULT*(1.-hunger.y)*(1.-disease);
 var weather=regional_weather(u32(src[i].habitat.z));if (p.options.w&2u)!=0u {let c=world[u32(src[i].habitat.z)];weather=clamp(c.climate.y/max(c.hydro.z,WEATHER_REFERENCE_RAIN_FLOOR),0.,MAX_REGIONAL_WEATHER_RATIO);}
 // Identity-owned mode retains GPU ration/weather/disease work, but commits
 // demographic transitions once on the CPU from actual resident identities.
 if (p.options.w&4u)!=0u {d.ages=vec4(old,weather);d.health.x=disease;d.health.y=select(0.,d.health.y+1.,shortage>SHORTAGE_DURATION_THRESHOLD);demography[i]=d;return vec2(0.);}
 d.ages=vec4(max(vec3(0.),old-losses+vec3(born-mature,mature-retire,retire)),weather);
 d.health.x=disease;d.health.y=select(0.,d.health.y+1.,shortage>SHORTAGE_DURATION_THRESHOLD);d.health.z+=dot(losses,vec3(1.));demography[i]=d;
 return vec2(born,dot(losses,vec3(1.)));
}
fn crop_calendar(i:u32,growth:f32)->f32 {
 var d=demography[i];d.crops.x+=growth;d.health.w=growth;var harvested=0.;
 let season=p.dims.z%CROP_CALENDAR_MONTHS;let harvest=u32(d.crops.z);
 if season==harvest {harvested=d.crops.x*farm_attendance(economies[i]);d.crops.x-=harvested;let reserved=min(harvested*HARVEST_SEED_RESERVE_SHARE,src[i].stock.x);d.crops.y+=reserved;harvested-=reserved;}
 if season==(harvest+PLANTING_LEAD_MONTHS)%CROP_CALENDAR_MONTHS {
  let planted=min(d.crops.y,src[i].stock.x)*farm_attendance(economies[i]);d.crops.y-=planted;d.crops.w=clamp(planted/max(SEED_RATIO_POPULATION_FLOOR,src[i].stock.x),0.,1.);
  // Seed remains living crop inventory; growth converts it to standing biomass without creating matter.
  d.crops.x+=planted;
 }
 demography[i]=d;return harvested;
}
