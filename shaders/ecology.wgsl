struct Eco { pools:array<vec4<f32>,41> }
struct Env { fields:array<vec4<f32>,25> }
struct Entry { a:vec4<f32>,b:vec4<f32>,c:vec4<f32>,d:vec4<f32>,ids:vec4<u32> }
struct Params {dims:vec4<u32>,physical:vec4<f32>,counts:vec4<u32>,options:vec4<u32>,event:vec4<u32>,storms:vec4<f32>,abundance:vec4<f32>,thermal:vec4<f32>}
@group(0) @binding(0) var<storage,read_write> terrain:array<Cell>;
@group(0) @binding(1) var<storage,read_write> environment:array<Env>;
@group(0) @binding(2) var<storage,read> src:array<Eco>;
@group(0) @binding(3) var<storage,read_write> dst:array<Eco>;
@group(0) @binding(4) var<storage,read> catalog:array<Entry>;
@group(0) @binding(5) var<storage,read> river:array<vec4<f32>>;
@group(0) @binding(6) var<storage,read_write> river_out:array<vec4<f32>>;
@group(0) @binding(7) var<uniform> p:Params;
const NONE:u32=0xffffffffu;
fn id(g:vec3<u32>,n:u32)->u32 {return g.z*n*n+g.y*n+g.x;}
fn direction(f:u32,u:f32,v:f32)->vec3<f32> {var d=vec3(u,v,-1.);switch f {case 0u:{d=vec3(1.,u,v);}case 1u:{d=vec3(-1.,u,v);}case 2u:{d=vec3(u,1.,v);}case 3u:{d=vec3(u,-1.,v);}case 4u:{d=vec3(u,v,1.);}default:{}}return normalize(d);}
fn pos(i:u32,n:u32)->vec3<f32> {return direction(i/(n*n),2.*(f32(i%n)+.5)/f32(n)-1.,2.*(f32(i/n%n)+.5)/f32(n)-1.);}
fn index(d:vec3<f32>,n:u32)->u32 {let a=abs(d);var f=0u;var uv=vec2(0.);if a.x>=a.y&&a.x>=a.z {f=select(1u,0u,d.x>=0.);uv=d.yz/a.x;}else if a.y>=a.z {f=select(3u,2u,d.y>=0.);uv=d.xz/a.y;}else {f=select(5u,4u,d.z>=0.);uv=d.xy/a.z;}let xy=vec2<u32>(clamp(floor((uv+1.)*.5*f32(n)),vec2(0.),vec2(f32(n-1u))));return f*n*n+xy.y*n+xy.x;}
fn neighbor(i:u32,k:u32,n:u32)->u32 {let offsets=array<vec2<i32>,4>(vec2(-1,0),vec2(1,0),vec2(0,-1),vec2(0,1));let xy=vec2<i32>(vec2<u32>(i%n,i/n%n))+offsets[k];if all(xy>=vec2(0))&&all(xy<vec2(i32(n))){return i/(n*n)*n*n+u32(xy.y)*n+u32(xy.x);}let uv=2.*(vec2<f32>(xy)+.5)/f32(n)-1.;return index(direction(i/(n*n),uv.x,uv.y),n);}
fn area(i:u32,n:u32)->f32 {let uv=2.*(vec2<f32>(vec2<u32>(i%n,i/n%n))+.5)/f32(n)-1.;let t=1./(f32(n)*sqrt(3.));var sum=0.;for(var k=0u;k<4u;k++){let q=uv+vec2(select(-t,t,(k&1u)!=0u),select(-t,t,(k&2u)!=0u));sum+=pow(1.+dot(q,q),-1.5);}return sum/f32(n*n)*p.physical.x*p.physical.x*1e6;}
fn parent(i:u32)->u32 {let n=p.dims.x;let m=p.dims.y;let r=n/m;return i/(n*n)*m*m+(i/n%n)/r*m+(i%n)/r;}
fn fine(i:u32,x:u32,y:u32)->u32 {let n=p.dims.x;let m=p.dims.y;let r=n/m;return i/(m*m)*n*n+((i/m%m)*r+y)*n+(i%m)*r+x;}
fn plants_offset()->u32 {return p.counts.x+p.counts.y+p.counts.z;}
fn guild_offset()->u32 {return plants_offset()+p.counts.w+p.options.x;}
fn microbial(j:u32)->vec2<f32> {if p.options.z==0u {return vec2(.4,.5);}for(var k=0u;k<p.options.z;k++){let t=catalog[guild_offset()+p.options.y+k];if t.ids.x==j {return t.a.xy;}}return vec2(0.);}
fn season_month()->u32 {return select(p.dims.z,max(1u,p.event.w)-1u,p.options.w==1u);}
fn land(e:Env)->f32 {return e.fields[0].z+e.fields[0].w;}
fn water(e:Env)->f32 {return e.fields[0].x+e.fields[0].y;}
fn region(e:Env)->u32 {var r=0u;for(var k=1u;k<4u;k++){if e.fields[0][k]>e.fields[0][r] {r=k;}}return r;}
@compute @workgroup_size(8,8)
fn aggregate(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=id(g,p.dims.y);let r=p.dims.x/p.dims.y;var e:Env;var a=0.;var la=0.;var wa=0.;var balance=0.;var stored=0.;var wind=0.;var temperature=0.;var rainfall=0.;
 for(var y=0u;y<r;y++){for(var x=0u;x<r;x++){let j=fine(i,x,y);let c=terrain[j];let v=area(j,p.dims.x);a+=v;e.fields[0][c.tags.x]+=v;balance+=(c.budget.x-c.budget.y)*v;stored+=(c.water.x+c.water.y+c.water.z)*v+river[j].w;wind+=c.climate.w*v;temperature+=select(c.hydro.y,c.climate.x,p.options.w==1u)*v;rainfall+=select(c.hydro.z,c.climate.y,p.options.w==1u)*v;
 let rock=catalog[c.ids.x];let soil=catalog[p.counts.x+p.counts.y+c.ids.y];
 if c.tags.x>=2u {la+=v;e.fields[1]+=vec4(select(c.hydro.y,c.climate.x,p.options.w==1u),select(c.hydro.z,c.climate.y,p.options.w==1u),c.water.y,c.climate.w)*v;e.fields[2]+=vec4(c.geology.x,c.terrain.x,c.terrain.z,c.life.w)*v;var chemistry=rock.b;if c.tags.y<p.counts.y {chemistry.x=min(1.,chemistry.x+catalog[p.counts.x+c.tags.y].a.w*c.geology.z*.05);}if c.tags.x==2u {chemistry.x*=p.abundance.x;}e.fields[4]+=chemistry*v;e.fields[5]+=soil.b*v;e.fields[6][rock.ids.x]+=v;e.fields[6].w+=soil.a.x*v;
 // Nested geological habitats, evaluated before coarse-grid averaging. Rates are
 // weighted by reactive substrate, not by the maximum activity in a coarse cell.
 let activity=clamp(c.geology.x,0.,1.);let groundwater=clamp(c.water.y*2.,0.,1.);
 let wet=clamp(select(c.hydro.z,c.climate.y,p.options.w==1u)/1400.+c.water.y*.5,0.,1.);
 let viable=groundwater>.05&&chemistry.y>.001;
 let enriched=viable&&activity>=.2;let province=viable&&activity>=.55;let vent=viable&&activity>=.82;
 e.fields[8]+=vec4(f32(enriched),f32(province),f32(vent),0.)*v;
 e.fields[8].w=max(e.fields[8].w,activity);
 let rate=wet*(.02*activity+.5*pow(activity,4.))*(.2+.8*groundwater);
 let reactive=chemistry.y*rate*v;e.fields[9].x+=reactive;
 for(var h=0u;h<2u;h++){if select(enriched,province,h==1u) {
 let base=10u+3u*h;
 e.fields[9][h+1u]+=reactive;
 e.fields[base]+=vec4(select(c.hydro.y,c.climate.x,p.options.w==1u),select(c.hydro.z,c.climate.y,p.options.w==1u),c.water.y,soil.a.x)*v;
 e.fields[base+1u][rock.ids.x]+=v;e.fields[base+1u].w+=chemistry.w*v;
 e.fields[base+2u]+=vec4(activity,f32(c.tags.x==3u),0.,0.)*v;
 }}
 }
 else {wa+=v;e.fields[3].z+=max(c.water.x,-c.terrain.x)*v;}
 }}
 // Fine shared-edge habitat conductance, cached for monthly transport. Cube seams
 // use the same fine neighbor mapping as terrain; no coarse land/water shortcut.
 for(var d=0u;d<4u;d++){for(var q=0u;q<r;q++){
 let x=select(q,select(0u,r-1u,d==1u),d<2u);
 let y=select(select(0u,r-1u,d==3u),q,d<2u);
 let j=fine(i,x,y);let c=terrain[j];let b=terrain[neighbor(j,d,p.dims.x)];
 if c.tags.x>=2u&&b.tags.x>=2u {
 e.fields[22][d]+=1./(1.+abs(c.terrain.x-b.terrain.x)/1000.);
 }
 if c.tags.x<2u&&c.tags.x==b.tags.x {e.fields[23][d]+=1.;}
 let flowing=c.routing.x==neighbor(j,d,p.dims.x)||b.routing.x==j;
 let channel=(c.water.w>1.&&b.water.w>1.&&flowing);
 let mouth=(c.tags.x<2u&&b.water.w>1.&&b.routing.x==j)||(b.tags.x<2u&&c.water.w>1.&&c.routing.x==neighbor(j,d,p.dims.x));
 if channel||mouth||(c.tags.x<2u&&c.tags.x==b.tags.x) {e.fields[24][d]+=1.;}
 }e.fields[22][d]/=f32(r);e.fields[23][d]/=f32(r);e.fields[24][d]/=f32(r);}
 for(var h=0u;h<2u;h++){let habitat_area=e.fields[8][h];for(var f=0u;f<3u;f++){e.fields[10u+3u*h+f]/=max(habitat_area,1.);}}
 e.fields[8]=vec4(e.fields[8].xyz/a,e.fields[8].w);e.fields[9]/=max(la,1.);
 e.fields[0]/=a;e.fields[1]/=max(la,1.);e.fields[2]/=max(la,1.);e.fields[4]/=max(la,1.);e.fields[5]/=max(la,1.);e.fields[6]/=max(la,1.);
 e.fields[3]=vec4(a,e.fields[2].y,e.fields[3].z/max(wa,1.),pos(i,p.dims.y).y);if la<=0. {e.fields[1].x=temperature/a;e.fields[1].y=rainfall/a;}e.fields[2].z=balance/a;e.fields[1].w=wind/a;e.fields[7]=vec4(pos(i,p.dims.y),stored/a);environment[i]=e;
}
fn total(s:Eco)->vec3<f32> {var t=vec3(0.);for(var k=0u;k<26u;k++){t+=s.pools[k].xyz;}return t;}
// A compact inherited regional trait, not a species identity or nutrient stock.
fn ecotypes_enabled()->bool {return (u32(p.abundance.z)&2u)!=0u;}
fn thermal_preference(s:Eco,k:u32)->f32 {return s.pools[38u+k/4u][k%4u];}
fn thermal_match(encoded:f32,temperature:f32,aquatic:bool)->f32 {
 if !ecotypes_enabled()||encoded==0. {return 1.;}
 let mismatch=(temperature-(encoded-81.))/select(15.,p.thermal.x,aquatic);
 return 1./(1.+mismatch*mismatch);
}
fn founder_preference(i:u32,k:u32,e:Env)->f32 {
 let phase=f32(p.dims.w%997u)*.017+f32(k)*2.399963;
 return 81.+clamp(e.fields[1].x+8.*sin(dot(pos(i,p.dims.y),vec3(3.,5.,7.))+phase),-80.,60.);
}
@compute @workgroup_size(8,8)
fn seed_ecology(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=id(g,p.dims.y);let e=environment[i];let l=land(e);let w=water(e);var s:Eco;
 s.pools[17]=vec4(.1, .02,.004,0.)*l;s.pools[18]=vec4(.4,.02,.002,0.)*l;
 let nutrient_fraction=(e.fields[0].z*p.abundance.x+e.fields[0].w)/max(l,.00001);
 s.pools[17].z*=nutrient_fraction;s.pools[18].z*=nutrient_fraction;
 s.pools[20]=vec4(.001,.0002,.00002,0.)*w;
 s.pools[21]=vec4(.05,.02,.003,0.)*w*min(e.fields[3].z/100.,60.);
 s.pools[22]=vec4(.2,.03,.02,0.)*w;
 s.pools[25]=vec4(0.,0.,max(.1,e.fields[4].x*10000.)*l,0.);
 s.pools[26]=vec4(.001*l, .002*l,10000.*e.fields[4].y*l,1.*l);
 // Explicit initial inventories include seed biomass; all subsequent growth consumes stocks.
 for(var k=0u;k<3u;k++){s.pools[k]=vec4(.02,.0005,.00004,0.)*l;}
 s.pools[23]=vec4(.0001,.000003,.0000003,0.)*w;
 // Explicit pre-human founder inventory; smooth occupied patches, not monthly
 // spontaneous recruitment. W records Ancient World founder ancestry share.
 for(var k=0u;k<p.options.y;k++){
 let t=catalog[guild_offset()+k];let habitat=select(l,w,t.ids.y==1u);
 let phase=f32(p.dims.w%997u)*.017+f32(k)*2.399963;
 let occupied_region=sin(dot(pos(i,p.dims.y),vec3(7.,11.,5.))+phase);
 if occupied_region>-.35&&habitat>0. {
 let carbon=habitat*.000001;
 if ecotypes_enabled() {s.pools[38u+k/4u][k%4u]=founder_preference(i,k,e);}
 s.pools[k+5u]=vec4(carbon,carbon*t.a.x,carbon*t.a.y,clamp(e.fields[0].w/max(l,.000001),0.,1.));
 }
 }
 s.pools[31]=vec4(1.,p.physical.z,1.,0.);
 s.pools[30]=vec4(total(s),e.fields[7].w);s.pools[25].w=e.fields[7].w;dst[i]=s;
}
@compute @workgroup_size(8,8)
fn replenish(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=id(g,p.dims.y);var s=src[i];let e=environment[i];
 // Fresh exposure is an explicit geological import, not an ecological fertility reset.
 let fresh=land(e)*e.fields[2].x*10.;let phosphorus=fresh*e.fields[4].x;
 s.pools[27].w+=e.fields[7].w-s.pools[25].w;s.pools[25].w=e.fields[7].w;s.pools[25].z+=phosphorus;s.pools[27].z+=phosphorus;s.pools[26].z+=fresh*e.fields[4].y;dst[i]=s;
}
// A conditional environment preserves narrow habitats without assigning their
// conditions to the rest of a coarse cell. Inventories remain whole-cell kg/m².
fn producer_environment(e:Env,k:u32)->Env {
 var habitat=e;if k>=3u&&k<5u {let base=10u+(k-3u)*3u;
 habitat.fields[1]=vec4(e.fields[base].xyz,e.fields[1].w);
 habitat.fields[6]=vec4(e.fields[base+1u].xyz,e.fields[base].w);
 habitat.fields[4].w=e.fields[base+1u].w;habitat.fields[2].x=e.fields[base+2u].x;
 habitat.fields[0].w=e.fields[base+2u].y;habitat.fields[0].z=1.-habitat.fields[0].w;
 }return habitat;
}
fn plant_score(e:Env,k:u32,j:u32)->f32 {
 if j>=p.counts.w {return 0.;}let t=catalog[plants_offset()+j];
 let outer=e.fields[0].w/max(land(e),.0001);
 if u32(t.c.x)!=k || (t.ids.x==1u&&outer<.5) {return 0.;}
 if e.fields[1].x<t.a.x||e.fields[1].x>t.a.y||e.fields[1].y<t.a.z||e.fields[1].y>t.a.w||e.fields[6].w<t.b.x {return 0.;}
 if t.ids.y<3u&&e.fields[6][t.ids.y]<.2 {return 0.;}
 let climate=clamp(1.-abs(e.fields[1].x-(t.a.x+t.a.y)*.5)/max(t.a.y-t.a.x,1.),.1,1.);
 return (t.b.y*(1.+t.d.x*e.fields[2].x)+t.d.z*.1)*climate;
}
fn select_except(e:Env,k:u32,excluded:u32)->u32 {
 var best=0.;var chosen=NONE;
 for(var j=0u;j<p.counts.w;j++){if j==excluded {continue;}let score=plant_score(e,k,j);if score>best {best=score;chosen=j;}}
 return chosen;
}
fn select_plant(e:Env,k:u32)->u32 {return select_except(e,k,NONE);}
// Diet access is local: terrestrial prey require land, aquatic prey require water.
fn food_access(e:Env,prey:u32)->f32 {
 if prey==13u||prey==14u||prey==15u||prey==22u||prey==23u {return min(1.,water(e)*4.);}
 return min(1.,land(e)*4.);
}
// A bounded replicator model changes composition inside one conserved layer pool.
// Shared production uses weighted traits, so competitors do not each receive a full energy budget.
fn community(e:Env,k:u32,old:vec4<f32>,energy:f32,biomass:f32,nutrients:vec2<f32>)->vec4<f32> {
 var first=select_plant(e,k);
 if old.w>0.&&plant_score(e,k,u32(old.x)-1u)>0. {first=u32(old.x)-1u;}
 var second=select_except(e,k,first);
 if old.w>0.&&u32(old.y)-1u!=first&&plant_score(e,k,u32(old.y)-1u)>0. {second=u32(old.y)-1u;}
 if p.abundance.y<.5 {second=NONE;}
 if second==NONE {return vec4(f32(first+1u),0.,1.,1.);}
 var fraction=select(.5,old.z,old.w>0.);
 if old.w>0.&&u32(old.x)!=first+1u {fraction=.01;}
 if old.w>0.&&u32(old.y)!=second+1u {fraction=.99;}
 let a=catalog[plants_offset()+first];let b=catalog[plants_offset()+second];
 let light=select(max(a.d.z,.05),1.,k==0u||k==5u);
 let other_light=select(max(b.d.z,.05),1.,k==0u||k==5u);
 let fit_a=min(energy*a.b.y*light,min(nutrients.x/a.c.y,nutrients.y/a.c.z)/p.physical.y)/max(biomass+.05,.05)-a.c.w-a.d.w*.15;
 let fit_b=min(energy*b.b.y*other_light,min(nutrients.x/b.c.y,nutrients.y/b.c.z)/p.physical.y)/max(biomass+.05,.05)-b.c.w-b.d.w*.15;
 fraction=clamp(fraction+p.physical.y*fraction*(1.-fraction)*clamp(fit_a-fit_b,-3.,3.),.0001,.9999);
 return vec4(f32(first+1u),f32(second+1u),fraction,1.);
}
fn mixture(state:vec4<f32>,fallback:Entry)->Entry {
 var a=fallback;if state.x>0. {a=catalog[plants_offset()+u32(state.x)-1u];}
 if state.y>0. {let b=catalog[plants_offset()+u32(state.y)-1u];let f=1.-state.z;
 a.a=mix(a.a,b.a,f);a.b=mix(a.b,b.b,f);a.c=mix(a.c,b.c,f);a.d=mix(a.d,b.d,f);}
 return a;
}
@compute @workgroup_size(8,8)
fn biology(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=id(g,p.dims.y);let e=environment[i];var s=src[i];let dt=p.physical.y;let l=max(0.,land(e)-select(0.,s.pools[24].w,p.options.w==1u));let w=water(e);let outer=e.fields[0].w/max(land(e),.00001);
 s.pools[19]*=.75; // Matching river injection, below, carries the other quarter as absolute mass.
 s.pools[27].w+=e.fields[2].z;s.pools[28]=vec4(0.);s.pools[29].w=0.;
 let wet=clamp(e.fields[1].y/1400.+e.fields[1].z*.5,0.,1.);
 let thermal=clamp(1.-abs(e.fields[1].x-23.)/40.,0.,1.);
 let sun=ecological_sunlight(e.fields[3].w,season_month(),p.abundance.w)*p.physical.w;
 let activity=e.fields[2].x*wet;
 let reaction_rate=e.fields[9].x/max(e.fields[4].y,.0000001);
 let reacted=min(s.pools[26].z,s.pools[26].z*reaction_rate*dt*s.pools[31].x);
 environment[i].fields[21]=vec4(reacted,0.,0.,1.);
 s.pools[26].z-=reacted;s.pools[26].x+=reacted*.002;s.pools[26].y+=reacted*.02;
 s.pools[26].w=min(l*2.,s.pools[26].w+wet*l*.02*dt);
 let released=min(s.pools[25].z,s.pools[25].z*(e.fields[4].z+activity*.003)*dt);
 s.pools[25].z-=released;s.pools[17].z+=released;
 // Decomposers respire carbon and return retained N/P to soil; losses enter groundwater.
 let decay=s.pools[18].xyz*min(.8,dt*(e.fields[5].w+outer)*thermal*wet*microbial(4u).y);
 s.pools[18]=vec4(s.pools[18].xyz-decay,0.);
 let retention=clamp(e.fields[5].x+outer*.15+microbial(7u).x*.1,0.,.99);
 s.pools[17]+=vec4(0.,decay.y*retention,decay.z*retention,0.);
 s.pools[19]+=vec4(0.,decay.yz*(1.-retention),0.);
 s.pools[27].x-=decay.x;s.pools[28].z+=decay.x;
 let leach=vec3(0.,s.pools[17].yz)*e.fields[5].z*wet*dt*(1.-retention);
 s.pools[17]-=vec4(leach,0.);s.pools[19]+=vec4(leach,0.);
 let sorbed=s.pools[17].z*e.fields[5].y*dt;s.pools[17].z-=sorbed;s.pools[24].z+=sorbed;
 let mobilized=s.pools[24].z*microbial(6u).y*wet*dt*.05;s.pools[24].z-=mobilized;s.pools[17].z+=mobilized;
 for(var k=0u;k<5u;k++) {
 let habitat=producer_environment(e,k);let old=s.pools[32u+k];
 let energy=select(sun*thermal*wet,s.pools[26].x*120.+s.pools[26].y*.01,k>=3u);
 var state=community(habitat,k,old,energy,s.pools[k].x/max(l,.001),s.pools[17].yz/max(l,.001));
 if k>=3u&&e.fields[8][k-3u]<=0. {state=vec4(0.,0.,1.,1.);}
 var changed=0.;if old.w>0. {if old.x!=state.x {changed+=old.z;}if old.y!=state.y {changed+=1.-old.z;}}
 let displaced=s.pools[k].xyz*clamp(changed,0.,1.);s.pools[k]-=vec4(displaced,0.);s.pools[18]+=vec4(displaced,0.);
 var plant=select(u32(state.y)-1u,u32(state.x)-1u,state.z>=.5);
 s.pools[32u+k]=select(vec4(0.),state,p.abundance.y>=.5);
 environment[i].fields[16u+k]=vec4(0.,0.,0.,3.);
 s.pools[k].w=select(0.,f32(plant+1u),plant!=NONE);
 var fallback:Entry;fallback.c=vec4(f32(k),.025,.002,.1);fallback.d=vec4(0.,0.,0.,.15);
 let t=mixture(state,fallback);
 let biomass=s.pools[k].x;
 let died=s.pools[k].xyz*min(1.,dt*(t.d.w*.15+select(.0,.5,plant==NONE)));
 s.pools[k]=vec4(s.pools[k].xyz-died,s.pools[k].w);s.pools[18]+=vec4(died,0.);
 let resp=min(s.pools[k].x,biomass*t.c.w*dt);s.pools[k].x-=resp;s.pools[27].x-=resp;s.pools[28].z+=resp;
 // Maintenance removes carbon; retranslocate nutrients no longer required by
 // living tissue instead of trapping them in increasingly carbon-poor biomass.
 let returned=max(vec2(0.),s.pools[k].yz-s.pools[k].x*t.c.yz);
 s.pools[k].y-=returned.x;s.pools[k].z-=returned.y;s.pools[17].y+=returned.x;s.pools[17].z+=returned.y;
 environment[i].fields[16u+k].z=died.x+resp;
 if plant==NONE||l<=0. {continue;}
 let shade=select(exp(-s.pools[0].x/max(l,.001)*.3)*max(t.d.z,.05),1.,k==0u);
 var photo=sun*thermal*wet*t.b.y*dt*l*shade*select(0.,1.,k<3u);
 // H2: 120 MJ/kg; yield 0.008 kg fixed C/MJ before symbiotic inefficiency.
 var access=.1;var local_wet=wet;
 if k>=3u {access=e.fields[9][k-2u]/max(e.fields[9].x,.0000001);local_wet=clamp(habitat.fields[1].z*2.,0.,1.);}
 let h=min(s.pools[26].x,s.pools[26].x*t.d.x*dt*access);
 var secondary=min(s.pools[26].y,s.pools[26].y*t.d.x*dt*.2*select(1.,access,k>=3u));
 let efficiency=.008*local_wet*min(1.,habitat.fields[4].w*50.);
 // The fictional H2/CO2 symbiosis fixes imported atmospheric carbon without
 // drawing on the aerobic oxidant pool. Secondary oxidative chemistry does.
 let hydrogen_carbon=h*120.*microbial(0u).x*efficiency;
 let secondary_potential=secondary*microbial(1u).x*efficiency;
 let secondary_carbon=min(secondary_potential,s.pools[26].w*.05);
 secondary*=secondary_carbon/max(secondary_potential,.00000001);
 let potential=hydrogen_carbon+secondary_potential;
 var chemo=hydrogen_carbon+secondary_carbon;
 environment[i].fields[21].z+=potential;
 let fix=min(s.pools[17].z*5.,(photo+chemo)*t.d.y*microbial(2u).x*.08);
 let fixed=min(fix,max(0.,(photo+chemo)*.1));
 // Nitrogen fixation costs 10 kg potential carbon per kg N in this game model.
 let cost=fixed*10.;let gross=photo+chemo;let fraction=select(0.,max(0.,1.-cost/max(gross,.0000001)),gross>0.);
 photo*=fraction;chemo*=fraction;s.pools[17].y+=fixed;s.pools[27].y+=fixed;s.pools[28].w+=fixed;
 let nitrogen_capacity=s.pools[17].y/t.c.y;let phosphorus_capacity=s.pools[17].z/t.c.z;
 let carbon=min(photo+chemo,min(nitrogen_capacity,phosphorus_capacity));
 // Codes: 0 energy, 1 N, 2 P, 3 habitat, 4 water, 5 trace, 6 oxidant.
 var limit=0.;if local_wet<.05 {limit=4.;}else if habitat.fields[4].w*50.<.1 {limit=5.;}else if secondary_potential>hydrogen_carbon&&secondary_potential>s.pools[26].w*.05 {limit=6.;}
 if carbon<photo+chemo {limit=select(1.,2.,phosphorus_capacity<nitrogen_capacity);}
 environment[i].fields[16u+k]=vec4(photo+chemo,carbon,died.x+resp,limit);
 let utilization=carbon/max(photo+chemo,.00000001);let consumed=utilization*fraction;
 s.pools[26].x-=h*consumed;s.pools[26].y-=secondary*consumed;
 // Fixation also consumes its reserved geochemical share even if subsequent plant growth is limited.
 let reserve=select(0.,cost/max(gross,.000001),gross>0.);
 s.pools[26].x=max(0.,s.pools[26].x-h*reserve);s.pools[26].y=max(0.,s.pools[26].y-secondary*reserve);
 // Charge oxidative chemistry used for both biomass and nitrogen fixation.
 s.pools[26].w=max(0.,s.pools[26].w-secondary_carbon*(consumed+reserve)/.05);
 environment[i].fields[21].y+=h*(consumed+reserve);
 let added=vec3(carbon,carbon*t.c.y,carbon*t.c.z);s.pools[k]+=vec4(added,0.);s.pools[17].y=max(0.,s.pools[17].y-added.y);s.pools[17].z=max(0.,s.pools[17].z-added.z);s.pools[27].x+=carbon;s.pools[28].x+=photo*utilization;s.pools[28].y+=chemo*utilization;
 if carbon<photo+chemo {s.pools[29].w=select(1.,2.,s.pools[17].z/t.c.z<s.pools[17].y/t.c.y);}
 }
 // Stratified lake compartments: symmetric volume exchange plus explicit particle settling.
 let depth=max(e.fields[3].z,1.);let surface=min(100.,depth);let deep=max(depth-surface,1.);
 let peripheral=exp(-pow((acos(clamp(e.fields[7].z,-1.,1.))-.90)/.16,2.));
 var shelf_gradient=0.;
 for(var edge=0u;edge<4u;edge++){let j=neighbor(i,edge,p.dims.y);let fractions=environment[j].fields[0];
 if fractions.x+fractions.y>0. {shelf_gradient+=max(0.,depth-environment[j].fields[3].z)/max(depth,1.);}}
 let wind_upwelling=clamp(abs(e.fields[1].w)/8.,.2,2.)*shelf_gradient*.25;
 let exchange=min(surface*.2,(.02+peripheral*(2.+wind_upwelling))*s.pools[31].y*dt)*w;
 let up=s.pools[21].xyz*min(.2,exchange/max(deep*w,.001));let down=s.pools[20].xyz*min(.2,exchange/max(surface*w,.001));
 s.pools[20]+=vec4(up-down,0.);s.pools[21]+=vec4(down-up,0.);s.pools[29].z=exchange;
 let old_aquatic=s.pools[37];let state=community(e,5u,old_aquatic,sun*thermal,s.pools[23].x/max(w,.001),s.pools[20].yz/max(w,.001));
 var changed_aqua=0.;if old_aquatic.w>0. {if old_aquatic.x!=state.x {changed_aqua+=old_aquatic.z;}if old_aquatic.y!=state.y {changed_aqua+=1.-old_aquatic.z;}}
 let displaced_aqua=s.pools[23].xyz*clamp(changed_aqua,0.,1.);s.pools[23]-=vec4(displaced_aqua,0.);s.pools[22]+=vec4(displaced_aqua,0.);
 s.pools[37]=select(vec4(0.),state,p.abundance.y>=.5);
 let aquatic=select(u32(state.y)-1u,u32(state.x)-1u,state.z>=.5);
 var fallback:Entry;fallback.c=vec4(5.,.03,.003,.08);fallback.d=vec4(0.,0.,0.,.3);
 let aquatic_traits=mixture(state,fallback);
 s.pools[23].w=select(0.,f32(aquatic+1u),aquatic!=NONE);
 let aquatic_photo=select(0.,sun*w*aquatic_traits.b.y*dt*thermal,aquatic!=NONE);
 let aquatic_fixed=min(s.pools[20].z*5.,aquatic_photo*aquatic_traits.d.y*microbial(3u).x*.08);
 s.pools[20].y+=aquatic_fixed;s.pools[27].y+=aquatic_fixed;s.pools[28].w+=aquatic_fixed;
 let algae=min(s.pools[20].y/aquatic_traits.c.y,min(s.pools[20].z/aquatic_traits.c.z,max(0.,aquatic_photo-aquatic_fixed*10.)));
 s.pools[23]+=vec4(algae,algae*aquatic_traits.c.y,algae*aquatic_traits.c.z,0.);s.pools[20].y=max(0.,s.pools[20].y-algae*aquatic_traits.c.y);s.pools[20].z=max(0.,s.pools[20].z-algae*aquatic_traits.c.z);s.pools[27].x+=algae;s.pools[28].x+=algae;
 let aquatic_resp=min(s.pools[23].x,s.pools[23].x*aquatic_traits.c.w*dt);s.pools[23].x-=aquatic_resp;s.pools[27].x-=aquatic_resp;s.pools[28].z+=aquatic_resp;
 let aquatic_returned=max(vec2(0.),s.pools[23].yz-s.pools[23].x*aquatic_traits.c.yz);
 s.pools[23].y-=aquatic_returned.x;s.pools[23].z-=aquatic_returned.y;s.pools[20].y+=aquatic_returned.x;s.pools[20].z+=aquatic_returned.y;
 let settling=s.pools[23].xyz*dt*aquatic_traits.d.w;s.pools[23]-=vec4(settling,0.);s.pools[22]+=vec4(settling,0.);
 let remin=s.pools[22].xyz*dt*.02*microbial(5u).y;s.pools[22]-=vec4(remin,0.);s.pools[21]+=vec4(0.,remin.yz,0.);s.pools[27].x-=remin.x;s.pools[28].z+=remin.x;
 s.pools[31].z=clamp(s.pools[31].z+exchange*.01-remin.x*2.,0.,1.);
 let release=s.pools[22].z*(1.-s.pools[31].z)*dt*.05;s.pools[22].z-=release;s.pools[21].z+=release;
 let burial=s.pools[22].xyz*dt*.001;s.pools[22]-=vec4(burial,0.);s.pools[24]+=vec4(burial,0.);
 for(var k=0u;k<p.options.y;k++) {
 let slot=k+5u;let t=catalog[guild_offset()+k];let deadmask=u32(s.pools[31].w);
 if (deadmask&(1u<<k))!=0u {let waste=select(18u,22u,t.ids.y==1u&&w>l);s.pools[waste]+=vec4(s.pools[slot].xyz,0.);s.pools[slot]=vec4(0.);s.pools[38u+k/4u][k%4u]=0.;continue;}
 var preference_temperature=thermal_preference(s,k);
 if ecotypes_enabled()&&preference_temperature==0.&&s.pools[slot].x>0. {
 preference_temperature=founder_preference(i,k,e); // Explicit first-use baseline for old stocks.
 s.pools[38u+k/4u][k%4u]=preference_temperature;}
 let demand=s.pools[slot].x*t.b.x*dt*thermal_match(preference_temperature,e.fields[1].x,t.ids.y==1u);
 var weighted=0.;var preference:array<f32,4>;let refuge=bitcast<f32>(t.ids.x);
 for(var d=0u;d<4u;d++){let prey=u32(t.c[d]);let food=s.pools[prey].x;
 // Rare animal prey use refuges / are less worth pursuing. Plants retain their
 // original preference. The lost encounter opportunity does not create food.
 preference[d]=t.d[d]*select(1.,food/max(food+refuge,1e-30),prey>=5u&&prey<17u&&refuge>0.);
 weighted+=food*preference[d]*food_access(e,prey);}
 // Saturating encounter rate: scarce prey cannot meet a fixed demand.
 // The guard only prevents division by zero. A biomass-scale epsilon here
 // creates an undocumented low-density feeding threshold even when H is zero.
 let capture=select(1.,weighted/max(weighted+t.b.w,1e-30),t.b.w>0.);
 var assimilable=0.;var meal=vec3(0.);
 for(var d=0u;d<4u;d++) {let prey=u32(t.c[d]);let access=food_access(e,prey);
 let bite=s.pools[prey].xyz*min(.25*access,demand*capture*preference[d]*access/max(weighted,1e-30));
 let efficiency=f32((t.ids[2u+d/2u]>>((d%2u)*16u))&65535u)/65535.;
 s.pools[prey]-=vec4(bite,0.);meal+=bite;assimilable+=bite.x*efficiency;
 }
 let growth=min(assimilable*t.a.z,min(meal.y/t.a.x,meal.z/t.a.y));let biomass=vec3(growth,growth*t.a.x,growth*t.a.y);
 if ecotypes_enabled()&&preference_temperature>0. {
 // Slow local adjustment is limited by replacement through actual growth.
 let replacement=growth/max(s.pools[slot].x+growth,1e-30);
 s.pools[38u+k/4u][k%4u]=mix(preference_temperature,81.+clamp(e.fields[1].x,-80.,60.),.02*replacement);}
 s.pools[slot]+=vec4(biomass,0.);
 let leftovers=max(vec3(0.),meal-biomass);let waste=select(18u,22u,t.ids.y==1u&&w>l);s.pools[waste]+=vec4(leftovers.x*.4,leftovers.yz,0.);s.pools[27].x-=leftovers.x*.6;s.pools[28].z+=leftovers.x*.6;
 let resp=min(s.pools[slot].x,s.pools[slot].x*t.a.w*dt);s.pools[slot].x-=resp;s.pools[27].x-=resp;s.pools[28].z+=resp;
 let excreted=max(vec2(0.),s.pools[slot].yz-s.pools[slot].x*t.a.xy);
 s.pools[slot].y-=excreted.x;s.pools[slot].z-=excreted.y;
 let dissolved=select(17u,20u,t.ids.y==1u&&w>l);s.pools[dissolved].y+=excreted.x;s.pools[dissolved].z+=excreted.y;
 let death=s.pools[slot].xyz*dt*.02;s.pools[slot]-=vec4(death,0.);s.pools[waste]+=vec4(death,0.);
 // Numerical extinction transfers the remainder; it never deletes nutrients.
 if s.pools[slot].x<1e-12 {s.pools[waste]+=vec4(s.pools[slot].xyz,0.);s.pools[slot]=vec4(0.);s.pools[38u+k/4u][k%4u]=0.;}

 }
 dst[i]=s;
}
// Pairwise conservative transport: at most 4 * .02 of each source pool per step.
// Absolute exchanged mass uses min(cell areas), so seams and unequal areas conserve.
fn exchange_fraction(i:u32,j:u32,k:u32)->f32 {
 let a=environment[i];let b=environment[j];let dt=p.physical.y;
 if k==20u||k==21u||k==23u {
 if water(a)<=0.||water(b)<=0. {return 0.;}
 // Exterior ocean and enclosed lake remain independent even in coarse mixed cells.
 if a.fields[0].x*b.fields[0].x+a.fields[0].y*b.fields[0].y<=0. {return 0.;}
 let tangent=normalize(cross(vec3(0.,0.,1.),a.fields[7].xyz)+vec3(.000001));
 let v=dot(tangent,b.fields[7].xyz-a.fields[7].xyz)*f32(p.dims.y);
 let friction=clamp(a.fields[3].z/200.,.1,1.);
 let flow=clamp(.5+v*.4*clamp(a.fields[1].w/8.,-2.,2.)*friction,0.,1.);return min(.02,dt*.1*flow)*select(.15,1.,k!=21u);
 }
 if k<5u||k>16u {return 0.;}
 let guild=k-5u;if (u32(src[j].pools[31].w)&(1u<<guild))!=0u {return 0.;}
 let t=catalog[guild_offset()+guild];let migrant=guild==10u||guild==11u;
 var edge=0u;for(var d=0u;d<4u;d++){if neighbor(i,d,p.dims.y)==j {edge=d;}}
 var conductance=a.fields[22][edge];
 if t.ids.y==1u {conductance=a.fields[23][edge];}
 if guild==10u {conductance=a.fields[24][edge];}
 // Birds can cross habitat boundaries; flight is bounded by existing step flux.
 if guild==11u {conductance=1.;}
 if (u32(p.abundance.z)&1u)!=0u {conductance=1.;} // explicit connectivity ablation
 if conductance<=0. {return 0.;}
 var food=0.;for(var d=0u;d<4u;d++){let prey=u32(t.c[d]);food+=src[j].pools[prey].x*t.d[d]*food_access(b,prey);}var attraction=food/(food+.01);
 if guild==3u||guild==4u {attraction=max(attraction,select(.1,.6,b.fields[3].y>a.fields[3].y));}
 if migrant {let season=sin(f32(season_month()%12u)*.5235988);attraction=max(attraction,select(water(b),land(b),season>0.)*.8);}
 return min(.02,dt*t.b.y*attraction*conductance);
}
// Fixed compartment accesses keep the compiler from spilling a dynamically
// indexed private Eco record. Each block preserves the original transfer order.
@compute @workgroup_size(8,8)
fn transport(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=id(g,p.dims.y);let a=environment[i].fields[3].x;
 dst[i]=src[i];var ledger=src[i].pools[27];var production=src[i].pools[28];
 { const k:u32=5u;let original=src[i].pools[k];var delta=vec3(0.);var moving=0.;var ancestry_delta=0.;var thermal_delta=0.;var known_delta=0.;let original_temperature=thermal_preference(src[i],k-5u);
 for(var d=0u;d<4u;d++){let j=neighbor(i,d,p.dims.y);
 if all(original.xyz==vec3(0.)) && all(src[j].pools[k].xyz==vec3(0.)) {continue;}
 let b=environment[j].fields[3].x;let edge=min(a,b);
 let outward=exchange_fraction(i,j,k);let inward=exchange_fraction(j,i,k);
 moving+=original.x*outward*edge/a;
 delta+=(src[j].pools[k].xyz*inward-original.xyz*outward)*edge/a;
 ancestry_delta+=(src[j].pools[k].x*src[j].pools[k].w*inward-original.x*original.w*outward)*edge/a;
 thermal_delta+=(src[j].pools[k].x*thermal_preference(src[j],k-5u)*inward-original.x*original_temperature*outward)*edge/a;
 known_delta+=(select(0.,src[j].pools[k].x,thermal_preference(src[j],k-5u)>0.)*inward-select(0.,original.x,original_temperature>0.)*outward)*edge/a;}
 var value=vec4(max(vec3(0.),original.xyz+delta),original.w);
 value.w=select(0.,clamp((original.x*original.w+ancestry_delta)/max(value.x,1e-30),0.,1.),value.x>0.);
 let known=max(0.,select(0.,original.x,original_temperature>0.)+known_delta);
 let temperature=select(0.,clamp((original.x*original_temperature+thermal_delta)/max(known,1e-30),1.,141.),value.x>0.&&known>0.);
 dst[i].pools[38u+(k-5u)/4u][(k-5u)%4u]=temperature;
 let cost=min(value.x,moving*.02);value.x-=cost;ledger.x-=cost;production.z+=cost;
 dst[i].pools[k]=value; }
 { const k:u32=6u;let original=src[i].pools[k];var delta=vec3(0.);var moving=0.;var ancestry_delta=0.;var thermal_delta=0.;var known_delta=0.;let original_temperature=thermal_preference(src[i],k-5u);
 for(var d=0u;d<4u;d++){let j=neighbor(i,d,p.dims.y);
 if all(original.xyz==vec3(0.)) && all(src[j].pools[k].xyz==vec3(0.)) {continue;}
 let b=environment[j].fields[3].x;let edge=min(a,b);
 let outward=exchange_fraction(i,j,k);let inward=exchange_fraction(j,i,k);
 moving+=original.x*outward*edge/a;
 delta+=(src[j].pools[k].xyz*inward-original.xyz*outward)*edge/a;
 ancestry_delta+=(src[j].pools[k].x*src[j].pools[k].w*inward-original.x*original.w*outward)*edge/a;
 thermal_delta+=(src[j].pools[k].x*thermal_preference(src[j],k-5u)*inward-original.x*original_temperature*outward)*edge/a;
 known_delta+=(select(0.,src[j].pools[k].x,thermal_preference(src[j],k-5u)>0.)*inward-select(0.,original.x,original_temperature>0.)*outward)*edge/a;}
 var value=vec4(max(vec3(0.),original.xyz+delta),original.w);
 value.w=select(0.,clamp((original.x*original.w+ancestry_delta)/max(value.x,1e-30),0.,1.),value.x>0.);
 let known=max(0.,select(0.,original.x,original_temperature>0.)+known_delta);
 let temperature=select(0.,clamp((original.x*original_temperature+thermal_delta)/max(known,1e-30),1.,141.),value.x>0.&&known>0.);
 dst[i].pools[38u+(k-5u)/4u][(k-5u)%4u]=temperature;
 let cost=min(value.x,moving*.02);value.x-=cost;ledger.x-=cost;production.z+=cost;
 dst[i].pools[k]=value; }
 { const k:u32=7u;let original=src[i].pools[k];var delta=vec3(0.);var moving=0.;var ancestry_delta=0.;var thermal_delta=0.;var known_delta=0.;let original_temperature=thermal_preference(src[i],k-5u);
 for(var d=0u;d<4u;d++){let j=neighbor(i,d,p.dims.y);
 if all(original.xyz==vec3(0.)) && all(src[j].pools[k].xyz==vec3(0.)) {continue;}
 let b=environment[j].fields[3].x;let edge=min(a,b);
 let outward=exchange_fraction(i,j,k);let inward=exchange_fraction(j,i,k);
 moving+=original.x*outward*edge/a;
 delta+=(src[j].pools[k].xyz*inward-original.xyz*outward)*edge/a;
 ancestry_delta+=(src[j].pools[k].x*src[j].pools[k].w*inward-original.x*original.w*outward)*edge/a;
 thermal_delta+=(src[j].pools[k].x*thermal_preference(src[j],k-5u)*inward-original.x*original_temperature*outward)*edge/a;
 known_delta+=(select(0.,src[j].pools[k].x,thermal_preference(src[j],k-5u)>0.)*inward-select(0.,original.x,original_temperature>0.)*outward)*edge/a;}
 var value=vec4(max(vec3(0.),original.xyz+delta),original.w);
 value.w=select(0.,clamp((original.x*original.w+ancestry_delta)/max(value.x,1e-30),0.,1.),value.x>0.);
 let known=max(0.,select(0.,original.x,original_temperature>0.)+known_delta);
 let temperature=select(0.,clamp((original.x*original_temperature+thermal_delta)/max(known,1e-30),1.,141.),value.x>0.&&known>0.);
 dst[i].pools[38u+(k-5u)/4u][(k-5u)%4u]=temperature;
 let cost=min(value.x,moving*.02);value.x-=cost;ledger.x-=cost;production.z+=cost;
 dst[i].pools[k]=value; }
 { const k:u32=8u;let original=src[i].pools[k];var delta=vec3(0.);var moving=0.;var ancestry_delta=0.;var thermal_delta=0.;var known_delta=0.;let original_temperature=thermal_preference(src[i],k-5u);
 for(var d=0u;d<4u;d++){let j=neighbor(i,d,p.dims.y);
 if all(original.xyz==vec3(0.)) && all(src[j].pools[k].xyz==vec3(0.)) {continue;}
 let b=environment[j].fields[3].x;let edge=min(a,b);
 let outward=exchange_fraction(i,j,k);let inward=exchange_fraction(j,i,k);
 moving+=original.x*outward*edge/a;
 delta+=(src[j].pools[k].xyz*inward-original.xyz*outward)*edge/a;
 ancestry_delta+=(src[j].pools[k].x*src[j].pools[k].w*inward-original.x*original.w*outward)*edge/a;
 thermal_delta+=(src[j].pools[k].x*thermal_preference(src[j],k-5u)*inward-original.x*original_temperature*outward)*edge/a;
 known_delta+=(select(0.,src[j].pools[k].x,thermal_preference(src[j],k-5u)>0.)*inward-select(0.,original.x,original_temperature>0.)*outward)*edge/a;}
 var value=vec4(max(vec3(0.),original.xyz+delta),original.w);
 value.w=select(0.,clamp((original.x*original.w+ancestry_delta)/max(value.x,1e-30),0.,1.),value.x>0.);
 let known=max(0.,select(0.,original.x,original_temperature>0.)+known_delta);
 let temperature=select(0.,clamp((original.x*original_temperature+thermal_delta)/max(known,1e-30),1.,141.),value.x>0.&&known>0.);
 dst[i].pools[38u+(k-5u)/4u][(k-5u)%4u]=temperature;
 let cost=min(value.x,moving*.02);value.x-=cost;ledger.x-=cost;production.z+=cost;
 dst[i].pools[k]=value; }
 { const k:u32=9u;let original=src[i].pools[k];var delta=vec3(0.);var moving=0.;var ancestry_delta=0.;var thermal_delta=0.;var known_delta=0.;let original_temperature=thermal_preference(src[i],k-5u);
 for(var d=0u;d<4u;d++){let j=neighbor(i,d,p.dims.y);
 if all(original.xyz==vec3(0.)) && all(src[j].pools[k].xyz==vec3(0.)) {continue;}
 let b=environment[j].fields[3].x;let edge=min(a,b);
 let outward=exchange_fraction(i,j,k);let inward=exchange_fraction(j,i,k);
 moving+=original.x*outward*edge/a;
 delta+=(src[j].pools[k].xyz*inward-original.xyz*outward)*edge/a;
 ancestry_delta+=(src[j].pools[k].x*src[j].pools[k].w*inward-original.x*original.w*outward)*edge/a;
 thermal_delta+=(src[j].pools[k].x*thermal_preference(src[j],k-5u)*inward-original.x*original_temperature*outward)*edge/a;
 known_delta+=(select(0.,src[j].pools[k].x,thermal_preference(src[j],k-5u)>0.)*inward-select(0.,original.x,original_temperature>0.)*outward)*edge/a;}
 var value=vec4(max(vec3(0.),original.xyz+delta),original.w);
 value.w=select(0.,clamp((original.x*original.w+ancestry_delta)/max(value.x,1e-30),0.,1.),value.x>0.);
 let known=max(0.,select(0.,original.x,original_temperature>0.)+known_delta);
 let temperature=select(0.,clamp((original.x*original_temperature+thermal_delta)/max(known,1e-30),1.,141.),value.x>0.&&known>0.);
 dst[i].pools[38u+(k-5u)/4u][(k-5u)%4u]=temperature;
 let cost=min(value.x,moving*.02);value.x-=cost;ledger.x-=cost;production.z+=cost;
 dst[i].pools[k]=value; }
 { const k:u32=10u;let original=src[i].pools[k];var delta=vec3(0.);var moving=0.;var ancestry_delta=0.;var thermal_delta=0.;var known_delta=0.;let original_temperature=thermal_preference(src[i],k-5u);
 for(var d=0u;d<4u;d++){let j=neighbor(i,d,p.dims.y);
 if all(original.xyz==vec3(0.)) && all(src[j].pools[k].xyz==vec3(0.)) {continue;}
 let b=environment[j].fields[3].x;let edge=min(a,b);
 let outward=exchange_fraction(i,j,k);let inward=exchange_fraction(j,i,k);
 moving+=original.x*outward*edge/a;
 delta+=(src[j].pools[k].xyz*inward-original.xyz*outward)*edge/a;
 ancestry_delta+=(src[j].pools[k].x*src[j].pools[k].w*inward-original.x*original.w*outward)*edge/a;
 thermal_delta+=(src[j].pools[k].x*thermal_preference(src[j],k-5u)*inward-original.x*original_temperature*outward)*edge/a;
 known_delta+=(select(0.,src[j].pools[k].x,thermal_preference(src[j],k-5u)>0.)*inward-select(0.,original.x,original_temperature>0.)*outward)*edge/a;}
 var value=vec4(max(vec3(0.),original.xyz+delta),original.w);
 value.w=select(0.,clamp((original.x*original.w+ancestry_delta)/max(value.x,1e-30),0.,1.),value.x>0.);
 let known=max(0.,select(0.,original.x,original_temperature>0.)+known_delta);
 let temperature=select(0.,clamp((original.x*original_temperature+thermal_delta)/max(known,1e-30),1.,141.),value.x>0.&&known>0.);
 dst[i].pools[38u+(k-5u)/4u][(k-5u)%4u]=temperature;
 let cost=min(value.x,moving*.02);value.x-=cost;ledger.x-=cost;production.z+=cost;
 dst[i].pools[k]=value; }
 { const k:u32=11u;let original=src[i].pools[k];var delta=vec3(0.);var moving=0.;var ancestry_delta=0.;var thermal_delta=0.;var known_delta=0.;let original_temperature=thermal_preference(src[i],k-5u);
 for(var d=0u;d<4u;d++){let j=neighbor(i,d,p.dims.y);
 if all(original.xyz==vec3(0.)) && all(src[j].pools[k].xyz==vec3(0.)) {continue;}
 let b=environment[j].fields[3].x;let edge=min(a,b);
 let outward=exchange_fraction(i,j,k);let inward=exchange_fraction(j,i,k);
 moving+=original.x*outward*edge/a;
 delta+=(src[j].pools[k].xyz*inward-original.xyz*outward)*edge/a;
 ancestry_delta+=(src[j].pools[k].x*src[j].pools[k].w*inward-original.x*original.w*outward)*edge/a;
 thermal_delta+=(src[j].pools[k].x*thermal_preference(src[j],k-5u)*inward-original.x*original_temperature*outward)*edge/a;
 known_delta+=(select(0.,src[j].pools[k].x,thermal_preference(src[j],k-5u)>0.)*inward-select(0.,original.x,original_temperature>0.)*outward)*edge/a;}
 var value=vec4(max(vec3(0.),original.xyz+delta),original.w);
 value.w=select(0.,clamp((original.x*original.w+ancestry_delta)/max(value.x,1e-30),0.,1.),value.x>0.);
 let known=max(0.,select(0.,original.x,original_temperature>0.)+known_delta);
 let temperature=select(0.,clamp((original.x*original_temperature+thermal_delta)/max(known,1e-30),1.,141.),value.x>0.&&known>0.);
 dst[i].pools[38u+(k-5u)/4u][(k-5u)%4u]=temperature;
 let cost=min(value.x,moving*.02);value.x-=cost;ledger.x-=cost;production.z+=cost;
 dst[i].pools[k]=value; }
 { const k:u32=12u;let original=src[i].pools[k];var delta=vec3(0.);var moving=0.;var ancestry_delta=0.;var thermal_delta=0.;var known_delta=0.;let original_temperature=thermal_preference(src[i],k-5u);
 for(var d=0u;d<4u;d++){let j=neighbor(i,d,p.dims.y);
 if all(original.xyz==vec3(0.)) && all(src[j].pools[k].xyz==vec3(0.)) {continue;}
 let b=environment[j].fields[3].x;let edge=min(a,b);
 let outward=exchange_fraction(i,j,k);let inward=exchange_fraction(j,i,k);
 moving+=original.x*outward*edge/a;
 delta+=(src[j].pools[k].xyz*inward-original.xyz*outward)*edge/a;
 ancestry_delta+=(src[j].pools[k].x*src[j].pools[k].w*inward-original.x*original.w*outward)*edge/a;
 thermal_delta+=(src[j].pools[k].x*thermal_preference(src[j],k-5u)*inward-original.x*original_temperature*outward)*edge/a;
 known_delta+=(select(0.,src[j].pools[k].x,thermal_preference(src[j],k-5u)>0.)*inward-select(0.,original.x,original_temperature>0.)*outward)*edge/a;}
 var value=vec4(max(vec3(0.),original.xyz+delta),original.w);
 value.w=select(0.,clamp((original.x*original.w+ancestry_delta)/max(value.x,1e-30),0.,1.),value.x>0.);
 let known=max(0.,select(0.,original.x,original_temperature>0.)+known_delta);
 let temperature=select(0.,clamp((original.x*original_temperature+thermal_delta)/max(known,1e-30),1.,141.),value.x>0.&&known>0.);
 dst[i].pools[38u+(k-5u)/4u][(k-5u)%4u]=temperature;
 let cost=min(value.x,moving*.02);value.x-=cost;ledger.x-=cost;production.z+=cost;
 dst[i].pools[k]=value; }
 { const k:u32=13u;let original=src[i].pools[k];var delta=vec3(0.);var moving=0.;var ancestry_delta=0.;var thermal_delta=0.;var known_delta=0.;let original_temperature=thermal_preference(src[i],k-5u);
 for(var d=0u;d<4u;d++){let j=neighbor(i,d,p.dims.y);
 if all(original.xyz==vec3(0.)) && all(src[j].pools[k].xyz==vec3(0.)) {continue;}
 let b=environment[j].fields[3].x;let edge=min(a,b);
 let outward=exchange_fraction(i,j,k);let inward=exchange_fraction(j,i,k);
 moving+=original.x*outward*edge/a;
 delta+=(src[j].pools[k].xyz*inward-original.xyz*outward)*edge/a;
 ancestry_delta+=(src[j].pools[k].x*src[j].pools[k].w*inward-original.x*original.w*outward)*edge/a;
 thermal_delta+=(src[j].pools[k].x*thermal_preference(src[j],k-5u)*inward-original.x*original_temperature*outward)*edge/a;
 known_delta+=(select(0.,src[j].pools[k].x,thermal_preference(src[j],k-5u)>0.)*inward-select(0.,original.x,original_temperature>0.)*outward)*edge/a;}
 var value=vec4(max(vec3(0.),original.xyz+delta),original.w);
 value.w=select(0.,clamp((original.x*original.w+ancestry_delta)/max(value.x,1e-30),0.,1.),value.x>0.);
 let known=max(0.,select(0.,original.x,original_temperature>0.)+known_delta);
 let temperature=select(0.,clamp((original.x*original_temperature+thermal_delta)/max(known,1e-30),1.,141.),value.x>0.&&known>0.);
 dst[i].pools[38u+(k-5u)/4u][(k-5u)%4u]=temperature;
 let cost=min(value.x,moving*.02);value.x-=cost;ledger.x-=cost;production.z+=cost;
 dst[i].pools[k]=value; }
 { const k:u32=14u;let original=src[i].pools[k];var delta=vec3(0.);var moving=0.;var ancestry_delta=0.;var thermal_delta=0.;var known_delta=0.;let original_temperature=thermal_preference(src[i],k-5u);
 for(var d=0u;d<4u;d++){let j=neighbor(i,d,p.dims.y);
 if all(original.xyz==vec3(0.)) && all(src[j].pools[k].xyz==vec3(0.)) {continue;}
 let b=environment[j].fields[3].x;let edge=min(a,b);
 let outward=exchange_fraction(i,j,k);let inward=exchange_fraction(j,i,k);
 moving+=original.x*outward*edge/a;
 delta+=(src[j].pools[k].xyz*inward-original.xyz*outward)*edge/a;
 ancestry_delta+=(src[j].pools[k].x*src[j].pools[k].w*inward-original.x*original.w*outward)*edge/a;
 thermal_delta+=(src[j].pools[k].x*thermal_preference(src[j],k-5u)*inward-original.x*original_temperature*outward)*edge/a;
 known_delta+=(select(0.,src[j].pools[k].x,thermal_preference(src[j],k-5u)>0.)*inward-select(0.,original.x,original_temperature>0.)*outward)*edge/a;}
 var value=vec4(max(vec3(0.),original.xyz+delta),original.w);
 value.w=select(0.,clamp((original.x*original.w+ancestry_delta)/max(value.x,1e-30),0.,1.),value.x>0.);
 let known=max(0.,select(0.,original.x,original_temperature>0.)+known_delta);
 let temperature=select(0.,clamp((original.x*original_temperature+thermal_delta)/max(known,1e-30),1.,141.),value.x>0.&&known>0.);
 dst[i].pools[38u+(k-5u)/4u][(k-5u)%4u]=temperature;
 let cost=min(value.x,moving*.02);value.x-=cost;ledger.x-=cost;production.z+=cost;
 dst[i].pools[k]=value; }
 { const k:u32=15u;let original=src[i].pools[k];var delta=vec3(0.);var moving=0.;var ancestry_delta=0.;var thermal_delta=0.;var known_delta=0.;let original_temperature=thermal_preference(src[i],k-5u);
 for(var d=0u;d<4u;d++){let j=neighbor(i,d,p.dims.y);
 if all(original.xyz==vec3(0.)) && all(src[j].pools[k].xyz==vec3(0.)) {continue;}
 let b=environment[j].fields[3].x;let edge=min(a,b);
 let outward=exchange_fraction(i,j,k);let inward=exchange_fraction(j,i,k);
 moving+=original.x*outward*edge/a;
 delta+=(src[j].pools[k].xyz*inward-original.xyz*outward)*edge/a;
 ancestry_delta+=(src[j].pools[k].x*src[j].pools[k].w*inward-original.x*original.w*outward)*edge/a;
 thermal_delta+=(src[j].pools[k].x*thermal_preference(src[j],k-5u)*inward-original.x*original_temperature*outward)*edge/a;
 known_delta+=(select(0.,src[j].pools[k].x,thermal_preference(src[j],k-5u)>0.)*inward-select(0.,original.x,original_temperature>0.)*outward)*edge/a;}
 var value=vec4(max(vec3(0.),original.xyz+delta),original.w);
 value.w=select(0.,clamp((original.x*original.w+ancestry_delta)/max(value.x,1e-30),0.,1.),value.x>0.);
 let known=max(0.,select(0.,original.x,original_temperature>0.)+known_delta);
 let temperature=select(0.,clamp((original.x*original_temperature+thermal_delta)/max(known,1e-30),1.,141.),value.x>0.&&known>0.);
 dst[i].pools[38u+(k-5u)/4u][(k-5u)%4u]=temperature;
 let cost=min(value.x,moving*.02);value.x-=cost;ledger.x-=cost;production.z+=cost;
 dst[i].pools[k]=value; }
 { const k:u32=16u;let original=src[i].pools[k];var delta=vec3(0.);var moving=0.;var ancestry_delta=0.;var thermal_delta=0.;var known_delta=0.;let original_temperature=thermal_preference(src[i],k-5u);
 for(var d=0u;d<4u;d++){let j=neighbor(i,d,p.dims.y);
 if all(original.xyz==vec3(0.)) && all(src[j].pools[k].xyz==vec3(0.)) {continue;}
 let b=environment[j].fields[3].x;let edge=min(a,b);
 let outward=exchange_fraction(i,j,k);let inward=exchange_fraction(j,i,k);
 moving+=original.x*outward*edge/a;
 delta+=(src[j].pools[k].xyz*inward-original.xyz*outward)*edge/a;
 ancestry_delta+=(src[j].pools[k].x*src[j].pools[k].w*inward-original.x*original.w*outward)*edge/a;
 thermal_delta+=(src[j].pools[k].x*thermal_preference(src[j],k-5u)*inward-original.x*original_temperature*outward)*edge/a;
 known_delta+=(select(0.,src[j].pools[k].x,thermal_preference(src[j],k-5u)>0.)*inward-select(0.,original.x,original_temperature>0.)*outward)*edge/a;}
 var value=vec4(max(vec3(0.),original.xyz+delta),original.w);
 value.w=select(0.,clamp((original.x*original.w+ancestry_delta)/max(value.x,1e-30),0.,1.),value.x>0.);
 let known=max(0.,select(0.,original.x,original_temperature>0.)+known_delta);
 let temperature=select(0.,clamp((original.x*original_temperature+thermal_delta)/max(known,1e-30),1.,141.),value.x>0.&&known>0.);
 dst[i].pools[38u+(k-5u)/4u][(k-5u)%4u]=temperature;
 let cost=min(value.x,moving*.02);value.x-=cost;ledger.x-=cost;production.z+=cost;
 dst[i].pools[k]=value; }
 { const k:u32=20u;let original=src[i].pools[k];var delta=vec3(0.);var moving=0.;
 for(var d=0u;d<4u;d++){let j=neighbor(i,d,p.dims.y);
 if all(original.xyz==vec3(0.)) && all(src[j].pools[k].xyz==vec3(0.)) {continue;}
 let b=environment[j].fields[3].x;let edge=min(a,b);
 let outward=exchange_fraction(i,j,k);let inward=exchange_fraction(j,i,k);
 moving+=original.x*outward*edge/a;
 delta+=(src[j].pools[k].xyz*inward-original.xyz*outward)*edge/a;}
 var value=vec4(max(vec3(0.),original.xyz+delta),original.w);
 dst[i].pools[k]=value; }
 { const k:u32=21u;let original=src[i].pools[k];var delta=vec3(0.);var moving=0.;
 for(var d=0u;d<4u;d++){let j=neighbor(i,d,p.dims.y);
 if all(original.xyz==vec3(0.)) && all(src[j].pools[k].xyz==vec3(0.)) {continue;}
 let b=environment[j].fields[3].x;let edge=min(a,b);
 let outward=exchange_fraction(i,j,k);let inward=exchange_fraction(j,i,k);
 moving+=original.x*outward*edge/a;
 delta+=(src[j].pools[k].xyz*inward-original.xyz*outward)*edge/a;}
 var value=vec4(max(vec3(0.),original.xyz+delta),original.w);
 dst[i].pools[k]=value; }
 { const k:u32=23u;let original=src[i].pools[k];var delta=vec3(0.);var moving=0.;
 for(var d=0u;d<4u;d++){let j=neighbor(i,d,p.dims.y);
 if all(original.xyz==vec3(0.)) && all(src[j].pools[k].xyz==vec3(0.)) {continue;}
 let b=environment[j].fields[3].x;let edge=min(a,b);
 let outward=exchange_fraction(i,j,k);let inward=exchange_fraction(j,i,k);
 moving+=original.x*outward*edge/a;
 delta+=(src[j].pools[k].xyz*inward-original.xyz*outward)*edge/a;}
 var value=vec4(max(vec3(0.),original.xyz+delta),original.w);
 dst[i].pools[k]=value; }
 let tangent=cross(vec3(0.,0.,1.),environment[i].fields[7].xyz);
 dst[i].pools[29].x=tangent.x;dst[i].pools[29].y=tangent.y;
 dst[i].pools[27]=ledger;dst[i].pools[28]=production;
}
@compute @workgroup_size(8,8)
fn river_inject(@builtin(global_invocation_id) g:vec3<u32>) {let i=id(g,p.dims.x);let c=terrain[i];let j=parent(i);let l=land(environment[j]);var added=vec4(0.);if c.tags.x>=2u {added=src[j].pools[19]*.25*area(i,p.dims.x)/max(l,.000001);}added.w=select(0.,c.life.w*area(i,p.dims.x),c.tags.x>=2u);river_out[i]=river[i]+added;}
@compute @workgroup_size(8,8)
fn river_route(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=id(g,p.dims.x);var mass=river[i]*select(.2,1.,terrain[i].routing.x==NONE);
 for(var k=0u;k<4u;k++){let j=neighbor(i,k,p.dims.x);if terrain[j].routing.x==i {mass+=river[j]*.8;}}
 river_out[i]=mass;
}
// The same overflow fraction credits coarse soil and debits fine routed payloads.
fn overflow_volume(i:u32)->f32 {
 if p.options.w==0u || terrain[i].routing.x==NONE || terrain[i].tags.x<2u {return 0.;}
 let capacity=max(0.,terrain[i].water.w)*31557600.*p.physical.y*2.;
 let fraction=select(1.,.05,terrain[i].water.w>1.&&terrain[i].hydro.x-terrain[i].terrain.x<.01);
 return min(max(0.,river[i].w-capacity),max(0.,1.5*fraction-terrain[i].water.x)*area(i,p.dims.x));
}
@compute @workgroup_size(8,8)
fn river_collect(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=id(g,p.dims.y);var s=src[i];let r=p.dims.x/p.dims.y;var stored=0.;
 for(var y=0u;y<r;y++){for(var x=0u;x<r;x++){let j=fine(i,x,y);let c=terrain[j];stored+=(c.water.x+c.water.y+c.water.z)*area(j,p.dims.x)+river[j].w;if c.routing.x==NONE {let slot=select(17u,20u,c.tags.x<2u);s.pools[slot]+=vec4(river[j].xyz/environment[i].fields[3].x,0.);}else {s.pools[17]+=vec4(river[j].xyz*(overflow_volume(j)/max(river[j].w,1e-20))/environment[i].fields[3].x,0.);}}}s.pools[25].w=stored/environment[i].fields[3].x;dst[i]=s;
}
@compute @workgroup_size(8,8)
fn river_clear(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=id(g,p.dims.x);var payload=river[i];
 if terrain[i].routing.x==NONE {terrain[i].water.x+=payload.w/area(i,p.dims.x);payload=vec4(0.);}
 else if p.options.w==1u && terrain[i].tags.x>=2u {
 // Water and dissolved nutrients leave the river together. river_collect has
 // already credited the latter to regional floodplain soil without atomics.
 let overflow=overflow_volume(i);
 payload=vec4(payload.xyz*(1.-overflow/max(payload.w,1e-20)),payload.w);
 terrain[i].water.x+=overflow/area(i,p.dims.x);payload.w-=overflow;
 }
 river_out[i]=payload;
}
@compute @workgroup_size(8,8)
fn feedback(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=id(g,p.dims.x);let j=parent(i);let s=src[j];var c=terrain[i];let l=max(land(environment[j]),.0001);
 c.life.y=clamp(s.pools[17].z/(.004*l),0.,1.);c.life.x=select(0.,clamp((s.pools[0].x+s.pools[1].x+s.pools[2].x)/(2.*l),0.,1.),c.tags.x>=2u);
 let chosen=u32(s.pools[0].w);c.ids.z=NONE;if chosen>0u&&c.tags.x>=2u {let t=catalog[plants_offset()+chosen-1u];if c.hydro.y>=t.a.x&&c.hydro.y<=t.a.y&&c.hydro.z>=t.a.z&&c.hydro.z<=t.a.w&&c.life.y>=t.b.x&&(t.ids.x==0u||c.tags.x==3u)&&(t.ids.y==3u||t.ids.y==catalog[c.ids.x].ids.x) {c.ids.z=chosen-1u;}}
 var habitat=0u;
 if c.tags.x==1u {habitat=select(6u,7u,s.pools[29].z>.05);}else if c.tags.x>=2u {
 if c.water.x>.1 {habitat=8u;}if s.pools[28].y>.00001 {habitat=5u;}if s.pools[26].y>.02&&c.water.y>.1 {habitat=4u;}if s.pools[26].x>.01 {habitat=3u;}
 }
 if habitat>0u {for(var b=0u;b<p.options.x;b++){if catalog[plants_offset()+p.counts.w+b].ids.x==habitat {c.ids.w=b;break;}}}
 terrain[i]=c;
}
@compute @workgroup_size(8,8)
fn intervention(@builtin(global_invocation_id) g:vec3<u32>) {let i=id(g,p.dims.y);var s=src[i];if p.event.z==NONE||region(environment[i])==p.event.z {switch p.event.x {case 1u:{s.pools[31].x=f32(p.event.y);}case 2u:{let k=p.event.y+5u;let t=catalog[guild_offset()+p.event.y];let e=environment[i];let waste=select(18u,22u,t.ids.y==1u&&water(e)>land(e));s.pools[waste]+=vec4(s.pools[k].xyz,0.);s.pools[k]=vec4(0.);s.pools[38u+p.event.y/4u][p.event.y%4u]=0.;s.pools[31].w=f32(u32(s.pools[31].w)|(1u<<p.event.y));}case 3u:{s.pools[31].w=f32(u32(s.pools[31].w)&~(1u<<p.event.y));}case 4u:{s.pools[31].y=bitcast<f32>(p.event.y);}default:{}}}dst[i]=s;}

