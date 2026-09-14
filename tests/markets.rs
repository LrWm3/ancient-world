use ancient_world::{
    civilization::{Civilization, History, Site},
    economy::{Economy, EconomyCatalog},
    society::{Demography, Route, Society},
};
fn network() -> History {
    History {
        demographic_audit: None,
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
            stock_recovery: false,
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
        staged_harbors: false,
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
        recovery: false,
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
        recovery: false,
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
        staged_harbors: false,
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
    assert!(h.money_residual().abs() < 1e-12);
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
        assert!(world.money_residual().abs() < 1e-12);
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
            work_funding: None,
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
    assert!(h.money_residual().abs() < 1e-12);
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
        assert!(world.money_residual().abs() < 1e-12);
        for loan in &world.credit.loans {
            let events: Vec<_> = world
                .events
                .iter()
                .filter(|e| e.subjects.contains(&("loan".into(), loan.id as u32)))
                .collect();
            assert_eq!(
                events.iter().map(|e| e.kind.as_str()).collect::<Vec<_>>(),
                vec!["loan_issued", "loan_arrears", "loan_defaulted"]
            );
            assert!(events[0].causes.is_empty());
            assert_eq!(events[1].causes, vec![events[0].id]);
            assert_eq!(events[2].causes, vec![events[1].id]);
            assert_eq!(events[2].month, 13);
            assert_eq!(events[2].site, Some(1));
            assert_eq!(world.credit.last_events[&loan.id], events[2].id);
        }
        let last = world.credit.last_events[&0];
        let mut corrupt = world.clone();
        corrupt
            .credit
            .last_events
            .insert(0, world.credit.last_events[&1]);
        assert!(
            corrupt.validate_credit().is_err(),
            "cannot link another loan's event"
        );
        let mut corrupt = world.clone();
        corrupt.events[last as usize].causes = vec![last];
        assert!(
            corrupt.validate_credit().is_err(),
            "cannot cycle a credit cause"
        );
        let mut corrupt = world.clone();
        let previous = corrupt.events[last as usize].causes[0];
        corrupt.events[previous as usize].month = world.month + 1;
        assert!(
            corrupt.validate_credit().is_err(),
            "cannot backdate a child before its cause"
        );
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
        credit::{councils::ReviewOutcome, taxes::Observation},
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
        if intervention == 1 {
            assert!(control.credit.council_reviews.is_empty());
            assert!(control.credit.council_review_counts.is_empty());
        } else {
            let review = &control.credit.council_reviews[1];
            assert_eq!(review.cash_gap, 20.);
            assert_eq!(review.monthly_costs_annualized, 240.);
            assert_eq!(
                review.outcome,
                match intervention {
                    0 => ReviewOutcome::MissingTaxEvidence,
                    2 => ReviewOutcome::NoContactedLender,
                    _ => ReviewOutcome::Submitted,
                }
            );
            // A submitted request can still be rejected by underwriting.
            if intervention == 3 {
                assert_eq!(review.expected_taxes, Some(0.));
            }
            if intervention == 4 {
                assert_eq!(review.annual_commitments, Some(2000.));
            }
            control.validate_credit().unwrap();
        }
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
    assert_eq!(h.credit.council_review_counts[&ReviewOutcome::Submitted], 1);
    assert_eq!(h.credit.council_review_counts[&ReviewOutcome::NoCashGap], 2);
    assert_eq!(h.credit.council_reviews[1].contacted_lenders, 2);
    assert_eq!(h.credit.council_reviews[1].expected_taxes, Some(2000.));
    h.validate_credit().unwrap();
    let mut invalid = h.clone();
    invalid.credit.council_reviews[1].monthly_costs_annualized = -1.;
    assert!(invalid.validate_credit().is_err());
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
            work_funding: None,
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
        assert!(world.money_residual().abs() < 1e-12);
    }
    assert_eq!(
        serde_json::to_value(&h).unwrap(),
        serde_json::to_value(resumed).unwrap()
    );
    h.credit.restructurings[0].proposal.borrower_consent = None;
    assert!(h.validate_credit().is_err());
}

