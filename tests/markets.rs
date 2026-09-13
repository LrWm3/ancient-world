use ancient_world::{
    civilization::{Civilization, History, Site},
    economy::{Economy, EconomyCatalog},
    society::{Demography, Route, Society},
};
fn network() -> History {
    History {
        credit: Default::default(),
        contagion: None,
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
        export_identities: vec![],
        export_payments: vec![],
        export_payment_timing: Default::default(),
        politics: None,
        governance: None,
        shipping: None,
        expeditions: None,
        living: None,
        society: Some(Society {
            town_support_policy: Default::default(),
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
        leadership: Default::default(),
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
        export_payment: None,
        infection: None,
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
        export_payment: None,
        infection: None,
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
        leadership: Default::default(),
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

#[test]
fn credit_cash_and_default_ledgers_run_without_gpu() {
    use ancient_world::credit::{Account, RepaymentSource, Status, Terms, SHARED_CURRENCY};
    let mut h = network();
    for site in &mut h.sites {
        site.economy.finance = [0.; 4];
    }
    h.sites[0].economy.finance = [100., 100., 0., 0.];
    let terms = Terms {
        lender: Account::Town(0),
        borrower: Account::Town(1),
        currency: SHARED_CURRENCY,
        source: RepaymentSource::Export {
            contract: 0,
            payment_month: 12,
        },
        annual_simple_rate: 0.12,
        maturity_month: 12,
        grace_months: 3,
    };
    let id = h.commit_credit_loan(terms, 25.).unwrap().unwrap();
    assert_eq!(h.sites[0].economy.finance[0], 75.);
    assert_eq!(h.sites[1].economy.finance[0], 25.);
    h.validate_credit().unwrap();
    assert!(h.economy_residuals()[3].abs() < 1e-12);
    let mut resumed: History = serde_json::from_value(serde_json::to_value(&h).unwrap()).unwrap();
    for world in [&mut h, &mut resumed] {
        world.month = 12;
        let paid = world.pay_credit_loan(id, 28.).unwrap();
        assert_eq!(paid, 25.);
        assert!((world.credit.loans[0].outstanding_principal - 3.).abs() < 1e-12);
        assert_eq!(world.credit.loans[0].status, Status::Arrears);
        world.month = 15;
        world.credit.loans[0].accrue_to(15).unwrap();
        world.credit.loans[0].write_off(15).unwrap();
        world.validate_credit().unwrap();
        assert_eq!(world.credit.loans[0].status, Status::Defaulted);
        assert_eq!(world.sites[0].economy.finance[0], 100.);
        assert_eq!(world.sites[1].economy.finance[0], 0.);
        assert!(world.economy_residuals()[3].abs() < 1e-12);
    }
    assert_eq!(
        serde_json::to_value(h).unwrap(),
        serde_json::to_value(resumed).unwrap()
    );
}

#[test]
fn credit_round_shares_actual_cash_and_cannot_replay_requests() {
    use ancient_world::credit::underwriting::{Evidence, Offer, Policy, Request};
    use ancient_world::credit::{Account, RepaymentSource, Terms, SHARED_CURRENCY};
    let mut h = network();
    for site in &mut h.sites {
        site.economy.finance = [0.; 4];
    }
    h.sites[0].economy.finance = [100., 100., 0., 0.];
    h.month = 1;
    let sources = [
        RepaymentSource::Export {
            contract: 0,
            payment_month: 12,
        },
        RepaymentSource::Export {
            contract: 1,
            payment_month: 12,
        },
    ];
    let evidence: Vec<_> = (1..=2)
        .map(|id| Evidence {
            source: sources[id - 1],
            beneficiary: Account::Town(id as u32),
            observed_month: 0,
            expected_receipts: 200.,
            operating_costs: 0.,
            expected_loss_fraction: 0.,
        })
        .collect();
    let requests: Vec<_> = (1..=2)
        .map(|id| Request {
            id: id as u64,
            month: 1,
            principal: 100.,
            terms: Terms {
                lender: Account::Town(0),
                borrower: Account::Town(id as u32),
                currency: SHARED_CURRENCY,
                source: sources[id - 1],
                annual_simple_rate: 0.,
                maturity_month: 13,
                grace_months: 3,
            },
        })
        .collect();
    let offers = vec![Offer {
        lender: Account::Town(0),
        month: 1,
        cash: 1000.,
        operating_reserve: 20.,
        offered_principal: 1000.,
        minimum_annual_rate: 0.,
    }];
    let index = h
        .fund_credit_requests(
            Policy::default(),
            offers.clone(),
            evidence.clone(),
            requests.clone(),
        )
        .unwrap();
    assert_eq!(h.credit.rounds[index].offers[0].cash, 100.);
    assert_eq!(h.sites[0].economy.finance[0], 20.);
    assert_eq!(h.sites[1].economy.finance[0], 40.);
    assert_eq!(h.sites[2].economy.finance[0], 40.);
    h.validate_credit().unwrap();
    assert!(h.economy_residuals()[3].abs() < 1e-12);
    let before = serde_json::to_value(&h).unwrap();
    assert!(h
        .fund_credit_requests(Policy::default(), offers, evidence, requests)
        .is_err());
    assert_eq!(before, serde_json::to_value(h).unwrap());
}

#[test]
fn scheduled_credit_protects_cash_shares_claims_and_defaults_without_money_creation() {
    use ancient_world::credit::{Account, RepaymentSource, Status, Terms, SHARED_CURRENCY};
    let mut h = network();
    for site in &mut h.sites {
        site.economy.finance = [0.; 4];
    }
    h.sites[0].economy.finance = [100., 100., 0., 0.];
    h.sites[2].economy.finance = [100., 100., 0., 0.];
    for lender in [0, 2] {
        h.commit_credit_loan(
            Terms {
                lender: Account::Town(lender),
                borrower: Account::Town(1),
                currency: SHARED_CURRENCY,
                source: RepaymentSource::Export {
                    contract: lender as u64,
                    payment_month: 11,
                },
                annual_simple_rate: 0.,
                maturity_month: 12,
                grace_months: 1,
            },
            40.,
        )
        .unwrap();
    }
    h.credit.servicing_policy.available_cash_share = 1.;
    h.credit
        .servicing_policy
        .protected_cash
        .push((Account::Town(1), 60.));
    h.month = 11;
    h.service_credit_month().unwrap();
    assert_eq!(h.sites[1].economy.finance[0], 80.);
    let mut resumed: History = serde_json::from_value(serde_json::to_value(&h).unwrap()).unwrap();
    for world in [&mut h, &mut resumed] {
        world.month = 12;
        world.service_credit_month().unwrap();
        assert_eq!(world.sites[1].economy.finance[0], 60.);
        assert_eq!(world.credit.service_receipts.len(), 2);
        for receipt in &world.credit.service_receipts {
            assert_eq!(receipt.paid, 10.);
        }
        let once = serde_json::to_value(&world).unwrap();
        world.service_credit_month().unwrap();
        assert_eq!(once, serde_json::to_value(&world).unwrap());
        world.month = 13;
        world.service_credit_month().unwrap();
        assert!(world
            .credit
            .loans
            .iter()
            .all(|l| l.status == Status::Defaulted));
        assert_eq!(world.sites[1].economy.finance[0], 60.);
        assert!(world.economy_residuals()[3].abs() < 1e-12);
        world.validate_credit().unwrap();
    }
    assert_eq!(
        serde_json::to_value(h).unwrap(),
        serde_json::to_value(resumed).unwrap()
    );
}

#[test]
fn council_credit_requires_receipts_need_contact_and_reaches_existing_treasury() {
    use ancient_world::{
        credit::taxes::Observation,
        household_economy::{council_allocation, HouseholdEconomy},
        society::Council,
    };
    let mut h = network();
    h.month = 13;
    h.credit.council_policy.enabled = true;
    for site in &mut h.sites {
        site.economy.finance[0] = 10_000.;
    }
    let society = h.society.as_mut().unwrap();
    society.councils = (0..3)
        .map(|civilization| Council {
            civilization,
            treasury: if civilization == 1 { 0. } else { 1000. },
            tax_rate: 0.2,
            distribution: None,
            pending_distribution: None,
            distribution_review: None,
            pending_tax: None,
            tax_effective_since: None,
            relief_paid: 0.,
        })
        .collect();
    let mut households = HouseholdEconomy::new(0);
    households
        .council_allocations
        .push(council_allocation::Receipt {
            month: 12,
            council: 1,
            policy: Default::default(),
            treasury: 0.,
            administration_forecast: 0.,
            relief_requested: 20.,
            relief_ceiling: 0.,
            relief_granted: 0.,
            relief_paid: 0.,
        });
    society.household_economy = Some(households);
    h.credit.tax_observations.push(Observation {
        month: 12,
        council: 1,
        collected: 2000.,
        support_requested: 0.,
        road_requested: Some(0.),
    });
    let original = h.clone();
    for intervention in 0..5 {
        let mut control = original.clone();
        match intervention {
            0 => control.credit.tax_observations.clear(),
            1 => control.credit.council_policy.enabled = false,
            2 => control
                .society
                .as_mut()
                .unwrap()
                .routes
                .iter_mut()
                .for_each(|r| r.open = false),
            3 => control.sites[1].economy.finance[0] = 0.,
            _ => control.credit.tax_observations[0].road_requested = Some(2000.),
        }
        assert_eq!(control.council_credit_month().unwrap(), 0);
        assert_eq!(control.society.as_ref().unwrap().councils[1].treasury, 0.);
    }
    assert_eq!(h.council_credit_month().unwrap(), 2);
    let society = h.society.as_ref().unwrap();
    assert_eq!(society.councils[1].treasury, 20.);
    assert_eq!(
        society.councils.iter().map(|c| c.treasury).sum::<f64>(),
        2000.
    );
    assert_eq!(
        h.credit
            .loans
            .iter()
            .map(|l| l.original_principal)
            .sum::<f64>(),
        20.
    );
    h.validate_credit().unwrap();
    let mut resumed: History = serde_json::from_value(serde_json::to_value(&h).unwrap()).unwrap();
    assert_eq!(resumed.council_credit_month().unwrap(), 0);
    assert_eq!(
        serde_json::to_value(h).unwrap(),
        serde_json::to_value(resumed).unwrap()
    );
}

#[test]
fn shared_issuance_caps_supply_and_does_not_reset_on_toggle_or_reload() {
    use ancient_world::{credit::issuance::Limits, society::Council};
    let mut h = network();
    h.society.as_mut().unwrap().councils = (0..3)
        .map(|civilization| Council {
            civilization,
            treasury: 0.,
            tax_rate: 0.2,
            distribution: None,
            pending_distribution: None,
            distribution_review: None,
            pending_tax: None,
            tax_effective_since: None,
            relief_paid: 0.,
        })
        .collect();
    h.configure_shared_issuance(true).unwrap();
    let schedule = h.credit.issuance.schedule.as_mut().unwrap();
    schedule.amount = 80.;
    schedule.end_month = 24;
    schedule.interval_months = 1;
    schedule.limits = Limits {
        per_issue: 60.,
        annual: 100.,
        lifetime: 150.,
        cooldown_months: 2,
    };
    for month in 1..=8 {
        h.month = month;
        h.shared_issuance_month().unwrap();
        let expected = match month {
            1 | 2 => 180.,
            _ => 300.,
        };
        assert_eq!(h.credit.issuance.total_issued(), expected);
        assert!(h.economy_residuals()[4].abs() < 1e-12);
    }
    let before = serde_json::to_value(&h).unwrap();
    h.shared_issuance_month().unwrap();
    assert_eq!(before, serde_json::to_value(&h).unwrap());
    let mut resumed: History = serde_json::from_value(before).unwrap();
    for month in 9..=25 {
        for world in [&mut h, &mut resumed] {
            world.month = month;
            if month == 9 {
                world.configure_shared_issuance(false).unwrap();
            }
            if month == 12 {
                world.configure_shared_issuance(true).unwrap();
            }
            world.shared_issuance_month().unwrap();
            world.validate_credit().unwrap();
            assert!(world.economy_residuals()[4].abs() < 1e-12);
        }
    }
    assert_eq!(h.credit.issuance.total_issued(), 450.);
    assert_eq!(h.credit.issuance.schedule.as_ref().unwrap().start_month, 1);
    assert_eq!(
        serde_json::to_value(&h).unwrap(),
        serde_json::to_value(resumed).unwrap()
    );
    let money = h.credit.issuance.total_issued();
    h.configure_shared_issuance(true).unwrap();
    h.month = 26;
    h.shared_issuance_month().unwrap();
    assert_eq!(h.credit.issuance.total_issued(), money);
    let mut corrupt = h.clone();
    corrupt.credit.issuance.receipts[0].issued += 1.;
    assert!(corrupt.validate_credit().is_err());
    let mut corrupt = h.clone();
    corrupt
        .credit
        .issuance
        .receipts
        .push(corrupt.credit.issuance.receipts[0].clone());
    assert!(corrupt.validate_credit().is_err());
    let mut corrupt = h.clone();
    corrupt
        .credit
        .issuance
        .schedule
        .as_mut()
        .unwrap()
        .issuers
        .push(999);
    assert!(corrupt.validate_credit().is_err());
    // A malformed huge authorization must not commit the first issuer and then fail.
    let mut overflow = h.clone();
    overflow.credit.issuance = Default::default();
    overflow.configure_shared_issuance(true).unwrap();
    let schedule = overflow.credit.issuance.schedule.as_mut().unwrap();
    schedule.amount = f64::MAX;
    schedule.limits = Limits {
        per_issue: f64::MAX,
        annual: f64::MAX,
        lifetime: f64::MAX,
        cooldown_months: 0,
    };
    overflow.month += 1;
    let before = serde_json::to_value(&overflow).unwrap();
    assert!(overflow.shared_issuance_month().is_err());
    assert_eq!(before, serde_json::to_value(overflow).unwrap());
}

#[test]
fn consensual_credit_extension_preserves_cash_and_resumes_collection() {
    use ancient_world::credit::{
        restructuring::{Decision, Proposal},
        underwriting::Evidence,
        Account, RepaymentSource, Status, Terms, SHARED_CURRENCY,
    };
    let mut h = network();
    for site in &mut h.sites {
        site.economy.finance = [0.; 4];
    }
    h.sites[0].economy.finance = [100., 100., 0., 0.];
    let source = RepaymentSource::Export {
        contract: 0,
        payment_month: 2,
    };
    let terms = Terms {
        lender: Account::Town(0),
        borrower: Account::Town(1),
        currency: SHARED_CURRENCY,
        source,
        annual_simple_rate: 0.,
        maturity_month: 3,
        grace_months: 3,
    };
    h.commit_credit_loan(terms.clone(), 25.).unwrap();
    h.month = 3;
    h.credit.servicing_policy.available_cash_share = 0.;
    h.service_credit_month().unwrap();
    let proposal = Proposal {
        month: 3,
        loan: 0,
        revised_maturity: 6,
        expected_payment_month: 5,
        evidence: Evidence {
            source,
            beneficiary: terms.borrower,
            observed_month: 3,
            expected_receipts: 100.,
            operating_costs: 10.,
            expected_loss_fraction: 0.,
        },
        lender_consent: Some(terms.lender),
        borrower_consent: Some(terms.borrower),
    };
    let opening_cash: Vec<_> = h.sites.iter().map(|s| s.economy.finance[0]).collect();
    assert_eq!(
        h.resolve_credit_restructuring(proposal.clone()).unwrap(),
        Decision::Accepted
    );
    assert!(h.resolve_credit_restructuring(proposal).is_err());
    assert_eq!(
        opening_cash,
        h.sites
            .iter()
            .map(|s| s.economy.finance[0])
            .collect::<Vec<_>>()
    );
    assert_eq!(h.credit.cash_receipts.len(), 1);
    h.validate_credit().unwrap();
    let mut resumed: History = serde_json::from_value(serde_json::to_value(&h).unwrap()).unwrap();
    for world in [&mut h, &mut resumed] {
        world.credit.servicing_policy.available_cash_share = 1.;
        for month in 4..=6 {
            world.month = month;
            world.service_credit_month().unwrap();
        }
        assert_eq!(world.credit.loans[0].status, Status::Repaid);
        world.validate_credit().unwrap();
        assert!(world.economy_residuals()[3].abs() < 1e-12);
    }
    assert_eq!(
        serde_json::to_value(&h).unwrap(),
        serde_json::to_value(resumed).unwrap()
    );
    h.credit.restructurings[0].proposal.borrower_consent = None;
    assert!(h.validate_credit().is_err());
}
