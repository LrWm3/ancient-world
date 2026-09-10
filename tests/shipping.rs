use ancient_world::{
    catalog::Catalog,
    config::Config,
    gpu::{ContextGpu, Generator},
};
#[test]
#[ignore = "requires hardware GPU"]
fn shipping_conserves_reservations_closures_and_checkpoint_continuation() {
    let mut g = Generator::new(
        pollster::block_on(ContextGpu::headless()).unwrap(),
        Config {
            resolution: 64,
            ecology_resolution: 64,
            seed: 7,
            ..Default::default()
        },
        Catalog::bundled().unwrap(),
    )
    .unwrap();
    g.run_epochs(1).unwrap();
    g.found_civilizations(5).unwrap();
    g.enable_society().unwrap();
    g.enable_politics().unwrap();
    g.enable_governance().unwrap();
    g.enable_shipping().unwrap();
    let terrain = g.snapshot().unwrap();
    assert!(
        g.civilizations
            .as_ref()
            .unwrap()
            .shipping
            .as_ref()
            .unwrap()
            .lanes
            .len()
            >= 3
    );
    for _ in 0..120 {
        g.advance_history(3).unwrap();
        if g.civilizations
            .as_ref()
            .unwrap()
            .cargo
            .iter()
            .any(|c| c.sea_lane.is_some())
        {
            break;
        }
    }
    let h = g.civilizations.as_ref().unwrap();
    h.validate(&terrain).unwrap(); // Includes actual connected lake-only routes and material ledgers.
    let cargo = h
        .cargo
        .iter()
        .find(|c| c.sea_lane.is_some())
        .expect("organic inter-island trade")
        .clone();
    assert_ne!(
        h.sites[cargo.from as usize].island,
        h.sites[cargo.to as usize].island
    );
    assert!(
        h.route_cost(cargo.from, cargo.to).is_none(),
        "shipping must not become an army route"
    );
    for (id, p) in h.shipping.as_ref().unwrap().ports.iter().enumerate() {
        let reserved: f32 = h
            .cargo
            .iter()
            .filter(|c| {
                c.sea_lane.is_some_and(|i| {
                    h.shipping.as_ref().unwrap().lanes[i as usize]
                        .ports
                        .contains(&(id as u32))
                })
            })
            .map(|c| c.kg)
            .sum();
        assert!(
            reserved <= p.capacity() + 0.01,
            "port capacity oversubscribed"
        );
    }
    let mut invalid = h.clone();
    invalid.shipping.as_mut().unwrap().lanes[0].cells[0] = h.sites[0].cell;
    assert!(invalid.validate(&terrain).is_err());
    let mut invalid = h.clone();
    invalid
        .cargo
        .iter_mut()
        .find(|c| c.sea_lane.is_some())
        .unwrap()
        .sea_lane = Some(u32::MAX);
    assert!(invalid.validate(&terrain).is_err());
    let mut invalid = h.clone();
    invalid.shipping.as_mut().unwrap().ports[0].assets[0] = f32::NAN;
    assert!(invalid.validate(&terrain).is_err());
    let lanes = h.shipping.as_ref().unwrap().lanes.len();
    for lane in 0..lanes {
        g.set_sea_lane_open(lane as u32, false).unwrap();
    }
    let h = g.civilizations.as_ref().unwrap();
    assert!(h
        .sea_quotes(&h.trade_distances().unwrap())
        .iter()
        .all(Option::is_none));
    let file = std::env::temp_dir().join(format!("shipping-{}.world", std::process::id()));
    g.save(&file).unwrap();
    let mut resumed = Generator::load(g.gpu.clone(), &file).unwrap();
    std::fs::remove_file(file).unwrap();
    g.advance_history(24).unwrap();
    for _ in 0..24 {
        resumed.advance_history(1).unwrap();
    }
    assert_eq!(
        serde_json::to_value(&g.civilizations).unwrap(),
        serde_json::to_value(&resumed.civilizations).unwrap()
    );
    let h = g.civilizations.as_ref().unwrap();
    assert!(h.cargo.iter().all(|c| c.sea_lane.is_none()));
    assert!(h.events.iter().any(|e| e.kind == "sea_arrival"
        && e.site == Some(cargo.to)
        && e.other == Some(cargo.from)
        && e.month == cargo.arrives));
    h.validate(&terrain).unwrap();
    for lane in 0..lanes {
        g.set_sea_lane_open(lane as u32, true).unwrap();
    }
    g.advance_history(120).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    assert!(h
        .events
        .iter()
        .any(|e| e.kind == "sea_arrival" && e.month > cargo.arrives + 24));
    h.validate(&terrain).unwrap();
}
