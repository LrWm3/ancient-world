use ancient_world::{
    civilization::{Civilization, History, Site},
    economy::{Economy, EconomyCatalog},
    society::{Demography, Route, Society},
};
fn network() -> History {
    History {
        trade_contact: Default::default(),
        resolution: None,
        person_duties: Default::default(),
        service_allocation: Default::default(),
        domestic: None,
        named_demography: None,
        military: Default::default(),
        participation: None,
        territorial_history: vec![],
        enterprises: None,
        experimental_tool_reserves: Default::default(),
        resources: None,
        offices: None,
        culture: None,
        farming_mode: None,
        version: 2,
        seed: 42,
        terrain_resolution: 16,
        source_epoch: 0,
        month: 0,
        civilizations: (0..3)
            .map(|id| Civilization {
                language: None,
                id,
                name: format!("State {id}"),
                leader: 0,
            })
            .collect(),
        sites: (0..3)
            .map(|id| Site {
                id,
                civilization: id,
                island: 0,
                cell: id,
                name: format!("Site {id}"),
                founded: 0,
                abandoned: false,
                lifecycle: Default::default(),
                stocks: bytemuck::Zeroable::zeroed(),
                economy: Economy {
                    policy: [0., 0., 0., 1.],
                    ..Default::default()
                },
                demography: Demography::default(),
            })
            .collect(),
        people: vec![],
        events: vec![],
        shipments: vec![],
        candidates: vec![],
        initial_food: 0.,
        initial_population: 0.,
        economy_catalog: Some(EconomyCatalog::bundled().unwrap()),
        nutrition_initial: [0.; 3],
        cargo: vec![],
        export_contracts: vec![],
        politics: None,
        governance: None,
        shipping: None,
        expeditions: None,
        living: None,
        society: Some(Society {
            council_funding: Default::default(),
            household_economy: None,
            indicators: None,
            version: 1,
            started: 0,
            households: vec![],
            councils: vec![],
            routes: vec![
                Route {
                    id: 0,
                    from: 0,
                    to: 1,
                    cells: vec![0, 1],
                    cost_km: 100.,
                    open: true,
                    flood_months: 0,
                    road_bricks: 0.,
                    upkeep: None,
                },
                Route {
                    id: 1,
                    from: 1,
                    to: 2,
                    cells: vec![1, 2],
                    cost_km: 200.,
                    open: true,
                    flood_months: 0,
                    road_bricks: 0.,
                    upkeep: None,
                },
            ],
            routed_sites: 3,
            raids: vec![],
            next_raid: 0,
            relocation: Default::default(),
        }),
    }
}
#[test]
fn commercial_network_connects_intermediate_towns_but_military_routes_stay_direct() {
    let mut h = network();
    assert_eq!(h.route_cost(0, 2), None);
    let d = h.trade_distances().unwrap();
    assert_eq!(d[2], 300.);
    assert_eq!(d[6], 300.);
    h.society.as_mut().unwrap().routes[0].road_bricks = 1000.;
    assert_eq!(h.trade_distances().unwrap()[2], 250.);
    h.society.as_mut().unwrap().routes[1].open = false;
    assert!(h.trade_distances().unwrap()[2].is_infinite());
    h.society.as_mut().unwrap().routes[1].open = true;
    h.sites[1].economy.policy[3] = 0.;
    assert!(h.trade_distances().unwrap()[2].is_infinite());
    h.sites[1].economy.policy[3] = 1.;
    h.sites[1].abandoned = true;
    assert!(h.trade_distances().unwrap()[2].is_infinite());
}
#[test]
fn old_catalogs_preserve_old_rules_and_invalid_settings_are_rejected() {
    let c = EconomyCatalog::bundled().unwrap();
    let mut json = serde_json::to_value(&c).unwrap();
    json.as_object_mut().unwrap().remove("market");
    json.as_object_mut().unwrap().remove("weather");
    let old: EconomyCatalog = serde_json::from_value(json).unwrap();
    old.validate().unwrap();
    assert!(!old.market.network_trade);
    assert_eq!(old.weather.drought_probability, 0.);
    assert_eq!(
        old.market.reserve_per_person,
        [3.; ancient_world::economy::GOODS]
    );
    let mut bad = c.clone();
    bad.market.max_distance_km = f32::NAN;
    assert!(bad.validate().is_err());
    let mut bad = c.clone();
    bad.weather.regime_months = 0;
    assert!(bad.validate().is_err());
    let mut bad = c;
    bad.weather.drought_severity = 1.01;
    assert!(bad.validate().is_err());
}
#[test]
#[ignore = "requires hardware GPU"]
fn drought_reduces_gpu_growth_and_rain_and_checkpoint_continuation_matches() {
    use ancient_world::{
        catalog::Catalog,
        config::Config,
        gpu::{ContextGpu, Generator},
    };
    let mut g = Generator::new(
        pollster::block_on(ContextGpu::headless()).unwrap(),
        Config {
            resolution: 64,
            ecology_resolution: 16,
            ..Default::default()
        },
        Catalog::bundled().unwrap(),
    )
    .unwrap();
    g.found_civilizations(5).unwrap();
    g.enable_society().unwrap();
    let path = std::env::temp_dir().join(format!("history-weather-{}.world", std::process::id()));
    g.save(&path).unwrap();
    let mut control = Generator::load(g.gpu.clone(), &path).unwrap();
    let mut c = g
        .civilizations
        .as_ref()
        .unwrap()
        .economy_catalog
        .clone()
        .unwrap();
    c.weather.drought_probability = 0.;
    control.configure_economy(c.clone()).unwrap();
    c.weather.drought_probability = 1.;
    c.weather.drought_severity = 1.;
    g.configure_economy(c).unwrap();
    g.advance_history(12).unwrap();
    control.advance_history(12).unwrap();
    let total = |g: &Generator| -> (f32, f32) {
        let h = g.civilizations.as_ref().unwrap();
        (
            // Stored seed and fish may still be processed during drought; measure new crop growth.
            h.sites.iter().map(|s| s.economy.agriculture[0]).sum(),
            h.sites.iter().map(|s| s.economy.water[2]).sum(),
        )
    };
    let dry = total(&g);
    let wet = total(&control);
    assert_eq!(dry, (0., 0.));
    assert!(wet.0 > 0. && wet.1 > 0.);
    g.save(&path).unwrap();
    let mut resumed = Generator::load(g.gpu.clone(), &path).unwrap();
    std::fs::remove_file(path).unwrap();
    g.advance_history(24).unwrap();
    for _ in 0..24 {
        resumed.advance_history(1).unwrap();
    }
    assert_eq!(
        serde_json::to_value(&g.civilizations).unwrap(),
        serde_json::to_value(&resumed.civilizations).unwrap()
    );
    let h = g.civilizations.as_ref().unwrap();
    assert!(h.food_residual().abs() < 0.001 && h.population_residual().abs() < 0.001);
    assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
    assert!(h.sites.iter().any(|s| s.stocks.stock[0]
        < control.civilizations.as_ref().unwrap().sites[s.id as usize]
            .stocks
            .stock[0]));
    // Near-extinction must not amplify drift between separately integrated resident totals.
    g.advance_history(240).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    assert!(h.sites.iter().all(|s| s.abandoned));
    h.validate(&g.snapshot().unwrap()).unwrap();
}

