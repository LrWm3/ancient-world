// Deterministic regional surface refinement for the explorer and regional exports.
// Macro drainage and ecological inventories remain on their declared simulation grids.
fn regional_hash(x:u32)->u32 {var q=x^v.extra.w;q=(q^(q>>16u))*0x7feb352du;q=(q^(q>>15u))*0x846ca68bu;return q^(q>>16u);}
fn regional_lattice(q:vec3<i32>)->f32 {return f32(regional_hash(bitcast<u32>(q.x)*73856093u^bitcast<u32>(q.y)*19349663u^bitcast<u32>(q.z)*83492791u)&0xffffffu)/16777216.;}
fn regional_noise(q:vec3<f32>)->f32 {
 let i=vec3<i32>(floor(q));let f=fract(q);let t=f*f*(3.-2.*f);
 return mix(mix(mix(regional_lattice(i),regional_lattice(i+vec3(1,0,0)),t.x),mix(regional_lattice(i+vec3(0,1,0)),regional_lattice(i+vec3(1,1,0)),t.x),t.y),mix(mix(regional_lattice(i+vec3(0,0,1)),regional_lattice(i+vec3(1,0,1)),t.x),mix(regional_lattice(i+vec3(0,1,1)),regional_lattice(i+vec3(1,1,1)),t.x),t.y),t.z);
}
fn regional_direction(f:u32,u:f32,w:f32)->vec3<f32> {
 switch f {case 0u:{return normalize(vec3(1.,u,w));}case 1u:{return normalize(vec3(-1.,u,w));}case 2u:{return normalize(vec3(u,1.,w));}case 3u:{return normalize(vec3(u,-1.,w));}case 4u:{return normalize(vec3(u,w,1.));}default:{return normalize(vec3(u,w,-1.));}}
}
fn regional_base(d:vec3<f32>)->vec3<f32> {
 let a=abs(d);var f=0u;var uv=vec2(0.);
 if a.x>=a.y&&a.x>=a.z {f=select(1u,0u,d.x>=0.);uv=d.yz/a.x;}else if a.y>=a.z {f=select(3u,2u,d.y>=0.);uv=d.xz/a.y;}else{f=select(5u,4u,d.z>=0.);uv=d.xy/a.z;}
 let n=f32(v.dims.x);let q=(uv+1.)*.5*n-.5;let corner=floor(q);let fraction=fract(q);let t=fraction*fraction*(3.-2.*fraction);let center=cells[index(d)];
 var sum=vec3(0.);var weight=0.;
 for(var k=0u;k<4u;k++){let dx=f32(k%2u);let dy=f32(k/2u);let xy=2.*(corner+vec2(dx,dy)+.5)/n-1.;
 let c=cells[index(regional_direction(f,xy.x,xy.y))];
 {
 let w=mix(1.-t.x,t.x,dx)*mix(1.-t.y,t.y,dy);
 let dry=select(0.,1.,c.tags.x>=2u&&c.water.x<=.25);
 sum+=vec3(c.terrain.x+c.water.x,c.geology.x*dry,c.water.w)*w;weight+=w;
 }}
 return select(vec3(center.terrain.x,center.geology.x,center.water.w),sum/max(weight,.00001),weight>0.);
}
fn regional_height(d:vec3<f32>)->f32 {
 let c=cells[index(d)];
 let base=regional_base(d);let frequency=f32(v.dims.x)*1.5;
 let broad=regional_noise(d*frequency);let fine=regional_noise(d*frequency*3.1+17.);
 let ridges=1.-abs(broad*2.-1.);
 let roughness=(25.+150.*base.y)/(1.+log(1.+max(0.,base.z))*.3);
 let land_weight=clamp(base.y*4.,0.,1.);
 return base.x+((ridges-.65)*roughness+(fine-.5)*roughness*.35)*land_weight;
}
// Interpolate visual attributes as well as height: categorical simulation cells
// remain inspectable, while regional rendering does not turn them into square tiles.
fn regional_color(d:vec3<f32>)->vec3<f32> {
 let a=abs(d);var f=0u;var uv=vec2(0.);
 if a.x>=a.y&&a.x>=a.z {f=select(1u,0u,d.x>=0.);uv=d.yz/a.x;}else if a.y>=a.z {f=select(3u,2u,d.y>=0.);uv=d.xz/a.y;}else{f=select(5u,4u,d.z>=0.);uv=d.xy/a.z;}
 let n=f32(v.dims.x);let q=(uv+1.)*.5*n-.5;let corner=floor(q);let fraction=fract(q);let t=fraction*fraction*(3.-2.*fraction);var color=vec3(0.);
 for(var k=0u;k<4u;k++){let dx=f32(k%2u);let dy=f32(k/2u);let xy=2.*(corner+vec2(dx,dy)+.5)/n-1.;
 let c=cells[index(regional_direction(f,xy.x,xy.y))];let w=mix(1.-t.x,t.x,dx)*mix(1.-t.y,t.y,dy);color+=color_for(c)*w;
 }
 return color;
}