// One month of local weather and water storage. Runoff travels in the fourth
// component of the fine watershed payload; receiving lake cells retain it.
fn history_drought(i:u32)->f32 {
 if p.options.w==0u {return 1.;}
 let region=vec3<u32>(floor((pos(i,p.dims.x)+vec3(1.))*4.));
 let key=region.x+9u*region.y+81u*region.z;
 let period=(max(1u,p.event.w)-1u)/max(12u,p.event.z);
 var v=key*7919u^period*104729u^p.dims.w;
 v=(v^(v>>16u))*0x7feb352du;v=(v^(v>>15u))*0x846ca68bu;v=v^(v>>16u);
 let drought=select(1.,1.-bitcast<f32>(p.event.y),f32(v&65535u)/65536.<bitcast<f32>(p.event.x));
 var storm=key*7919u^p.event.w*104729u^p.dims.w^0x9e3779b9u;
 storm=(storm^(storm>>16u))*0x7feb352du;storm=(storm^(storm>>15u))*0x846ca68bu;storm=storm^(storm>>16u);
 return drought*select(1.,p.storms.y,f32(storm&65535u)/65536.<p.storms.x);
}
@compute @workgroup_size(8,8)
fn monthly_weather(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=id(g,p.dims.x);var c=terrain[i];let dt=p.physical.y;
 // Shared solar calendar; retain the weather amplitude at default 23.44-degree tilt.
 let seasonal=seasonal_declination_sine(season_month(),p.abundance.w)/sin(.40910518)*pos(i,p.dims.x).y;
 let temp=c.hydro.y+seasonal*12.;
 var unmanaged=1.;if p.options.w==1u {c.climate.y=max(0.,c.hydro.z)*(1.+seasonal*.25)*history_drought(i);if c.tags.x>=2u {unmanaged=clamp(1.-src[parent(i)].pools[24].w/max(land(environment[parent(i)]),.00001),0.,1.);}}
 let precipitation=unmanaged*max(0.,c.hydro.z)*.001*dt*(1.+seasonal*.25)*history_drought(i);
 let snow=select(0.,precipitation,temp<0.);let melt=min(c.water.z,max(temp,0.)*.025*dt);
 c.water.z+=snow-melt;let available=precipitation-snow+melt;
 let evaporation=min(available+c.water.x,max(temp+5.,0.)*.015*dt*unmanaged);
 c.budget.x=precipitation;c.budget.y=evaporation;c.life.w=0.;
 if c.tags.x<2u {c.water.x=max(0.,c.water.x+available-evaporation);}
 else {
 // Secondary lakes evaporate stored water too, rather than accumulating every
 // past wet month indefinitely. Rain is consumed before the existing reservoir.
 c.water.x=max(0.,c.water.x-max(0.,evaporation-available));
 let input=max(0.,available-evaporation);let infiltration=min(max(0.,100.-c.water.y),input*catalog[c.ids.x].a.y*.6);
 let release=c.water.y*.04*dt;c.water.y+=infiltration-release;
 let flood_release=select(0.,max(0.,c.water.x-max(0.,c.hydro.x-c.terrain.x))*.5,p.options.w==1u);c.water.x-=flood_release;
 let runoff=input-infiltration+release+flood_release;let retained=min(runoff,max(0.,c.hydro.x-c.terrain.x-c.water.x));
 c.water.x+=retained;c.life.w=runoff-retained;
 }
 c.climate.x=temp;terrain[i]=c;
}

// Farm claims already debit the ecological water ledger. Remove that same water
// from physical reservoirs before the next environmental aggregation.
@compute @workgroup_size(8,8)
fn reconcile_plots(@builtin(global_invocation_id) g:vec3<u32>) {
 let i=id(g,p.dims.x);let j=parent(i);let actual=environment[j].fields[7].w;
 let ratio=select(1.,clamp(src[j].pools[25].w/max(actual,1e-20),0.,1.),actual>0.);
 terrain[i].water=vec4(terrain[i].water.xyz*ratio,terrain[i].water.w);
 river_out[i]=vec4(river[i].xyz,river[i].w*ratio);
}