#[test]
fn hostile_intermediaries_cannot_be_used_to_evade_war_closures() {
    use ancient_world::politics::{Politics, War};
    let mut h = network();
    h.politics = Some(Politics {
        occupation_months: 0,
        version: 1,
        started: 0,
        kin: vec![],
        marriages: vec![],
        factions: vec![],
        household_factions: vec![],
        governing: vec![],
        controllers: vec![0, 1, 2],
        claims: vec![],
        wars: vec![War {
            name: String::new(),
            id: 0,
            attacker: 0,
            defender: 1,
            goal: 1,
            started: 0,
            ended: None,
            outcome: String::new(),
            cause: 0,
        }],
        birth_observed: vec![],
        birth_credit: vec![],
    });
    assert!(h.trade_distances().unwrap()[2].is_infinite());
    h.politics.as_mut().unwrap().wars[0].ended = Some(0);
    assert_eq!(h.trade_distances().unwrap()[2], 300.);
}

#[test]
#[ignore = "requires hardware GPU"]
fn multi_hop_cargo_is_reserved_once_and_existing_contract_survives_closure() {
    use ancient_world::{
        catalog::Catalog,
        config::Config,
        gpu::{ContextGpu, Generator},
    };
    let mut g = Generator::new(
        pollster::block_on(ContextGpu::headless()).unwrap(),
        Config {
            resolution: 64,
            ecology_resolution: 16,
            ..Default::default()
        },
        Catalog::bundled().unwrap(),
    )
    .unwrap();
    g.found_civilizations(16).unwrap();
    g.enable_society().unwrap();
    let h = g.civilizations.as_ref().unwrap();
    let routes = &h.society.as_ref().unwrap().routes;
    let (a, b, c, first, second) = routes
        .iter()
        .find_map(|r| {
            routes.iter().find_map(|s| {
                if r.id == s.id || r.cost_km + s.cost_km >= 2900. {
                    return None;
                }
                let b = r.to;
                let c = if s.from == b {
                    s.to
                } else if s.to == b {
                    s.from
                } else {
                    return None;
                };
                (c != r.from).then_some((r.from, b, c, r.clone(), s.clone()))
            })
        })
        .expect("connected surveyed fixture");
    let h = g.civilizations.as_mut().unwrap();
    let mut pair = vec![first, second];
    for (i, r) in pair.iter_mut().enumerate() {
        r.id = i as u32;
    }
    h.society.as_mut().unwrap().routes = pair;
    for s in &mut h.sites {
        s.economy.policy[3] = if [a, b, c].contains(&s.id) { 1. } else { 0. };
    }
    let tools = h.sites[c as usize].economy.goods[3];
    h.sites[c as usize].economy.goods[3] = 0.;
    h.sites[a as usize].economy.goods[3] += tools;
    let mut catalog = h.economy_catalog.clone().unwrap();
    catalog.recipes.clear();
    catalog.market.adaptive_prices = true;
    catalog.weather = Default::default();
    g.configure_economy(catalog).unwrap();
    g.advance_history(3).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    assert_eq!(h.route_cost(a, c), None);
    let cargo = h
        .cargo
        .iter()
        .find(|cargo| cargo.from == a && cargo.to == c && cargo.good == 3)
        .expect("tools must traverse the intermediate market")
        .clone();
    assert!(cargo.kg > 1. && cargo.paid > 0.);
    g.civilizations
        .as_mut()
        .unwrap()
        .society
        .as_mut()
        .unwrap()
        .routes[0]
        .open = false;
    g.advance_history(cargo.arrives - 3).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    assert!(!h
        .cargo
        .iter()
        .any(|c| c.from == a && c.to == cargo.to && c.good == 3));
    assert!(h.events.iter().any(|e| e.kind == "market_arrival"
        && e.month == cargo.arrives
        && e.site == Some(c)
        && e.other == Some(a)));
    let receipt = h
        .trade_contact
        .receipts
        .iter()
        .find(|r| r.month == cargo.arrives && r.from == a && r.to == c)
        .expect("actual delivery contributes recent contact even after route closure");
    assert!(receipt.kg >= cargo.kg as f64);
    assert!(h
        .trade_contact
        .links(cargo.arrives)
        .any(|pair| pair == (a, c)));
    assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
}

