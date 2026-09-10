use ancient_world::gpu::{read_buffer, ContextGpu};
use wgpu::util::DeviceExt;

// Independent reference: numerically integrate the instantaneous dot product
// of the rotating surface normal and the sun, clipping the night side.
fn reference(y: f64, month: usize, tilt: f64) -> f64 {
    let longitude = (month as f64 - 2.) * std::f64::consts::TAU / 12.;
    let sun_y = tilt.sin() * longitude.sin();
    let steps = 16384;
    (0..steps)
        .map(|h| {
            let angle = (h as f64 + 0.5) * std::f64::consts::TAU / steps as f64;
            (y * sun_y + (1. - y * y).sqrt() * (1. - sun_y * sun_y).sqrt() * angle.cos()).max(0.)
        })
        .sum::<f64>()
        * std::f64::consts::PI
        / steps as f64
}

#[test]
#[ignore = "requires hardware GPU"]
fn solar_geometry_matches_rotating_surface_reference() {
    let gpu = pollster::block_on(ContextGpu::headless()).unwrap();
    let mut inputs: Vec<[f32; 4]> = Vec::new();
    for tilt in [0_f32, 23.44, 90.] {
        for month in 0..12 {
            // Equal-area latitude samples plus exact poles.
            for k in 0..=200 {
                inputs.push([-1. + k as f32 / 100., month as f32, tilt.to_radians(), 0.]);
            }
        }
    }
    let input = gpu
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&inputs),
            usage: wgpu::BufferUsages::STORAGE,
        });
    let output = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (inputs.len() * 4) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let source = format!(
        "{}\n{}",
        include_str!("../shaders/sunlight.wgsl"),
        r#"
@group(0) @binding(0) var<storage,read> inputs:array<vec4<f32>>;
@group(0) @binding(1) var<storage,read_write> results:array<f32>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id:vec3<u32>) {
 if id.x>=arrayLength(&inputs) {return;}
 let v=inputs[id.x];results[id.x]=ecological_sunlight(v.x,u32(v.y),v.z);
}"#
    );
    let module = gpu
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });
    let pipeline = gpu
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
    let bindings = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: input.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: output.as_entire_binding(),
            },
        ],
    });
    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_compute_pass(&Default::default());
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bindings, &[]);
        pass.dispatch_workgroups((inputs.len() as u32).div_ceil(64), 1, 1);
    }
    gpu.queue.submit(Some(encoder.finish()));
    let bytes = read_buffer(&gpu, &output, 0, (inputs.len() * 4) as u64).unwrap();
    let values: &[f32] = bytemuck::cast_slice(&bytes);
    let mut max_error = 0_f64;
    for (v, actual) in inputs.iter().zip(values) {
        let expected = reference(v[0] as f64, v[1] as usize, v[2] as f64);
        let error = (*actual as f64 - expected).abs();
        max_error = max_error.max(error);
        assert!(
            actual.is_finite() && *actual >= 0. && error < 2e-5,
            "{v:?}: {actual} vs {expected}"
        );
    }
    let at = |tilt: usize, month: usize, k: usize| values[(tilt * 12 + month) * 201 + k];
    for tilt in 0..3 {
        for month in 0..12 {
            let mean = (0..200)
                .map(|k| (at(tilt, month, k) + at(tilt, month, k + 1)) as f64 * 0.5)
                .sum::<f64>()
                / 200.;
            assert!((mean - std::f64::consts::PI / 4.).abs() < 0.001);
            for k in 0..=200 {
                assert!((at(tilt, month, k) - at(tilt, (month + 6) % 12, 200 - k)).abs() < 2e-5);
                if tilt == 0 {
                    assert!((at(tilt, month, k) - at(tilt, 0, k)).abs() < 2e-5);
                }
            }
        }
    }
    assert!(at(1, 6, 175) > at(1, 0, 175)); // July > January in north
    assert!(at(1, 6, 25) < at(1, 0, 25));
    assert_eq!(at(1, 11, 200), 0.); // north polar night
    assert!(at(1, 5, 200) > 1.);
    assert!(at(1, 2, 100) > at(1, 5, 100)); // equatorial equinox maximum
    println!(
        "{} GPU samples; maximum absolute quadrature error {max_error:.9}",
        values.len()
    );
}
