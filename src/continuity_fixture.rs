//! Small generated fixture shared by the six continuity integration tests.
pub(crate) fn world() -> crate::gpu::Generator {
    use crate::{
        catalog::Catalog,
        config::Config,
        gpu::{ContextGpu, Generator},
    };
    let mut g = Generator::new(
        pollster::block_on(ContextGpu::headless()).unwrap(),
        Config {
            resolution: 64,
            ecology_resolution: 16,
            seed: 17,
            ..Default::default()
        },
        Catalog::bundled().unwrap(),
    )
    .unwrap();
    g.found_civilizations(16).unwrap();
    g.enable_society().unwrap();
    g.enable_politics().unwrap();
    g.enable_governance().unwrap();
    g.enable_offices().unwrap();
    g
}