#[test]
fn shipping_shares_capacity_between_lanes_and_respects_port_closures() {
    use ancient_world::{
        economy::Cargo,
        shipping::{Port, SeaLane, Shipping, TARGET},
    };
    let mut h = network();
    for s in &mut h.sites {
        s.island = s.id;
    }
    h.society.as_mut().unwrap().routes.clear();
    h.shipping = Some(Shipping {
        version: 1,
        started: 0,
        surveyed_sites: 3,
        ports: (0..3)
            .map(|site| Port {
                fleet: None,
                work: None,
                site,
                access: vec![site],
                water_cell: site + 3,
                access_km: 100.,
                assets: TARGET,
                commissioned: Some(0),
                flood_months: 0,
            })
            .collect(),
        lanes: vec![
            SeaLane {
                ports: [0, 1],
                cells: vec![],
                km: 600.,
                open: true,
                flood_months: 0,
            },
            SeaLane {
                ports: [0, 2],
                cells: vec![],
                km: 1200.,
                open: true,
                flood_months: 0,
            },
        ],
    });
    let roads = h.trade_distances().unwrap();
    assert_eq!(h.sea_quotes(&roads)[1], Some((350., 0)));
    assert_eq!(h.sea_quotes(&roads)[2], Some((500., 1)));
    h.cargo.push(Cargo {
        voyage_clock: None,
        freight_edges: vec![],
        freight_stops: vec![],
        from: 0,
        to: 1,
        good: 0,
        kg: 300.,
        paid: 300.,
        arrives: 12,
        sea_lane: Some(0),
        weather_delay_months: 0,
    });
    h.cargo.push(Cargo {
        voyage_clock: None,
        freight_edges: vec![],
        freight_stops: vec![],
        from: 0,
        to: 2,
        good: 0,
        kg: 200.,
        paid: 200.,
        arrives: 12,
        sea_lane: Some(1),
        weather_delay_months: 0,
    });
    assert_eq!(h.sea_capacity(0), 500.);
    assert_eq!(h.sea_capacity(1), 500.);
    assert_eq!(h.sea_capacity(u32::MAX), 0.);
    h.shipping.as_mut().unwrap().lanes[0].open = false;
    assert_eq!(h.sea_quotes(&roads)[1], None);
    assert!(h.sea_quotes(&roads)[2].is_some());
    h.sites[0].economy.policy[3] = 0.;
    assert!(h.sea_quotes(&roads).iter().all(Option::is_none));
    h.sites[0].economy.policy[3] = 1.;
    h.shipping.as_mut().unwrap().ports[0].assets[1] = 0.;
    assert!(h.sea_quotes(&roads).iter().all(Option::is_none));
}