#[test]
fn precision_residue_settles_without_cash_or_default_but_insolvency_does_not() {
    use ancient_world::credit::{
        Account, EntryKind, RepaymentSource, Status, Terms, SHARED_CURRENCY,
    };
    let mut h = network();
    for s in &mut h.sites {
        s.economy.finance = [0.; 4];
    }
    h.sites[0].economy.finance = [2000., 2000., 0., 0.];
    h.sites[1].economy.finance = [200., 200., 0., 0.];
    h.commit_credit_loan(
        Terms {
            lender: Account::Town(0),
            borrower: Account::Town(1),
            currency: SHARED_CURRENCY,
            source: RepaymentSource::Export {
                contract: 0,
                payment_month: 2,
            },
            annual_simple_rate: 0.2736842105263158,
            maturity_month: 3,
            grace_months: 3,
        },
        2.3724365234375,
    )
    .unwrap()
    .unwrap();
    h.month = 3;
    let due = {
        let l = &mut h.credit.loans[0];
        l.accrue_to(3).unwrap();
        l.total_due()
    };
    h.pay_credit_loan(0, due).unwrap();
    assert!(h.credit.loans[0].total_due() > 0.);
    let mut unpaid: History = serde_json::from_value(serde_json::to_value(&h).unwrap()).unwrap();
    let cash = unpaid.sites[1].economy.finance[0];
    unpaid
        .transfer_credit_cash(
            Account::Town(1),
            Account::Town(2),
            SHARED_CURRENCY,
            f64::from(cash),
            0.,
        )
        .unwrap();
    assert_eq!(unpaid.sites[1].economy.finance[0], 0.);
    let mut protected: History = serde_json::from_value(serde_json::to_value(&h).unwrap()).unwrap();
    protected.credit.servicing_policy.available_cash_share = 0.;
    let opening_cash: Vec<_> = h.sites.iter().map(|s| s.economy.finance[0]).collect();
    let mut resumed: History = serde_json::from_value(serde_json::to_value(&h).unwrap()).unwrap();
    for world in [&mut h, &mut resumed] {
        world.service_credit_month().unwrap();
        assert_eq!(world.credit.loans[0].status, Status::PrecisionSettled);
        let receipt = world.credit.service_receipts.last().unwrap();
        assert!(!receipt.defaulted && receipt.paid == 0. && receipt.precision_settled > 0.);
        assert_eq!(
            opening_cash,
            world
                .sites
                .iter()
                .map(|s| s.economy.finance[0])
                .collect::<Vec<_>>()
        );
        assert!(world.credit.loans[0]
            .entries
            .iter()
            .any(|e| matches!(e.kind, EntryKind::PrecisionWriteOff)));
        world.month = 6;
        world.service_credit_month().unwrap();
        world.validate_credit().unwrap();
        assert!(world.money_residual().abs() < 1e-12);
    }
    assert_eq!(
        serde_json::to_value(&h).unwrap(),
        serde_json::to_value(resumed).unwrap()
    );
    let mut corrupt: History = serde_json::from_value(serde_json::to_value(&h).unwrap()).unwrap();
    corrupt.credit.service_receipts[0].precision_settled = 0.;
    assert!(corrupt.validate_credit().is_err());
    for world in [&mut unpaid, &mut protected] {
        for month in 3..=6 {
            world.month = month;
            world.service_credit_month().unwrap();
        }
        assert_eq!(world.credit.loans[0].status, Status::Defaulted);
        assert!(world
            .credit
            .service_receipts
            .iter()
            .all(|r| r.precision_settled == 0.));
        world.validate_credit().unwrap();
    }
}

