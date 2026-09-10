struct View { dims:vec4<u32>, camera:vec4<f32>, pan:vec4<f32>, counts:vec4<u32>, extra:vec4<u32> }
struct Entry { a:vec4<f32>,b:vec4<f32>,c:vec4<f32>,d:vec4<f32>,ids:vec4<u32> }
@group(0) @binding(0) var<storage,read> cells:array<Cell>;
@group(0) @binding(1) var<storage,read> catalog:array<Entry>;
@group(0) @binding(2) var<uniform> v:View;
@group(0) @binding(3) var output:texture_storage_2d<rgba8unorm,write>;
struct Eco { pools:array<vec4<f32>,38> }
@group(0) @binding(4) var<storage,read> ecology:array<Eco>;
const PI:f32=3.14159265359;
fn index(d:vec3<f32>)->u32 {
 let a=abs(d);var f=0u;var uv=vec2(0.);
 if a.x>=a.y && a.x>=a.z {f=select(1u,0u,d.x>=0.);uv=d.yz/a.x;}else if a.y>=a.z {f=select(3u,2u,d.y>=0.);uv=d.xz/a.y;}else{f=select(5u,4u,d.z>=0.);uv=d.xy/a.z;}
 let xy=vec2<u32>(clamp(floor((uv+1.)*.5*f32(v.dims.x)),vec2(0.),vec2(f32(v.dims.x-1u))));return f*v.dims.x*v.dims.x+xy.y*v.dims.x+xy.x;
}
fn palette(x:f32)->vec3<f32> {return .48+.38*cos(vec3(0.,2.,4.)+x*6.28);}
fn terrain(c:Cell)->vec3<f32> {
 if c.tags.x<2u {return mix(vec3(.025,.10,.18),vec3(.08,.36,.43),clamp(1.-c.water.x/2500.,0.,1.));}
 if c.water.x>.25 {return vec3(.08,.36,.52);}
 let bio=v.counts.x+v.counts.y+v.counts.z+v.counts.w;
 var color=mix(vec3(.49,.43,.28),vec3(.12,.32,.19),c.life.x);
 if c.ids.w<v.extra.x {color=mix(vec3(.49,.43,.28),catalog[bio+c.ids.w].c.xyz,.4+c.life.x*.6);}
 color=mix(color,vec3(.62,.59,.52),smoothstep(1800.,6000.,c.terrain.x));
 if c.water.z>.1 {color=mix(color,vec3(.89,.94,.95),min(c.water.z,1.));}
 if c.water.w>2500. {color=mix(color,vec3(.15,.55,.68),.75);}
 return color;
}
fn color_for(c:Cell)->vec3<f32> {
 let water=c.tags.x<2u;let layer=v.dims.w;
 switch layer {
  case 0u:{return terrain(c);}
  case 1u:{if water{return mix(vec3(.02,.06,.2),vec3(.1,.5,.62),clamp((c.terrain.x+4000.)/4120.,0.,1.));}return mix(vec3(.2,.4,.22),vec3(.95,.9,.77),clamp(c.terrain.x/6000.,0.,1.));}
  case 2u:{return palette(f32(c.routing.w)*.618);}
  case 3u:{if water || c.water.x>.25 {return terrain(c); }return catalog[c.ids.x].c.xyz;}
  case 4u:{return mix(vec3(.035,.045,.065),vec3(.95,.65,.2),clamp(c.geology.z*2.,0.,1.));}
  case 5u:{return palette(f32(c.ids.y)*.618);}
  case 6u:{return mix(vec3(.16,.35,.8),vec3(.98,.3,.12),clamp((c.hydro.y+30.)/75.,0.,1.));}
  case 7u:{return mix(vec3(.68,.42,.2),vec3(.13,.65,.8),clamp(c.hydro.z/3000.,0.,1.));}
  case 8u:{return mix(vec3(.2,.45,.8),vec3(.9,.55,.25),clamp(c.climate.w/35.+.5,0.,1.));}
  case 9u:{return mix(vec3(.08,.12,.16),vec3(.92,.97,1.),clamp(c.water.z/2.,0.,1.));}
  case 10u:{if c.routing.z!=0xffffffffu {return palette(f32(c.routing.z)*.618);}return mix(vec3(.08,.12,.18),vec3(.8,.72,.45),clamp(f32(c.routing.y)/300.,0.,1.));}
  case 11u:{if water{return vec3(.025,.12,.2);}return mix(vec3(.12,.16,.14),vec3(.22,.8,1.),clamp(log(1.+c.water.w)/10.,0.,1.));}
  case 12u:{return mix(vec3(.14,.18,.16),vec3(.15,.65,.9),clamp(log(1.+c.water.x)/6.,0.,1.));}
  case 13u:{if water{return vec3(.025,.12,.2);}if c.ids.w<v.extra.x{return catalog[v.counts.x+v.counts.y+v.counts.z+v.counts.w+c.ids.w].c.xyz;}return vec3(.5);}
  default:{return mix(vec3(.4,.3,.2),vec3(.1,.65,.28),c.life.x);}
 }
}
fn eco_color(id:u32,layer:u32)->vec3<f32> {
 let n=v.dims.x;let m=v.extra.z;let r=n/m;let j=id/(n*n)*m*m+(id/n%n)/r*m+(id%n)/r;let e=ecology[j];var value=0.;
 switch layer {
 case 15u:{value=e.pools[28].x*10.;}case 16u:{value=e.pools[28].y*100.;}
 case 17u:{value=e.pools[17].y*50.;}case 18u:{value=e.pools[17].z*250.;}
 case 19u:{return select(select(vec3(.2,.6,.3),vec3(.9,.5,.1),e.pools[29].w==1.),vec3(.7,.2,.8),e.pools[29].w==2.);}
 case 20u,21u,22u,23u,24u:{value=log(1.+e.pools[layer-20u].x*10.);}
 case 25u:{for(var k=5u;k<17u;k++){value+=e.pools[k].x;}value=log(1.+value*100.);}
 case 26u:{return palette(atan2(e.pools[29].y,e.pools[29].x)/6.283185);}
 case 27u:{value=e.pools[29].z*5.;}case 28u:{value=e.pools[21].z*10.;}
 case 29u:{value=e.pools[23].x*100.;}default:{value=e.pools[18].x;}
 }
 return mix(vec3(.025,.04,.08),vec3(.25,.9,.65),clamp(value,0.,1.));
}
@compute @workgroup_size(8,8)
fn render(@builtin(global_invocation_id) g:vec3<u32>) {
 if g.x>=v.dims.y || g.y>=v.dims.z {return;}
 let size=vec2<f32>(v.dims.yz);let uv=(vec2<f32>(g.xy)+.5)/size;
 var d=vec3(0.);var shade=1.;var rim=0.;
 if v.camera.w>.5 {
  let xy=(uv-.5)*2.*vec2(size.x/size.y,1.)/v.camera.z;
  let r=dot(xy,xy);
  if r>1. {let glow=exp(-(sqrt(r)-1.)*18.)*.12;textureStore(output,vec2<i32>(g.xy),vec4(vec3(.023,.036,.055)+vec3(.1,.3,.4)*glow,1.));return;}
  let local=vec3(xy.x,-xy.y,sqrt(1.-r));
  let pitched=vec3(local.x,local.y*cos(v.camera.y)+local.z*sin(v.camera.y),-local.y*sin(v.camera.y)+local.z*cos(v.camera.y));
  d=vec3(pitched.x*cos(v.camera.x)+pitched.z*sin(v.camera.x),pitched.y,-pitched.x*sin(v.camera.x)+pitched.z*cos(v.camera.x));
  shade=.5+.5*max(0.,dot(local,normalize(vec3(-.4,.5,1.))));rim=pow(1.-local.z,4.);
 } else {
  let q=(uv-.5)/v.camera.z+.5+v.pan.xy;let lon=(q.x-.5)*2.*PI;let lat=(.5-q.y)*PI;
  if abs(lat)>PI*.5 {textureStore(output,vec2<i32>(g.xy),vec4(.023,.036,.055,1.));return;}
  d=vec3(cos(lat)*sin(lon),sin(lat),cos(lat)*cos(lon));
 }
 let id=index(d);var c=cells[id];
 if v.camera.z>2. && (v.dims.w==0u||v.dims.w==1u) {c.terrain.x=mix(c.terrain.x,regional_height(d),smoothstep(2.,4.,v.camera.z));}
 var color=color_for(c)*shade;
 if v.camera.z>2. && (v.dims.w==0u||v.dims.w==1u) {color=mix(color,regional_color(d)*shade,smoothstep(2.,4.,v.camera.z));}
 if v.dims.w>=15u {color=eco_color(id,v.dims.w)*shade;}
 if v.dims.w==0u || v.dims.w==1u || v.dims.w==3u {
  let tangent=normalize(cross(d,vec3(.01,1.,.01)));let other=cells[index(normalize(d+tangent*2./f32(v.dims.x)))];
  var contrast=(c.terrain.x-other.terrain.x)*.0004;
  if v.camera.z>2. && v.dims.w!=3u {
   let step=2./f32(v.dims.x)/min(v.camera.z,16.);
   contrast=(regional_height(d)-regional_height(normalize(d+tangent*step)))*.003;
  }
  color*=clamp(1.+contrast,.65,1.25);
 }
 color=mix(color,vec3(.24,.55,.7),rim*.3);
 if id==v.extra.y {color=mix(color,vec3(1.,.83,.36),.75);}
 textureStore(output,vec2<i32>(g.xy),vec4(color,1.));
}
