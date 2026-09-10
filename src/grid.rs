//! The CPU copy of the cube-sphere mapping is for picking, export and tests.
//! Simulation topology and geometry are evaluated in WGSL.
pub fn direction(face: u32, u: f32, v: f32) -> [f32; 3] {
    let p = match face {
        0 => [1., u, v],
        1 => [-1., u, v],
        2 => [u, 1., v],
        3 => [u, -1., v],
        4 => [u, v, 1.],
        _ => [u, v, -1.],
    };
    let l = p.iter().map(|x| x * x).sum::<f32>().sqrt();
    p.map(|x| x / l)
}
pub fn index(p: [f32; 3], n: u32) -> u32 {
    let a = p.map(f32::abs);
    let (f, u, v) = if a[0] >= a[1] && a[0] >= a[2] {
        (if p[0] >= 0. { 0 } else { 1 }, p[1] / a[0], p[2] / a[0])
    } else if a[1] >= a[2] {
        (if p[1] >= 0. { 2 } else { 3 }, p[0] / a[1], p[2] / a[1])
    } else {
        (if p[2] >= 0. { 4 } else { 5 }, p[0] / a[2], p[1] / a[2])
    };
    let coord = |x: f32| (((x + 1.) * 0.5 * n as f32).floor() as i32).clamp(0, n as i32 - 1) as u32;
    f * n * n + coord(v) * n + coord(u)
}
pub fn cell_direction(i: u32, n: u32) -> [f32; 3] {
    direction(
        i / (n * n),
        2. * ((i % n) as f32 + 0.5) / n as f32 - 1.,
        2. * (((i / n) % n) as f32 + 0.5) / n as f32 - 1.,
    )
}
pub fn neighbor(i: u32, n: u32, dx: i32, dy: i32) -> u32 {
    let f = i / (n * n);
    let x = (i % n) as i32 + dx;
    let y = ((i / n) % n) as i32 + dy;
    let u = 2. * (x as f32 + 0.5) / n as f32 - 1.;
    let v = 2. * (y as f32 + 0.5) / n as f32 - 1.;
    index(direction(f, u, v), n)
}
pub fn solid_angle(i: u32, n: u32) -> f64 {
    let x = (i % n) as f64;
    let y = ((i / n) % n) as f64;
    let n = n as f64;
    let f = |u: f64, v: f64| (u * v).atan2((u * u + v * v + 1.).sqrt());
    let (u0, u1, v0, v1) = (
        2. * x / n - 1.,
        2. * (x + 1.) / n - 1.,
        2. * y / n - 1.,
        2. * (y + 1.) / n - 1.,
    );
    f(u1, v1) - f(u0, v1) - f(u1, v0) + f(u0, v0)
}
