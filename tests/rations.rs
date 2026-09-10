use ancient_world::gpu::{read_buffer, ContextGpu};
use ancient_world::society::Demography;
use wgpu::util::DeviceExt;

#[test]
fn older_demography_defaults_to_equal_rations() {
    let d: Demography =
        serde_json::from_str(r#"{"ages":[1,2,3,1],"crops":[0,0,2,1],"health":[0,0,0,0]}"#).unwrap();
    assert_eq!(d.ration_priority, [0.; 4]);
    assert_eq!(d.ration_eaten, [0.; 4]);
    assert_eq!(std::mem::size_of::<Demography>(), 112);
}

#[test]
#[ignore = "requires hardware GPU"]
fn gpu_rations_are_bounded_conservative_and_protect_prioritized_cohorts() {
    let gpu = pollster::block_on(ContextGpu::headless()).unwrap();
    let source = include_str!("../shaders/society.wgsl");
    let kernel = source
        .split("fn allocate_rations")
        .nth(1)
        .unwrap()
        .split("fn demographic_month")
        .next()
        .unwrap();
    let shader = format!(
        "fn allocate_rations{kernel}
struct Fixture {{ need:vec4<f32>, priority:vec4<f32> }}
@group(0) @binding(0) var<storage,read> cases:array<Fixture>;
@group(0) @binding(1) var<storage,read_write> result:array<vec4<f32>>;
@compute @workgroup_size(1) fn main(@builtin(global_invocation_id) id:vec3<u32>) {{
 let c=cases[id.x]; result[id.x]=vec4(allocate_rations(c.need.xyz,c.need.w,c.priority.xyz),0.);
}}"
    );
    let mut cases = vec![];
    for need in [
        [100., 200., 100.],
        [0., 200., 0.],
        [1., 10000., 2.],
        [0.; 3],
    ] {
        for food in [0., 1., 50., 200., 400., 100000.] {
            for priority in [[0.; 3], [3., 0., 3.], [0., 3., 0.]] {
                cases.push([
                    need[0],
                    need[1],
                    need[2],
                    food,
                    priority[0],
                    priority[1],
                    priority[2],
                    0.,
                ]);
            }
        }
    }
    let input = gpu
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&cases),
            usage: wgpu::BufferUsages::STORAGE,
        });
    let output = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: cases.len() as u64 * 16,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let module = gpu
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(shader.into()),
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
    let group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
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
        pass.set_bind_group(0, &group, &[]);
        pass.dispatch_workgroups(cases.len() as u32, 1, 1);
    }
    gpu.queue.submit(Some(encoder.finish()));
    let bytes = read_buffer(&gpu, &output, 0, cases.len() as u64 * 16).unwrap();
    let results = bytemuck::cast_slice::<u8, [f32; 4]>(&bytes);
    for (c, r) in cases.iter().zip(results) {
        let need = c[..3].iter().sum::<f32>();
        let food = c[3].min(need);
        let sum = r[..3].iter().sum::<f32>();
        assert!((sum - food).abs() < 0.002, "{c:?}: {r:?}");
        for k in 0..3 {
            assert!(r[k].is_finite() && r[k] >= 0. && r[k] <= c[k] + 0.001);
            if need > 0. {
                let equal = c[k] * food / need;
                assert!(r[k] + 0.001 >= equal * 0.5);
                if c[4..7] == [0.; 3] {
                    assert!((r[k] - equal).abs() < 0.002);
                }
            }
        }
    }
    // Equal needs, finite shortage: protecting children/elders costs adult rations.
    let idx = cases
        .iter()
        .position(|c| *c == [100., 200., 100., 200., 3., 0., 3., 0.])
        .unwrap();
    let r = results[idx];
    assert!(r[0] > 50. && r[2] > 50. && r[1] < 100.);
}