#[test]
fn sea_arrival_cannot_bypass_hostile_inland_transit() {
    use ancient_world::{
        politics::{Politics, War},
        shipping::{Port, SeaLane, Shipping, TARGET},
    };
    let mut h = network();
    let mut intermediate = h.sites[1].clone();
    intermediate.id = 3;
    h.sites.push(intermediate);
    for s in &mut h.sites {
        s.island = if s.id == 0 { 0 } else { 1 };
    }
    h.society.as_mut().unwrap().routes = vec![
        Route {
            id: 0,
            from: 1,
            to: 3,
            cells: vec![],
            cost_km: 100.,
            open: true,
            flood_months: 0,
            road_bricks: 0.,
            upkeep: None,
        },
        Route {
            id: 1,
            from: 3,
            to: 2,
            cells: vec![],
            cost_km: 100.,
            open: true,
            flood_months: 0,
            road_bricks: 0.,
            upkeep: None,
        },
    ];
    h.politics = Some(Politics {
        occupation_months: 0,
        version: 1,
        started: 0,
        kin: vec![],
        marriages: vec![],
        factions: vec![],
        household_factions: vec![],
        governing: vec![],
        controllers: vec![0, 2, 2, 1],
        claims: vec![],
        birth_observed: vec![],
        birth_credit: vec![],
        wars: vec![War {
            name: String::new(),
            id: 0,
            attacker: 0,
            defender: 1,
            goal: 3,
            started: 0,
            ended: None,
            outcome: String::new(),
            cause: 0,
        }],
    });
    h.shipping = Some(Shipping {
        version: 1,
        started: 0,
        surveyed_sites: 4,
        ports: (0..2)
            .map(|site| Port {
                fleet: None,
                work: None,
                site,
                access: vec![site],
                water_cell: site + 3,
                access_km: 100.,
                assets: TARGET,
                commissioned: Some(0),
                flood_months: 0,
            })
            .collect(),
        lanes: vec![SeaLane {
            ports: [0, 1],
            cells: vec![],
            km: 600.,
            open: true,
            flood_months: 0,
        }],
    });
    let roads = h.trade_distances().unwrap();
    assert_eq!(
        roads[6], 200.,
        "local neutral carrier can traverse the region"
    );
    assert!(
        h.sea_quotes(&roads)[1].is_some(),
        "neutral harbor remains accessible"
    );
    assert!(
        h.sea_quotes(&roads)[2].is_none(),
        "cargo origin's hostile transit restriction survives transshipment"
    );
    h.politics.as_mut().unwrap().wars[0].ended = Some(0);
    assert_eq!(
        h.sea_quotes(&h.trade_distances().unwrap())[2],
        Some((550., 0))
    );
}

#[test]
fn delayed_cargo_spoilage_is_validated_and_legacy_compatible() {
    use ancient_world::economy::{EconomyCatalog, FOOD};
    let c = EconomyCatalog::bundled().unwrap();
    let fish = c.index("fish").unwrap();
    let preserved = c.index("preserved_food").unwrap();
    assert_eq!(c.delay_spoilage(FOOD), 0.2);
    assert_eq!(c.delay_spoilage(fish), 0.35);
    assert_eq!(c.delay_spoilage(preserved), 0.01);
    assert_eq!(c.delay_spoilage(c.index("tools").unwrap()), 0.);
    let mut old = serde_json::to_value(&c).unwrap();
    for good in old["goods"].as_array_mut().unwrap() {
        good.as_object_mut().unwrap().remove("delay_spoilage");
    }
    let old: EconomyCatalog = serde_json::from_value(old).unwrap();
    old.validate().unwrap();
    assert_eq!(old.delay_spoilage(fish), 0.);
    assert_eq!(old.delay_spoilage(FOOD), 0.2);
    for bad in [-0.01, 1.01, f32::NAN, f32::INFINITY] {
        let mut invalid = c.clone();
        invalid.goods[fish].delay_spoilage = Some(bad);
        assert!(invalid.validate().is_err());
    }
    for rate in [0., 1.] {
        let mut boundary = c.clone();
        boundary.goods[fish].delay_spoilage = Some(rate);
        boundary.validate().unwrap();
        assert_eq!(boundary.delay_spoilage(fish), rate);
    }
}
