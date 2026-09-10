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

struct Tile { surface:vec4<f32>, climate:vec4<f32>, water:vec4<f32>, ids:vec4<u32>, route:vec4<u32>, forcing:vec4<f32>, strata:vec4<f32>, rocks:vec4<u32> }
struct Uniforms { dims:vec4<u32>, center:vec4<f32>, east:vec4<f32>, north:vec4<f32>, counts:vec4<u32> }
struct Entry { a:vec4<f32>, b:vec4<f32>, c:vec4<f32>, d:vec4<f32>, ids:vec4<u32> }
@group(0) @binding(0) var<storage,read> planet:array<Cell>;
@group(0) @binding(1) var<storage,read> src:array<Tile>;
@group(0) @binding(2) var<storage,read_write> dst:array<Tile>;
@group(0) @binding(3) var<uniform> u:Uniforms;
@group(0) @binding(4) var<storage,read_write> changed:atomic<u32>;
@group(0) @binding(5) var<storage,read> catalog:array<Entry>;
const NONE:u32=0xffffffffu;
fn hash(x:u32)->u32 { var v=x; v=(v^(v>>16u))*0x7feb352du; v=(v^(v>>15u))*0x846ca68bu; return v^(v>>16u); }
fn rand(x:u32)->f32 { return f32(hash(x^u.dims.z)&0xffffffu)/16777216.; }
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
fn pos(i:u32)->vec3<f32> { let n=u.dims.y; return direction(i/(n*n),2.*(f32(i%n)+.5)/f32(n)-1.,2.*(f32((i/n)%n)+.5)/f32(n)-1.); }
fn index(d:vec3<f32>)->u32 {
 let a=abs(d); var f=0u; var uv=vec2(0.);
 if a.x>=a.y && a.x>=a.z { f=select(1u,0u,d.x>=0.);uv=d.yz/a.x; }
 else if a.y>=a.z { f=select(3u,2u,d.y>=0.);uv=d.xz/a.y; }
 else { f=select(5u,4u,d.z>=0.);uv=d.xy/a.z; }
 let xy=vec2<u32>(clamp(floor((uv+1.)*.5*f32(u.dims.y)),vec2(0.),vec2(f32(u.dims.y-1u))));
 return f*u.dims.y*u.dims.y+xy.y*u.dims.y+xy.x;
}
fn xy(i:u32)->vec2<u32> {return vec2(i%u.dims.x,i/u.dims.x);}
fn edge(i:u32)->bool {let q=xy(i);return any(q==vec2(0u))||any(q==vec2(u.dims.x-1u));}
fn adjacent(i:u32,k:u32)->u32 {let offsets=array<vec2<i32>,8>(vec2(-1,0),vec2(1,0),vec2(0,-1),vec2(0,1),vec2(-1,-1),vec2(1,-1),vec2(-1,1),vec2(1,1));let q=vec2<i32>(xy(i))+offsets[k];if any(q<vec2(0))||any(q>=vec2(i32(u.dims.x))){return NONE;}return u32(q.y)*u.dims.x+u32(q.x);}
fn sample_direction(i:u32)->vec3<f32> {let q=(vec2<f32>(xy(i))+.5)/f32(u.dims.x)-.5;return normalize(u.center.xyz+(u.east.xyz*q.x+u.north.xyz*q.y)*u.center.w/u.east.w);}
struct Macro { field:vec4<f32>, lake:vec2<f32>, land:u32, body:u32 }
fn macro_fields(d:vec3<f32>)->Macro {
 let a=abs(d);var f=0u;var uv=vec2(0.);
 if a.x>=a.y&&a.x>=a.z {f=select(1u,0u,d.x>=0.);uv=d.yz/a.x;}else if a.y>=a.z {f=select(3u,2u,d.y>=0.);uv=d.xz/a.y;}else{f=select(5u,4u,d.z>=0.);uv=d.xy/a.z;}
 let n=f32(u.dims.y);let q=(uv+1.)*.5*n-.5;let corner=floor(q);let t=fract(q);var height=vec4(0.);var lake=vec2(0.);var land=planet[index(d)].tags.x;var body=1u;var land_weight=-1.;var water_weight=-1.;
 for(var k=0u;k<4u;k++){let dx=f32(k%2u);let dy=f32(k/2u);let p=2.*(corner+vec2(dx,dy)+.5)/n-1.;let cell=planet[index(direction(f,p.x,p.y))];let weight=mix(1.-t.x,t.x,dx)*mix(1.-t.y,t.y,dy);height+=vec4(cell.terrain.x,cell.geology.x,cell.hydro.y,cell.hydro.z)*weight;
 if cell.water.x>.25 {lake+=vec2(cell.terrain.x+cell.water.x,1.)*weight;if weight>water_weight {body=select(cell.tags.x,1u,cell.tags.x>=2u);water_weight=weight;}}
 if cell.tags.x>=2u&&weight>land_weight {land=cell.tags.x;land_weight=weight;}
 }return Macro(height,lake,land,body);
}
@compute @workgroup_size(8,8)
fn generate(@builtin(global_invocation_id) g:vec3<u32>) {
 if any(g.xy>=vec2(u.dims.x)){return;}let i=g.y*u.dims.x+g.x;let d=sample_direction(i);let c=planet[index(d)];
 // World-space noise makes relief repeatable independently of patch orientation.
 let scale=u.east.w/max(8000.,u.center.w/f32(u.dims.x)*16.);let ridge=1.-abs(noise(d*scale)*2.-1.);
 let inherited=macro_fields(d);let fields=inherited.field;let relief=(ridge-.65)*(35.+220.*fields.y)+(noise(d*scale*4.)-.5)*65.;
 var h=fields.x+relief;let sea=select(0.,inherited.lake.x/max(inherited.lake.y,.000001),inherited.lake.y>0.);
 if inherited.lake.y==0. {h=max(h,1.);}
 let t=fields.z-(h-fields.x)*.0065;let rain=max(0.,fields.w);let runoff=rain*.001*clamp(.25+c.geology.x*.25,.1,.8);
 let water=max(0.,sea-h);let area=pow(u.center.w/f32(u.dims.x),2.)/pow(1.+pow(length((vec2<f32>(g.xy)+.5)/f32(u.dims.x)-.5)*u.center.w/u.east.w,2.),1.5);
 let outlet=edge(i)||water>0.;let spill=select(1e20,h,outlet);
 dst[i]=Tile(vec4(h,c.terrain.y,max(.02,c.terrain.z),area),vec4(t,rain,c.life.y,c.life.x),vec4(water,spill,0.,runoff*area),vec4(c.ids.xyz,select(inherited.land,select(inherited.body,c.tags.x,c.tags.x>=2u),water>0.)),vec4(NONE,select(NONE,0u,outlet),index(d),select(0u,1u,water>0.)),vec4(water*area,select(0.,f32(c.tags.y+1u),c.tags.y!=NONE),c.geology.z,max(0.,c.geology.w+h-c.terrain.x)),c.strata,vec4(c.ids.x,c.tags.z,c.tags.w,0u));
}
@compute @workgroup_size(8,8)
fn drainage(@builtin(global_invocation_id) g:vec3<u32>) {
 if any(g.xy>=vec2(u.dims.x)){return;}let i=g.y*u.dims.x+g.x;var c=src[i];if edge(i)||c.route.w==1u {dst[i]=c;return;}
 c.water.y=1e20;c.route.x=NONE;c.route.y=NONE;
 for(var k=0u;k<8u;k++){let j=adjacent(i,k);if j==NONE{continue;}let b=src[j];if b.route.y==NONE{continue;}let spill=max(c.surface.x,b.water.y);let rank=b.route.y+1u;
 let slope=max(0.,c.surface.x-b.water.y)/select(1.,1.41421356,k>=4u);
 var previous=-1.;if c.route.x!=NONE {let delta=abs(vec2<i32>(xy(i))-vec2<i32>(xy(c.route.x)));previous=max(0.,c.surface.x-src[c.route.x].water.y)/select(1.,1.41421356,all(delta==vec2(1)));}
 if spill<c.water.y||(spill==c.water.y&&(slope>previous||(slope==previous&&(rank<c.route.y||(rank==c.route.y&&hash(j)<hash(c.route.x)))))){c.water.y=spill;c.route.x=j;c.route.y=rank;}}
 if c.route.x!=src[i].route.x||c.route.y!=src[i].route.y||c.water.y!=src[i].water.y {atomicAdd(&changed,1u);}dst[i]=c;
}
@compute @workgroup_size(8,8)
fn flow(@builtin(global_invocation_id) g:vec3<u32>) {
 if any(g.xy>=vec2(u.dims.x)){return;}let i=g.y*u.dims.x+g.x;var c=src[i];var amount=c.water.w;
 for(var k=0u;k<8u;k++){let j=adjacent(i,k);if j!=NONE&&src[j].route.x==i{amount+=src[j].water.z;}}
 // One year of water input; storage is the local depression capacity.
 let retained=select(min(amount,max(0.,c.water.y-c.surface.x)*c.surface.w),0.,c.route.w==1u);
 c.water.x=select(retained/c.surface.w,c.water.x,c.route.w==1u);c.water.z=amount-retained;
 if abs(c.water.z-src[i].water.z)>.001 {atomicAdd(&changed,1u);}dst[i]=c;
}
@compute @workgroup_size(8,8)
fn habitat(@builtin(global_invocation_id) g:vec3<u32>) {
 if any(g.xy>=vec2(u.dims.x)){return;}let i=g.y*u.dims.x+g.x;var c=src[i];var slope=0.;
 for(var k=0u;k<8u;k++){let j=adjacent(i,k);if j!=NONE{slope=max(slope,abs(c.surface.x-src[j].surface.x)/sqrt(c.surface.w));}}
 c.surface.z=clamp(c.surface.z/(1.+slope*8.)+log(1.+c.water.z)*.025,.01,4.);
 // Keep rock inheritance, select soil by alluvial deposition and slope.
 if c.water.z>1e6&&u.counts.z>1u {c.ids.y=1u;}
 let offset=u.counts.x+u.counts.y+u.counts.z;var best=-1.;c.ids.z=NONE;
 for(var k=0u;k<u.counts.w;k++){let plant=catalog[offset+k];
 if c.climate.x<plant.a.x||c.climate.x>plant.a.y||c.climate.y<plant.a.z||c.climate.y>plant.a.w {continue;}
 if c.water.x>.25||c.climate.z<plant.b.x||plant.c.x>=3.|| (plant.ids.x==1u&&c.ids.w!=3u)|| (plant.ids.y!=3u&&plant.ids.y!=catalog[c.ids.x].ids.x) {continue;}
 let score=rand(i+k*7919u)*max(.01,plant.b.y);if score>best {best=score;c.ids.z=k;}}
 c.climate.w=select(clamp(c.climate.z*(1.-slope)*min(1.,c.climate.y/800.),0.,1.),0.,c.ids.z==NONE);dst[i]=c;
}
fn local_transfer(i:u32,j:u32)->f32 {
 let a=src[i];let b=src[j];if a.route.w==1u||b.route.w==1u||a.water.y!=b.water.y||a.water.x<=0.{return 0.;}
 let head=a.surface.x+a.water.x-max(a.surface.x,b.surface.x+b.water.x);if head<=max(.0005,abs(a.surface.x)*.0000005){return 0.;}
 return min(a.water.x*a.surface.w*.24,head*a.surface.w*b.surface.w/(a.surface.w+b.surface.w)*.48);
}
@compute @workgroup_size(8,8)
fn pools(@builtin(global_invocation_id) g:vec3<u32>) {
 if any(g.xy>=vec2(u.dims.x)){return;}let i=g.y*u.dims.x+g.x;var c=src[i];var net=0.;
 for(var k=0u;k<4u;k++){let j=adjacent(i,k);if j!=NONE{net+=local_transfer(j,i)-local_transfer(i,j);}}
 let delta=net/c.surface.w;c.water.x=max(0.,c.water.x+delta);if abs(delta)>max(.0005,abs(c.surface.x)*.0000005){atomicAdd(&changed,1u);}dst[i]=c;
}