#[test]
fn affordable_claim_above_forgiveness_cap_retries_instead_of_defaulting() {
    use ancient_world::credit::{Account, RepaymentSource, Status, Terms, SHARED_CURRENCY};
    let mut h = network();
    for s in &mut h.sites {
        s.economy.finance = [0.; 4];
    }
    h.sites[0].economy.finance = [2000., 2000., 0., 0.];
    h.sites[1].economy.finance = [200., 200., 0., 0.];
    h.commit_credit_loan(
        Terms {
            lender: Account::Town(0),
            borrower: Account::Town(1),
            currency: SHARED_CURRENCY,
            source: RepaymentSource::Export {
                contract: 0,
                payment_month: 2,
            },
            annual_simple_rate: 0.2736842105263158,
            maturity_month: 3,
            grace_months: 3,
        },
        0.0479736328125,
    )
    .unwrap()
    .unwrap();
    h.month = 3;
    h.service_credit_month().unwrap();
    let mut resumed: History = serde_json::from_value(serde_json::to_value(&h).unwrap()).unwrap();
    for world in [&mut h, &mut resumed] {
        for month in 4..=6 {
            world.month = month;
            world.service_credit_month().unwrap();
        }
        let receipt = world.credit.service_receipts.last().unwrap();
        assert!(receipt.precision_blocked && !receipt.defaulted);
        assert_eq!(receipt.paid, 0.);
        assert_eq!(receipt.precision_settled, 0.);
        assert_eq!(world.credit.loans[0].status, Status::Arrears);
        assert!(world.credit.loans[0].total_due() > 0.);
        world.validate_credit().unwrap();
        assert!(world.money_residual().abs() < 1e-12);
    }
    assert_eq!(
        serde_json::to_value(&h).unwrap(),
        serde_json::to_value(resumed).unwrap()
    );
    let mut corrupt: History = serde_json::from_value(serde_json::to_value(&h).unwrap()).unwrap();
    corrupt
        .credit
        .service_receipts
        .last_mut()
        .unwrap()
        .defaulted = true;
    assert!(corrupt.validate_credit().is_err());
    let mut unpaid: History = serde_json::from_value(serde_json::to_value(&h).unwrap()).unwrap();
    let cash = unpaid.sites[1].economy.finance[0];
    unpaid
        .transfer_credit_cash(
            Account::Town(1),
            Account::Town(2),
            SHARED_CURRENCY,
            cash as f64,
            0.,
        )
        .unwrap();
    unpaid.month = 7;
    unpaid.service_credit_month().unwrap();
    assert_eq!(unpaid.credit.loans[0].status, Status::Defaulted);
    assert!(
        !unpaid
            .credit
            .service_receipts
            .last()
            .unwrap()
            .precision_blocked
    );
    unpaid.validate_credit().unwrap();
    // Changing actual account balances can permit a previously blocked collection.
    let lender_cash = h.sites[0].economy.finance[0];
    h.transfer_credit_cash(
        Account::Town(0),
        Account::Town(2),
        SHARED_CURRENCY,
        lender_cash as f64,
        0.,
    )
    .unwrap();
    h.month = 7;
    h.service_credit_month().unwrap();
    assert!(h.credit.service_receipts.last().unwrap().paid > 0.);
    h.validate_credit().unwrap();
    assert!(h.money_residual().abs() < 1e-12);
}

