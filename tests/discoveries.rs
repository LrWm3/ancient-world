use ancient_world::{
    catalog::Catalog,
    config::Config,
    expeditions::{Objective, Phase, Rules},
    gpu::{ContextGpu, Generator},
};
fn world() -> Generator {
    let mut g = Generator::new(
        pollster::block_on(ContextGpu::headless()).unwrap(),
        Config {
            resolution: 64,
            ecology_resolution: 64,
            seed: 7,
            // Specimen accounting requires repeated, deliberately funded voyages.
            island_phosphorus_scale: 1.,
            settlement_plot_hectares: 5000.,
            crop_yield_scale: 1.,
            ..Default::default()
        },
        Catalog::bundled().unwrap(),
    )
    .unwrap();
    g.run_epochs(1).unwrap();
    g.found_civilizations(5).unwrap();
    // Specimen transport/research is the subject here, not competition with family care
    // for the tool-making workforce during the twenty-year economic warm-up.
    g.civilizations
        .as_mut()
        .unwrap()
        .set_domestic_households(false)
        .unwrap();
    // These fixtures isolate repeated specimen/rescue transfers under the legacy farm control.
    g.set_diversified_farming(false).unwrap();
    g.enable_society().unwrap();
    g.enable_politics().unwrap();
    g.enable_governance().unwrap();
    g.enable_shipping().unwrap();
    g.enable_expeditions().unwrap();
    g.configure_expeditions(Rules {
        automatic: false,
        hazard_scale: 0.,
        ..Default::default()
    })
    .unwrap();
    g.advance_history(240).unwrap();
    g.enable_discoveries().unwrap();
    g
}
fn launch(g: &mut Generator, goal: Objective) -> (u32, u32) {
    let cells = g.snapshot().unwrap();
    let routes = g
        .civilizations
        .as_ref()
        .unwrap()
        .expeditions
        .as_ref()
        .unwrap()
        .routes
        .clone();
    for (i, r) in routes.iter().enumerate() {
        let cell = &cells[*r.cells.last().unwrap() as usize];
        if goal == Objective::Ecology
            && (cell.life[0] <= 0.15 || !(-5. ..=40.).contains(&cell.climate[0]))
        {
            continue;
        }
        if goal == Objective::Geology && cell.geology[0] <= 0. {
            continue;
        }
        match g.launch_expedition(i as u32, goal, None) {
            Ok(id) => return (id, r.travel_months),
            Err(error) => eprintln!("route {i}: {error:#}"),
        }
    }
    panic!("fixture requires a wealthy sponsor and suitable destination")
}
#[test]
#[ignore = "requires hardware GPU"]
fn study_precedes_production_and_checkpoint_preserves_consumption() {
    let mut g = world();
    let (id, travel) = launch(&mut g, Objective::Ecology);
    g.advance_history(2 * travel + 6).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    let x = h.expeditions.as_ref().unwrap();
    let d = x.discoveries.as_ref().unwrap();
    assert_eq!(x.voyages[id as usize].phase, Phase::Returned);
    assert!(d.collected[0] > 1.5);
    assert_eq!(d.remedy_made, 0.);
    assert_eq!(d.studied[0], 0.);
    let site = x.voyages[id as usize].origin as usize;
    // Paused workshops retain samples without producing output or closing local markets.
    g.set_specimen_workshop_open(site as u32, false).unwrap();
    g.advance_history(6).unwrap();
    assert_eq!(
        g.civilizations
            .as_ref()
            .unwrap()
            .expeditions
            .as_ref()
            .unwrap()
            .discoveries
            .as_ref()
            .unwrap()
            .studied[0],
        0.
    );
    g.set_specimen_workshop_open(site as u32, true).unwrap();
    g.civilizations.as_mut().unwrap().sites[site]
        .demography
        .health[0] = 0.3;
    g.advance_history(3).unwrap();
    let d = g
        .civilizations
        .as_ref()
        .unwrap()
        .expeditions
        .as_ref()
        .unwrap()
        .discoveries
        .as_ref()
        .unwrap();
    assert!(d.studied[0] > 0. && d.studied[0] < 1.5);
    assert_eq!(d.remedy_made, 0.);
    let path = std::env::temp_dir().join(format!("specimen-resume-{}.world", std::process::id()));
    g.save(&path).unwrap();
    let mut b = Generator::load(g.gpu.clone(), &path).unwrap();
    std::fs::remove_file(path).unwrap();
    g.advance_history(48).unwrap();
    for _ in 0..48 {
        b.advance_history(1).unwrap();
    }
    assert_eq!(
        serde_json::to_value(&g.civilizations).unwrap(),
        serde_json::to_value(&b.civilizations).unwrap()
    );
    let h = g.civilizations.as_ref().unwrap();
    let x = h.expeditions.as_ref().unwrap();
    let d = x.discoveries.as_ref().unwrap();
    assert!(d.remedy_made > 0. && d.remedy_used > 0. && d.worker_months > 0.);
    assert!(d.residuals(x).iter().all(|v| v.abs() < 1e-8));
    h.validate(&g.snapshot().unwrap()).unwrap();
    let mut bad = h.clone();
    bad.expeditions
        .as_mut()
        .unwrap()
        .discoveries
        .as_mut()
        .unwrap()
        .workshops[0]
        .remedy += 1.;
    assert!(bad.validate(&g.snapshot().unwrap()).is_err());
}
#[test]
#[ignore = "requires hardware GPU"]
fn accessible_mineral_stock_depletes_and_fertilizer_uses_real_phosphorus() {
    let mut g = world();
    let (id, travel) = launch(&mut g, Objective::Geology);
    g.advance_history(travel + 3).unwrap();
    let h = g.civilizations.as_mut().unwrap();
    let d = h
        .expeditions
        .as_mut()
        .unwrap()
        .discoveries
        .as_mut()
        .unwrap();
    let source = &mut d.sources[0];
    // Controlled small accessible outcrop; already collected material remains accounted.
    source.remaining[1] = 0.125;
    source.initial[1] = source.collected[1] + 0.125;
    g.advance_history(travel + 30).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    let x = h.expeditions.as_ref().unwrap();
    let d = x.discoveries.as_ref().unwrap();
    assert_eq!(d.sources[0].remaining[1], 0.);
    assert!(d.phosphorus_applied > 0., "workshops: {:#?}", d.workshops);
    // Only actually processed crust releases P; the completion tolerance must
    // neither round study inventories up nor produce nutrients from study waste.
    assert!((d.phosphorus_applied - d.processed[1] * 0.08).abs() < 1e-10);
    assert!((d.studied[1] - 1.5).abs() < 1e-6);
    assert!(d.residuals(x).iter().all(|v| v.abs() < 1e-8));
    let collected = d.collected[1];
    let route = x.voyages[id as usize].route;
    g.advance_history(60).unwrap();
    g.launch_expedition(route, Objective::Geology, None)
        .unwrap();
    g.advance_history(travel * 2 + 8).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    let d = h
        .expeditions
        .as_ref()
        .unwrap()
        .discoveries
        .as_ref()
        .unwrap();
    assert_eq!(collected, d.collected[1]);
    h.validate(&g.snapshot().unwrap()).unwrap();
}
#[test]
#[ignore = "requires hardware GPU"]
fn rescue_moves_specimens_and_total_loss_discards_them() {
    let mut g = world();
    let (id, travel) = launch(&mut g, Objective::Ecology);
    g.advance_history(travel + 2).unwrap();
    let h = g.civilizations.as_mut().unwrap();
    let e = &mut h.expeditions.as_mut().unwrap().voyages[id as usize];
    let sample = e.samples[0];
    assert!(sample > 0.);
    let route = e.route;
    e.phase = Phase::Stranded;
    e.due = h.month + 1000;
    let origin = e.origin as usize;
    // This fixture tests specimen custody, not whether a sponsor can afford two
    // consecutive voyages. Fund the rescue with existing neighboring tool stocks.
    let mut needed =
        (25. + h.sites[origin].stocks.stock[0] * 0.5 - h.sites[origin].economy.goods[3]).max(0.);
    for donor in 0..h.sites.len() {
        if donor == origin {
            continue;
        }
        let transfer = needed.min(h.sites[donor].economy.goods[3]);
        h.sites[donor].economy.goods[3] -= transfer;
        h.sites[origin].economy.goods[3] += transfer;
        needed -= transfer;
    }
    assert!(needed < 0.001, "fixture needs existing rescue equipment");
    // Explicit rescue capital, transferred from existing town cash; no new money.
    let controller = h.controller(origin as u32) as usize;
    let money_before = h
        .sites
        .iter()
        .map(|s| s.economy.finance[0] as f64)
        .sum::<f64>()
        + h.society.as_ref().unwrap().councils[controller].treasury;
    for donor in 0..h.sites.len() {
        let treasury = h.society.as_ref().unwrap().councils[controller].treasury;
        let needed = (1201. - treasury).max(0.);
        let pool = &mut h.sites[donor].economy.finance[0];
        let before = *pool;
        *pool = (*pool as f64 - needed.min(*pool as f64)).max(0.) as f32;
        h.society.as_mut().unwrap().councils[controller].treasury += before as f64 - *pool as f64;
    }
    assert!(h.society.as_ref().unwrap().councils[controller].treasury >= 1200.);
    let money_after = h
        .sites
        .iter()
        .map(|s| s.economy.finance[0] as f64)
        .sum::<f64>()
        + h.society.as_ref().unwrap().councils[controller].treasury;
    assert!((money_after - money_before).abs() < 1e-8);
    let rescue = g
        .launch_expedition(route, Objective::Rescue, Some(id))
        .unwrap();
    g.advance_history(travel).unwrap();
    let h = g.civilizations.as_mut().unwrap();
    let x = h.expeditions.as_mut().unwrap();
    assert_eq!(x.voyages[id as usize].samples, [0.; 2]);
    assert_eq!(x.voyages[rescue as usize].samples[0], sample);
    // Keep all transferred people and specimens on an isolated vessel with no provisions.
    let e = &mut x.voyages[rescue as usize];
    h.sites[e.origin as usize].stocks.stock[1] += e.food;
    e.food = 0.;
    e.due = h.month + 1000;
    g.advance_history(17).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    let x = h.expeditions.as_ref().unwrap();
    assert_eq!(x.voyages[rescue as usize].phase, Phase::Lost);
    assert_eq!(x.discoveries.as_ref().unwrap().discarded[0], sample);
    h.validate(&g.snapshot().unwrap()).unwrap();
}
#[test]
#[ignore = "requires hardware GPU"]
fn healthy_towns_keep_samples_until_treatment_is_needed() {
    let mut g = world();
    let (id, travel) = launch(&mut g, Objective::Ecology);
    g.advance_history(travel * 2 + 30).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    let x = h.expeditions.as_ref().unwrap();
    let d = x.discoveries.as_ref().unwrap();
    assert!(d.remedy_made > 0. && d.remedy_made < 0.5);
    assert_eq!(d.remedy_used, 0.);
    let site = x.voyages[id as usize].origin as usize;
    let workshop = d
        .workshops
        .iter()
        .find(|w| w.site as usize == site)
        .unwrap();
    // Typed botanicals now share the organic manifest: only the remaining resin
    // can become medicine. Healthy-town processing must still leave resin stored.
    assert!(workshop.samples[0] > 0., "{workshop:#?}");
    assert!(workshop.botanicals.received.iter().sum::<f64>() > 0.);
    let before = d.processed[0];
    g.civilizations.as_mut().unwrap().sites[site]
        .demography
        .health[0] = 0.3;
    g.advance_history(12).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    let d = h
        .expeditions
        .as_ref()
        .unwrap()
        .discoveries
        .as_ref()
        .unwrap();
    assert!(d.remedy_used > 0. && d.processed[0] > before);
    h.validate(&g.snapshot().unwrap()).unwrap();
}
