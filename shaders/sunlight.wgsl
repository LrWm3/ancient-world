// Circular orbit, twelve equal months: March/September equinoxes,
// June/December solstices. Month zero is January. Representative days,
// not integration over a whole month. Tilt is in radians.
fn seasonal_declination_sine(month:u32, tilt:f32)->f32 {
 return sin(tilt)*sin((f32(month%12u)-2.)*0.5235987755982988);
}
// Daily mean positive solar zenith cosine, normalized to equatorial equinox
// (1/pi). Latitude input is sin(latitude), i.e. the unit position's y.
// Explicit polar cases avoid division by zero and invalid acos arguments.
fn ecological_sunlight(latitude_sine:f32, month:u32, tilt:f32)->f32 {
 let latitude=clamp(latitude_sine,-1.,1.);
 let declination=seasonal_declination_sine(month,tilt);
 let a=latitude*declination;
 let b=sqrt(max(0.,1.-latitude*latitude))*sqrt(max(0.,1.-declination*declination));
 if a>=b {return 3.141592653589793*max(a,0.);}
 if a<=-b {return 0.;}
 let sunset=acos(clamp(-a/b,-1.,1.));
 return max(0.,a*sunset+b*sin(sunset));
}