#[test]
fn abandoned_town_treasuries_settle_existing_debt_but_cannot_originate() {
    use ancient_world::credit::{Account, RepaymentSource, Status, Terms, SHARED_CURRENCY};
    for abandoned in [0, 1] {
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
                payment_month: 11,
            },
            annual_simple_rate: 0.,
            maturity_month: 12,
            grace_months: 1,
        };
        h.commit_credit_loan(terms.clone(), 40.).unwrap().unwrap();
        h.sites[abandoned].abandoned = true;
        h.credit.servicing_policy.available_cash_share = 1.;
        let opening = serde_json::to_value(&h).unwrap();
        // Origination remains ineligible on either side, before money moves.
        assert!(h.commit_credit_loan(terms, 1.).is_err());
        assert_eq!(opening, serde_json::to_value(&h).unwrap());
        let mut resumed: History = serde_json::from_value(opening).unwrap();
        for world in [&mut h, &mut resumed] {
            world.month = 12;
            world.service_credit_month().unwrap();
            assert_eq!(world.credit.loans[0].status, Status::Repaid);
            assert_eq!(world.sites[0].economy.finance[0], 100.);
            assert_eq!(world.sites[1].economy.finance[0], 0.);
            assert_eq!(world.credit.service_receipts[0].paid, 40.);
            assert!(world.credit.service_receipts[0].accounts_available);
            assert!(world.sites[abandoned].abandoned);
            assert!(world.money_residual().abs() < 1e-12);
            world.validate_credit().unwrap();
            let once = serde_json::to_value(&world).unwrap();
            // A retained abandoned treasury is valid; a missing/rebound identity
            // or invalid cash is not, even when the loan is fully repaid.
            for party in [0, 1] {
                let mut missing: History = serde_json::from_value(once.clone()).unwrap();
                missing.sites[party].id = 99;
                assert!(missing.validate_credit().is_err());
                for cash in [-1., f32::NAN, f32::INFINITY] {
                    let mut invalid: History = serde_json::from_value(once.clone()).unwrap();
                    invalid.sites[party].economy.finance[0] = cash;
                    assert!(invalid.validate_credit().is_err());
                }
            }
            world.service_credit_month().unwrap();
            assert_eq!(once, serde_json::to_value(&world).unwrap());
        }
        assert_eq!(
            serde_json::to_value(h).unwrap(),
            serde_json::to_value(resumed).unwrap()
        );
    }
}

#[test]
fn default_recovery_preserves_loss_transfers_real_cash_and_cannot_replay() {
    use ancient_world::credit::recovery::{Reason, Request};
    use ancient_world::credit::{Account, RepaymentSource, Status, Terms, SHARED_CURRENCY};
    let mut h = network();
    for s in &mut h.sites {
        s.economy.finance = [0.; 4];
    }
    h.sites[0].economy.finance = [100., 100., 0., 0.];
    h.sites[2].economy.finance = [10., 10., 0., 0.];
    let loan = h
        .commit_credit_loan(
            Terms {
                lender: Account::Town(0),
                borrower: Account::Town(1),
                currency: SHARED_CURRENCY,
                source: RepaymentSource::Export {
                    contract: 0,
                    payment_month: 11,
                },
                annual_simple_rate: 0.12,
                maturity_month: 12,
                grace_months: 0,
            },
            25.,
        )
        .unwrap()
        .unwrap();
    let request = |id, month, allowance| Request {
        id,
        month,
        loan,
        allowance,
        reason: Reason::DelayedProceeds,
    };
    let unchanged = serde_json::to_value(&h).unwrap();
    assert!(h.recover_defaulted_credit(request(0, 0, 5.)).is_err());
    assert_eq!(unchanged, serde_json::to_value(&h).unwrap());
    h.month = 12;
    h.credit.loans[0].accrue_to(12).unwrap();
    h.credit.loans[0].write_off(12).unwrap();
    let original = serde_json::to_value(&h.credit.loans[0]).unwrap();
    // Retained abandoned accounts can settle without becoming operating towns.
    h.sites[0].abandoned = true;
    h.sites[1].abandoned = true;
    assert_eq!(h.recover_defaulted_credit(request(0, 12, 5.)).unwrap(), 5.);
    assert_eq!(h.credit.recoveries[0].transfer.interest, 3.);
    assert_eq!(h.credit.recoveries[0].transfer.principal, 2.);
    h.validate_credit().unwrap();
    assert!(h.money_residual().abs() < 1e-12);
    let before = serde_json::to_value(&h).unwrap();
    assert!(h.recover_defaulted_credit(request(0, 12, 5.)).is_err());
    assert_eq!(before, serde_json::to_value(&h).unwrap());
    let mut resumed: ancient_world::civilization::History = serde_json::from_value(before).unwrap();
    for world in [&mut h, &mut resumed] {
        world.month = 13;
        // Only twenty cash remains although the unrecovered loss is twenty-three.
        assert_eq!(
            world
                .recover_defaulted_credit(request(1, 13, 100.))
                .unwrap(),
            20.
        );
        assert_eq!(
            world
                .recover_defaulted_credit(request(2, 13, 100.))
                .unwrap(),
            0.
        );
        world.month = 14;
        // A subsequent transfer from another existing treasury funds the remainder.
        assert_eq!(
            world
                .transfer_credit_cash(Account::Town(2), Account::Town(1), SHARED_CURRENCY, 3., 0.)
                .unwrap()
                .amount(),
            3.
        );
        assert_eq!(
            world
                .recover_defaulted_credit(request(3, 14, 100.))
                .unwrap(),
            3.
        );
        assert_eq!(
            world
                .recover_defaulted_credit(request(4, 14, 100.))
                .unwrap(),
            0.
        );
        assert_eq!(
            serde_json::to_value(&world.credit.loans[0]).unwrap(),
            original
        );
        assert_eq!(world.credit.loans[0].status, Status::Defaulted);
        assert_eq!(world.credit.loans[0].total_due(), 0.);
        world.validate_credit().unwrap();
        assert!(world.money_residual().abs() < 1e-12);
    }
    assert_eq!(
        serde_json::to_value(&h).unwrap(),
        serde_json::to_value(&resumed).unwrap()
    );
    let mut corrupt = h.clone();
    corrupt.credit.recoveries[1].transfer.principal += 1.;
    assert!(corrupt.validate_credit().is_err());
}

#[test]
fn assigned_credit_routes_payments_and_recovery_by_month_without_rewriting_origin() {
    use ancient_world::credit::{
        ownership, recovery, Account, RepaymentSource, Status, Terms, SHARED_CURRENCY,
    };
    let mut h = network();
    for s in &mut h.sites {
        s.economy.finance = [0.; 4];
    }
    h.sites[0].economy.finance = [200., 200., 0., 0.];
    let terms = Terms {
        lender: Account::Town(0),
        borrower: Account::Town(1),
        currency: SHARED_CURRENCY,
        source: RepaymentSource::Export {
            contract: 0,
            payment_month: 2,
        },
        annual_simple_rate: 0.12,
        maturity_month: 3,
        grace_months: 3,
    };
    h.commit_credit_loan(terms.clone(), 100.).unwrap();
    h.month = 1;
    h.pay_credit_loan(0, 10.).unwrap();
    h.month = 2;
    let request = ownership::Request {
        id: 1,
        loan: 0,
        month: 2,
        from: Account::Town(0),
        to: Account::Town(2),
        owner_consent: Some(Account::Town(0)),
        recipient_consent: Some(Account::Town(2)),
        cause: Some(h.events[0].id),
    };
    let cash: Vec<_> = h.sites.iter().map(|s| s.economy.finance[0]).collect();
    let mut invalid: History = serde_json::from_value(serde_json::to_value(&h).unwrap()).unwrap();
    invalid.sites[2].economy.finance[0] = -1.;
    let invalid_before = serde_json::to_value(&invalid).unwrap();
    assert!(invalid.assign_credit_claim(request.clone()).is_err());
    assert_eq!(invalid_before, serde_json::to_value(&invalid).unwrap());
    h.assign_credit_claim(request.clone()).unwrap();
    assert_eq!(
        cash,
        h.sites
            .iter()
            .map(|s| s.economy.finance[0])
            .collect::<Vec<_>>()
    );
    let once = serde_json::to_value(&h).unwrap();
    assert!(h.assign_credit_claim(request).is_err());
    assert_eq!(once, serde_json::to_value(&h).unwrap());
    h.pay_credit_loan(0, 5.).unwrap();
    assert_eq!(
        h.credit.cash_receipts.last().unwrap().transfer.to,
        Account::Town(0)
    );
    h.validate_credit().unwrap();
    let checkpoint = serde_json::to_value(&h).unwrap();
    let mut resumed: History = serde_json::from_value(checkpoint.clone()).unwrap();
    for world in [&mut h, &mut resumed] {
        world.month = 3;
        let old_cash = world.sites[0].economy.finance[0];
        let new_cash = world.sites[2].economy.finance[0];
        let paid = world.pay_credit_loan(0, 12.).unwrap();
        assert!(paid > 0.);
        assert_eq!(world.sites[0].economy.finance[0], old_cash);
        assert_eq!(
            f64::from(world.sites[2].economy.finance[0] - new_cash),
            paid
        );
        assert_eq!(
            world.credit.cash_receipts.last().unwrap().transfer.to,
            Account::Town(2)
        );
        assert_eq!(
            serde_json::to_value(&world.credit.loans[0].terms).unwrap(),
            serde_json::to_value(&terms).unwrap()
        );
        world.validate_credit().unwrap();
        assert!(world.money_residual().abs() < 1e-12);
    }
    assert_eq!(
        serde_json::to_value(&h).unwrap(),
        serde_json::to_value(resumed).unwrap()
    );
    // Consent follows the effective creditor; the original lender cannot extend
    // someone else's claim. Each comparison starts from the same pending archive.
    for (consent, expected) in [
        (
            Account::Town(0),
            ancient_world::credit::restructuring::Decision::NoConsent,
        ),
        (
            Account::Town(2),
            ancient_world::credit::restructuring::Decision::Accepted,
        ),
    ] {
        let mut world: History = serde_json::from_value(checkpoint.clone()).unwrap();
        world.month = 3;
        world.credit.servicing_policy.available_cash_share = 0.;
        world.service_credit_month().unwrap();
        let decision = world
            .resolve_credit_restructuring(ancient_world::credit::restructuring::Proposal {
                month: 3,
                loan: 0,
                revised_maturity: 6,
                expected_payment_month: 5,
                lender_consent: Some(consent),
                borrower_consent: Some(Account::Town(1)),
                evidence: ancient_world::credit::underwriting::Evidence {
                    work_funding: None,
                    source: terms.source,
                    beneficiary: terms.borrower,
                    observed_month: 3,
                    expected_receipts: 1000.,
                    operating_costs: 0.,
                    expected_loss_fraction: 0.,
                },
            })
            .unwrap();
        assert_eq!(decision, expected);
        assert_eq!(
            world.credit.restructurings[0].creditor,
            Some(Account::Town(2))
        );
        assert_eq!(
            world.credit.restructurings[0].opening.terms.lender,
            Account::Town(0)
        );
        world.validate_credit().unwrap();
        let restored: History =
            serde_json::from_value(serde_json::to_value(&world).unwrap()).unwrap();
        restored.validate_credit().unwrap();
    }
    // The new owner has cash, but its inherited outstanding principal consumes
    // the lender exposure ceiling. Raising only that ceiling permits this request.
    for (limit, expected) in [(1., 0.), (1000., 1.)] {
        use ancient_world::credit::underwriting::{Evidence, Offer, Policy, Request};
        let mut world: History = serde_json::from_value(serde_json::to_value(&h).unwrap()).unwrap();
        let source = RepaymentSource::Export {
            contract: 99,
            payment_month: 5,
        };
        let round = world
            .fund_credit_requests(
                Policy {
                    max_lender_principal: limit,
                    ..Default::default()
                },
                vec![Offer {
                    lender: Account::Town(2),
                    month: 3,
                    cash: 10.,
                    operating_reserve: 0.,
                    offered_principal: 1.,
                    minimum_annual_rate: 0.,
                }],
                vec![Evidence {
                    work_funding: None,
                    source,
                    beneficiary: Account::Town(0),
                    observed_month: 3,
                    expected_receipts: 100.,
                    operating_costs: 0.,
                    expected_loss_fraction: 0.,
                }],
                vec![Request {
                    id: 90,
                    month: 3,
                    principal: 1.,
                    terms: Terms {
                        lender: Account::Town(2),
                        borrower: Account::Town(0),
                        source,
                        currency: SHARED_CURRENCY,
                        annual_simple_rate: 0.,
                        maturity_month: 6,
                        grace_months: 1,
                    },
                }],
            )
            .unwrap();
        assert_eq!(world.credit.rounds[round].grants[0].granted, expected);
        world.validate_credit().unwrap();
    }
    let mut defaulted: History = serde_json::from_value(checkpoint).unwrap();
    defaulted.month = 7;
    defaulted.credit.servicing_policy.available_cash_share = 0.;
    defaulted.service_credit_month().unwrap();
    assert_eq!(defaulted.credit.loans[0].status, Status::Defaulted);
    let original_loss = serde_json::to_value(&defaulted.credit.loans[0]).unwrap();
    let old_cash = defaulted.sites[0].economy.finance[0];
    let paid = defaulted
        .recover_defaulted_credit(recovery::Request {
            id: 50,
            month: 7,
            loan: 0,
            allowance: 10.,
            reason: recovery::Reason::VoluntarySettlement,
        })
        .unwrap();
    assert!(paid > 0.);
    assert_eq!(defaulted.sites[0].economy.finance[0], old_cash);
    assert_eq!(defaulted.credit.recoveries[0].transfer.to, Account::Town(2));
    assert_eq!(
        original_loss,
        serde_json::to_value(&defaulted.credit.loans[0]).unwrap()
    );
    defaulted.validate_credit().unwrap();
    assert!(defaulted.money_residual().abs() < 1e-12);
    let mut corrupted: History = serde_json::from_value(serde_json::to_value(&h).unwrap()).unwrap();
    corrupted
        .credit
        .cash_receipts
        .last_mut()
        .unwrap()
        .transfer
        .to = Account::Town(0);
    assert!(corrupted.validate_credit().is_err());
}

#[test]
fn household_claim_receipts_reconcile_without_enabling_household_credit() {
    use ancient_world::credit::{ownership, Account, RepaymentSource, Terms, SHARED_CURRENCY};
    use ancient_world::household_economy::{HouseholdAccount, HouseholdEconomy};
    use ancient_world::society::Household;
    let mut h = network();
    for site in &mut h.sites {
        site.economy.finance = [0.; 4];
    }
    h.sites[0].economy.finance = [100., 100., 0., 0.];
    let society = h.society.as_mut().unwrap();
    society.households.push(Household {
        id: 0,
        site: 2,
        name: "Beneficiary".into(),
        share: 1.,
        head: 0,
        vacant_since: None,
        founded: 0,
        parent: None,
        generation: 0,
    });
    let mut economy = HouseholdEconomy::new(0);
    economy.accounts.push(HouseholdAccount::default());
    society.household_economy = Some(economy);
    let terms = Terms {
        lender: Account::Town(0),
        borrower: Account::Town(1),
        currency: SHARED_CURRENCY,
        source: RepaymentSource::Export {
            contract: 0,
            payment_month: 2,
        },
        annual_simple_rate: 0.12,
        maturity_month: 3,
        grace_months: 3,
    };
    h.commit_credit_loan(terms.clone(), 20.).unwrap();
    let request = ownership::Request {
        id: 80,
        month: 0,
        loan: 0,
        from: Account::Town(0),
        to: Account::Household(0),
        owner_consent: Some(Account::Town(0)),
        recipient_consent: Some(Account::Household(0)),
        cause: None,
    };
    let pristine = serde_json::to_value(&h).unwrap();
    let mut lost: History = serde_json::from_value(pristine.clone()).unwrap();
    lost.society
        .as_mut()
        .unwrap()
        .relocation
        .lost_households
        .insert(0);
    let lost_before = serde_json::to_value(&lost).unwrap();
    assert!(lost.assign_credit_claim(request.clone()).is_err());
    assert_eq!(lost_before, serde_json::to_value(&lost).unwrap());
    h.assign_credit_claim(request).unwrap();
    assert_eq!(
        h.society
            .as_ref()
            .unwrap()
            .household_economy
            .as_ref()
            .unwrap()
            .accounts[0]
            .cash,
        0.
    );
    let mut resumed: History = serde_json::from_value(serde_json::to_value(&h).unwrap()).unwrap();
    for world in [&mut h, &mut resumed] {
        world.month = 1;
        let original_lender = world.sites[0].economy.finance[0];
        let paid = world.pay_credit_loan(0, 6.).unwrap();
        let transfer = &world.credit.cash_receipts.last().unwrap().transfer;
        let e = world
            .society
            .as_ref()
            .unwrap()
            .household_economy
            .as_ref()
            .unwrap();
        let a = &e.accounts[0];
        assert!(paid > 0. && transfer.interest > 0. && transfer.principal > 0.);
        assert_eq!(a.cash, paid);
        assert_eq!(a.credit_principal_received, transfer.principal);
        assert_eq!(a.credit_interest_received, transfer.interest);
        assert_eq!(a.wages + a.dividends + a.relief + a.capital_returned, 0.);
        assert_eq!(world.sites[0].economy.finance[0], original_lender);
        e.validate(world).unwrap();
        world.validate_credit().unwrap();
        assert!(world.money_residual().abs() < 1e-12);
        let before = serde_json::to_value(&world).unwrap();
        for (lender, borrower) in [
            (Account::Household(0), Account::Town(2)),
            (Account::Town(0), Account::Household(0)),
        ] {
            let mut forbidden = terms.clone();
            forbidden.lender = lender;
            forbidden.borrower = borrower;
            assert!(world.commit_credit_loan(forbidden, 1.).is_err());
            assert_eq!(before, serde_json::to_value(&world).unwrap());
        }
        assert!(world
            .transfer_credit_cash(
                Account::Household(0),
                Account::Town(0),
                SHARED_CURRENCY,
                1.,
                0.
            )
            .is_err());
        assert_eq!(before, serde_json::to_value(&world).unwrap());
    }
    assert_eq!(
        serde_json::to_value(&h).unwrap(),
        serde_json::to_value(resumed).unwrap()
    );
    h.month = 3;
    h.credit.servicing_policy.available_cash_share = 0.;
    h.service_credit_month().unwrap();
    let result = h
        .resolve_credit_restructuring(ancient_world::credit::restructuring::Proposal {
            month: 3,
            loan: 0,
            revised_maturity: 6,
            expected_payment_month: 5,
            lender_consent: Some(Account::Household(0)),
            borrower_consent: Some(Account::Town(1)),
            evidence: ancient_world::credit::underwriting::Evidence {
                work_funding: None,
                source: terms.source,
                beneficiary: terms.borrower,
                observed_month: 3,
                expected_receipts: 100.,
                operating_costs: 0.,
                expected_loss_fraction: 0.,
            },
        })
        .unwrap();
    assert_eq!(
        result,
        ancient_world::credit::restructuring::Decision::Accepted
    );
    h.validate_credit().unwrap();
    h.society
        .as_ref()
        .unwrap()
        .household_economy
        .as_ref()
        .unwrap()
        .validate(&h)
        .unwrap();
    // Old household archives initialize both receipt counters to zero.
    let mut old = serde_json::to_value(HouseholdAccount::default()).unwrap();
    old.as_object_mut()
        .unwrap()
        .remove("credit_principal_received");
    old.as_object_mut()
        .unwrap()
        .remove("credit_interest_received");
    let restored: HouseholdAccount = serde_json::from_value(old).unwrap();
    assert_eq!(
        restored.credit_principal_received + restored.credit_interest_received,
        0.
    );
}

#[test]
fn monetary_residual_detects_created_cash_without_water_changes() {
    let mut h = network();
    for site in &mut h.sites {
        site.economy.finance = [0.; 4];
    }
    h.sites[0].economy.finance = [100., 100., 0., 0.];
    let water = h.economy_residuals()[3];
    assert_eq!(h.money_residual(), 0.);
    h.sites[1].economy.finance[0] += 1.; // Deliberately unrecorded creation.
    assert_eq!(h.money_residual(), -0.01);
    assert_eq!(h.economy_residuals()[3], water);
    h.sites[1].economy.finance[0] -= 1.;
    assert_eq!(h.money_residual(), 0.);
    h.sites[0].economy.finance[0] -= 1.; // Deliberately unrecorded destruction.
    assert_eq!(h.money_residual(), 0.01);
    assert_eq!(h.economy_residuals()[3], water);
}
