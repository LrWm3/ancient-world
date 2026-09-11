//! Short-horizon interventions on identical histories, using production consumers.
//! Setup inventories are declared fixtures, not claims about naturally occurring worlds.
use super::*;
use crate::{
    catalog::Catalog,
    config::Config,
    culture::Institution,
    gpu::{ContextGpu, Generator},
    institution_capacity::Capacity,
};

fn fixture(seed: u32) -> History {
    let mut g = Generator::new(
        pollster::block_on(ContextGpu::headless()).unwrap(),
        Config {
            resolution: 64,
            ecology_resolution: 16,
            seed,
            ..Default::default()
        },
        Catalog::bundled().unwrap(),
    )
    .unwrap();
    g.found_civilizations(5).unwrap();
    g.enable_society().unwrap();
    g.enable_politics().unwrap();
    g.enable_governance().unwrap();
    let mut h = g.civilizations.take().unwrap();
    h.month = 24;
    // Remove unrelated material grievances and route construction from these fixtures.
    h.society.as_mut().unwrap().routes.clear();
    h.society.as_mut().unwrap().indicators = None;
    for s in &mut h.sites {
        s.stocks.stock[3] = 0.;
        s.economy.soil[3] = 0.;
        s.economy.finance[0] = 10_000.;
    }
    let members = h.culture.as_ref().unwrap().site_people(&h, 0);
    assert!(members.len() >= 2);
    let c = h.culture.as_mut().unwrap();
    c.institutions.clear();
    c.institutions.push(Institution {
        id: 0,
        name: "Causal fixture school".into(),
        kind: InstitutionKind::Scholarly,
        site: 0,
        tradition: None,
        members: members[..2].to_vec(),
        leader: members[0],
        treasury: 0.,
        active: true,
        founded: 0,
        knowledge: Default::default(),
        property: vec![],
        dues: 0.,
        expenses: 0.,
        capacity: Some(Capacity::new(0)),
    });
    c.labor_budget = vec![0.5; h.sites.len()];
    let controller = h.controller(0);
    h.society.as_mut().unwrap().councils[controller as usize].treasury = 100.;
    h
}

fn pending(h: &mut History, demand: Demand) {
    let controller = h.controller(0);
    let faction = h
        .politics
        .as_ref()
        .unwrap()
        .factions
        .iter()
        .find(|f| f.civilization == controller && f.interest == 3)
        .unwrap()
        .id;
    h.politics.as_mut().unwrap().governing[controller as usize] = faction;
    h.event(
        "civic_petition",
        Some(0),
        None,
        "Controlled petition".into(),
    );
    h.governance.as_mut().unwrap().petitions.push(Petition {
        site: 0,
        controller,
        faction,
        institution: 0,
        opened: h.month,
        cause: h.events.last().unwrap().id,
        demand,
        requested: if demand == Demand::Autonomy { 0. } else { 10. },
        pressure: 0.8,
        resolved: None,
        outcome: None,
        resolution_reason: None,
        responding_faction: None,
        honored: false,
        paid: 0.,
    });
    h.month += 3;
}

fn money(h: &History) -> f64 {
    h.sites
        .iter()
        .map(|s| s.economy.finance[0] as f64)
        .sum::<f64>()
        + h.society
            .as_ref()
            .unwrap()
            .councils
            .iter()
            .map(|c| c.treasury)
            .sum::<f64>()
        + h.culture
            .as_ref()
            .unwrap()
            .institutions
            .iter()
            .map(|n| n.treasury)
            .sum::<f64>()
        + h.society
            .as_ref()
            .unwrap()
            .household_economy
            .as_ref()
            .unwrap()
            .accounts
            .iter()
            .map(|a| a.cash)
            .sum::<f64>()
}

#[test]
#[ignore = "requires hardware GPU"]
fn learning_grant_requires_work_and_staff_to_preserve_readiness() {
    for seed in [7, 17] {
        let mut baseline = fixture(seed);
        pending(&mut baseline, Demand::Learning);
        let mut funded = baseline.clone();
        let initial = money(&funded);
        resolve(&mut funded);
        assert!(funded.governance.as_ref().unwrap().petitions[0].honored);
        assert!((money(&funded) - initial).abs() < 1e-8);
        // Same petition and response: remove just the transferred money, returning it
        // to its payer. Event text and reputation cannot finance upkeep.
        baseline = funded.clone();
        baseline.culture.as_mut().unwrap().institutions[0].treasury -= 10.;
        let payer = baseline.controller(0) as usize;
        baseline.society.as_mut().unwrap().councils[payer].treasury += 10.;
        let mut no_work = funded.clone();
        let mut no_staff = funded.clone();
        no_staff.culture.as_mut().unwrap().institutions[0]
            .members
            .clear();
        for quarter in 0..4 {
            for (h, work) in [
                (&mut baseline, 0.5),
                (&mut funded, 0.5),
                (&mut no_work, 0.),
                (&mut no_staff, 0.5),
            ] {
                h.month = 30 + quarter * 3;
                let mut c = h.culture.take().unwrap();
                c.labor_budget.fill(work);
                c.maintain_institutions(h);
                h.culture = Some(c);
                assert!((money(h) - initial).abs() < 1e-6);
            }
        }
        let ready = |h: &History| {
            h.culture.as_ref().unwrap().institutions[0]
                .capacity
                .as_ref()
                .unwrap()
                .readiness
        };
        assert!(ready(&funded) > ready(&baseline) + 0.5);
        assert!(funded.culture.as_ref().unwrap().institutions[0].operational());
        assert!(!baseline.culture.as_ref().unwrap().institutions[0].operational());
        assert_eq!(ready(&no_work), ready(&baseline));
        assert_eq!(ready(&no_staff), ready(&baseline));
        for h in [&no_work, &no_staff] {
            assert_eq!(h.culture.as_ref().unwrap().institutions[0].treasury, 10.);
        }
        assert_eq!(
            funded.culture.as_ref().unwrap().institutions[0].expenses,
            2.
        );
    }
}

#[test]
#[ignore = "requires hardware GPU"]
fn autonomy_response_reduces_real_tax_and_administrative_cost() {
    for seed in [7, 17] {
        let mut granted = fixture(seed);
        pending(&mut granted, Demand::Autonomy);
        let old = granted.governance.as_ref().unwrap().administrations[0].autonomy;
        let initial = money(&granted);
        resolve(&mut granted);
        assert!((money(&granted) - initial).abs() < 1e-8);
        let mut ablated = granted.clone();
        // Preserve the response, loyalty and all other state; remove only autonomy.
        ablated.governance.as_mut().unwrap().administrations[0].autonomy = old;
        let delta = granted.governance.as_ref().unwrap().administrations[0].autonomy - old;
        assert!(delta > 0.);
        let payer = granted.controller(0) as usize;
        let tax_rate = granted.society.as_ref().unwrap().councils[payer].tax_rate;
        let expected_tax = granted.sites[0].economy.finance[0] * tax_rate * 0.75 * delta;
        let mut fiscal_control = ablated.clone();
        let mut fiscal_grant = granted.clone();
        fiscal_control.social_year();
        fiscal_grant.social_year();
        assert!(
            (fiscal_grant.sites[0].economy.finance[0]
                - fiscal_control.sites[0].economy.finance[0]
                - expected_tax)
                .abs()
                < 0.002
        );
        assert!(
            (fiscal_control.society.as_ref().unwrap().councils[payer].treasury
                - fiscal_grant.society.as_ref().unwrap().councils[payer].treasury
                - expected_tax as f64)
                .abs()
                < 0.002
        );
        let expected_work = granted.sites[0].stocks.stock[0] * 0.01 * 0.5 * delta;
        granted.governance_month();
        ablated.governance_month();
        let wages = |h: &History| h.governance.as_ref().unwrap().administrations[0].wages_paid;
        assert!((wages(&ablated) - wages(&granted) - expected_work as f64).abs() < 1e-5);
        assert!(
            granted.governance.as_ref().unwrap().administrations[0].unrest
                <= ablated.governance.as_ref().unwrap().administrations[0].unrest
        );
    }
}

#[test]
#[ignore = "requires hardware GPU"]
fn remembered_delivery_changes_eligible_households_not_distant_or_uninformed_ones() {
    for seed in [7, 17] {
        let mut informed = fixture(seed);
        pending(&mut informed, Demand::Learning);
        resolve(&mut informed);
        // Neutral material conditions; artisans narrowly trail incumbent growers
        // without reputation. Actual politics_year owns scoring and eligibility.
        for s in &mut informed.sites {
            s.economy.labor = [0.82, 0., 0., 0.18];
            s.economy.finance[1] = 1.;
            s.economy.finance[2] = 0.;
        }
        for a in &mut informed.culture.as_mut().unwrap().agents {
            a.traits[2] = 0.;
            a.traits[3] = 0.;
        }
        for f in &mut informed.politics.as_mut().unwrap().factions {
            f.cohesion = 1.;
        }
        for hh in &informed.society.as_ref().unwrap().households {
            let civ = informed.sites[hh.site as usize].civilization;
            let grower = informed
                .politics
                .as_ref()
                .unwrap()
                .factions
                .iter()
                .find(|f| f.civilization == civ && f.interest == 0)
                .unwrap()
                .id;
            informed.politics.as_mut().unwrap().household_factions[hh.id as usize] = grower;
        }
        let prior = informed
            .politics
            .as_ref()
            .unwrap()
            .household_factions
            .clone();
        let mut ablated = informed.clone();
        ablated.governance.as_mut().unwrap().petitions.clear();
        let mut foreign = informed.clone();
        // A response attributed to another government conveys no local credit.
        foreign.governance.as_mut().unwrap().petitions[0].controller =
            (foreign.controller(0) + 1) % foreign.civilizations.len() as u32;
        informed.politics_year();
        ablated.politics_year();
        foreign.politics_year();
        let choices = |h: &History| h.politics.as_ref().unwrap().household_factions.clone();
        assert_eq!(choices(&ablated), prior);
        assert_eq!(choices(&foreign), choices(&ablated));
        let mut switched = 0;
        for hh in &informed.society.as_ref().unwrap().households {
            let id = hh.id as usize;
            let selected = informed.politics.as_ref().unwrap().household_factions[id];
            if hh.site == 0 && (hh.id + informed.month / 12) % 3 == 0 {
                assert_eq!(
                    informed.politics.as_ref().unwrap().factions[selected as usize].interest,
                    3
                );
                switched += 1;
            } else {
                assert_eq!(
                    selected, prior[id],
                    "no remote or out-of-schedule conversion"
                );
            }
        }
        assert!(
            switched > 0,
            "fixture must expose the political-credit consumer"
        );
        assert!(
            informed.politics.as_ref().unwrap().wars.is_empty(),
            "petition credit alone is not a material grievance or campaign supply"
        );
    }
}

#[test]
#[ignore = "requires hardware GPU"]
fn escalation_requires_grievance_access_claim_and_finite_supplies() {
    for seed in [7, 17] {
        let mut ready = fixture(seed);
        pending(&mut ready, Demand::Learning);
        resolve(&mut ready);
        let target = ready
            .sites
            .iter()
            .find(|s| ready.controller(s.id) != ready.controller(0))
            .unwrap()
            .id;
        // A controlled route graph, not a claim about these sampled towns' geography.
        ready
            .society
            .as_mut()
            .unwrap()
            .routes
            .push(crate::society::Route {
                id: 0,
                from: 0,
                to: target,
                cells: vec![ready.sites[0].cell, ready.sites[target as usize].cell],
                cost_km: 150.,
                open: true,
                flood_months: 0,
                road_bricks: 0.,
                upkeep: None,
            });
        ready
            .politics
            .as_mut()
            .unwrap()
            .claims
            .push(crate::politics::Claim {
                cell: ready.sites[0].cell,
                sites: vec![0, target],
            });
        ready.sites[0].stocks.stock[0] = 100.;
        ready.sites[0].demography.ages[1] = 60.;
        ready.sites[0].stocks.stock[1] = 10_000.;
        ready.sites[0].economy.goods[3] = 10.;
        ready.sites[target as usize].stocks.stock[1] = 30_000.;
        let mut no_grievance = ready.clone();
        no_grievance.politics_year();
        assert!(
            no_grievance.politics.as_ref().unwrap().wars.is_empty(),
            "an honored petition cannot substitute for a material grievance"
        );
        ready.event(
            "food_crisis",
            Some(0),
            None,
            "Controlled historical shortage".into(),
        );
        let cause = ready.events.last().unwrap().id;
        let inventory = |h: &History| {
            [
                h.sites
                    .iter()
                    .map(|s| s.stocks.stock[0] as f64)
                    .sum::<f64>()
                    + h.society
                        .as_ref()
                        .unwrap()
                        .raids
                        .iter()
                        .map(|r| r.soldiers as f64)
                        .sum::<f64>(),
                h.sites
                    .iter()
                    .map(|s| s.stocks.stock[1] as f64)
                    .sum::<f64>()
                    + h.society
                        .as_ref()
                        .unwrap()
                        .raids
                        .iter()
                        .map(|r| r.food as f64)
                        .sum::<f64>(),
                h.sites
                    .iter()
                    .map(|s| s.economy.goods[3] as f64)
                    .sum::<f64>()
                    + h.society
                        .as_ref()
                        .unwrap()
                        .raids
                        .iter()
                        .map(|r| r.equipment as f64)
                        .sum::<f64>(),
            ]
        };
        for missing in ["route", "claim", "tools", "food", "recent grievance"] {
            let mut branch = ready.clone();
            match missing {
                "route" => branch.society.as_mut().unwrap().routes[0].open = false,
                "claim" => branch.politics.as_mut().unwrap().claims.clear(),
                "tools" => branch.sites[0].economy.goods[3] = 0.,
                "food" => branch.sites[0].stocks.stock[1] = 0.,
                _ => branch.month += 25,
            }
            let before = inventory(&branch);
            branch.politics_year();
            assert!(
                branch.politics.as_ref().unwrap().wars.is_empty(),
                "{missing}"
            );
            assert_eq!(
                inventory(&branch),
                before,
                "failed muster consumes nothing: {missing}"
            );
        }
        let before = inventory(&ready);
        ready.politics_year();
        assert_eq!(ready.politics.as_ref().unwrap().wars.len(), 1);
        assert_eq!(
            inventory(&ready),
            before,
            "mobilization transfers actual stocks"
        );
        let war = &ready.politics.as_ref().unwrap().wars[0];
        assert!(ready.events[war.cause as usize].causes.contains(&cause));
        assert_eq!(ready.society.as_ref().unwrap().raids[0].soldiers, 10.);
    }
}

#[test]
#[ignore = "requires hardware GPU"]
fn faction_accountability_remembers_the_responder_after_turnover() {
    let mut h = fixture(17);
    pending(&mut h, Demand::Learning);
    let controller = h.controller(0) as usize;
    let opponent = h
        .politics
        .as_ref()
        .unwrap()
        .factions
        .iter()
        .find(|f| f.civilization == controller as u32 && f.interest == 2)
        .unwrap()
        .id;
    h.politics.as_mut().unwrap().governing[controller] = opponent;
    h.month += 9; // Valid but opposed petition expires after twelve months.
    let before = money(&h);
    resolve(&mut h);
    let p = &h.governance.as_ref().unwrap().petitions[0];
    assert!(!p.honored);
    assert_eq!(p.responding_faction, Some(opponent));
    assert_eq!(p.resolution_reason.as_deref(), Some("political opposition"));
    assert_eq!(money(&h), before);
    let advocate = p.faction;
    assert_eq!(credit(&h, h.politics.as_ref().unwrap(), 0, 3), 0.);
    assert!(credit(&h, h.politics.as_ref().unwrap(), 0, 2) < 0.);
    h.politics.as_mut().unwrap().governing[controller] = advocate;
    assert!(credit(&h, h.politics.as_ref().unwrap(), 0, 2) < 0.);
    assert_eq!(credit(&h, h.politics.as_ref().unwrap(), 0, 3), 0.);
    let resumed: History = serde_json::from_slice(&serde_json::to_vec(&h).unwrap()).unwrap();
    assert_eq!(
        credit(&h, h.politics.as_ref().unwrap(), 0, 2),
        credit(&resumed, resumed.politics.as_ref().unwrap(), 0, 2)
    );
    validate(&h, &h.governance.as_ref().unwrap().petitions).unwrap();
}

#[test]
#[ignore = "requires hardware GPU"]
fn faction_alignment_responds_to_food_access_with_identical_town_stocks() {
    let mut fed = fixture(81);
    let household_sites: Vec<_> = fed
        .society
        .as_ref()
        .unwrap()
        .households
        .iter()
        .map(|hh| hh.site)
        .collect();
    let wallets = fed
        .society
        .as_mut()
        .unwrap()
        .household_economy
        .as_mut()
        .unwrap();
    wallets.observed = fed.month;
    wallets
        .accounts
        .resize(household_sites.len(), Default::default());
    for (a, site) in wallets.accounts.iter_mut().zip(household_sites) {
        a.food_site = Some(site);
        a.need = 100.;
        a.hunger = 0.;
    }
    for f in &mut fed.politics.as_mut().unwrap().factions {
        f.cohesion = 1.;
    }
    let mut hungry = fed.clone();
    for a in &mut hungry
        .society
        .as_mut()
        .unwrap()
        .household_economy
        .as_mut()
        .unwrap()
        .accounts
    {
        a.hunger = 1.;
    }
    let mut stale = hungry.clone();
    stale
        .society
        .as_mut()
        .unwrap()
        .household_economy
        .as_mut()
        .unwrap()
        .observed -= 1;
    for h in [&mut fed, &mut hungry, &mut stale] {
        h.politics_year();
    }
    let bread = |h: &History| {
        let p = h.politics.as_ref().unwrap();
        p.household_factions
            .iter()
            .filter(|&&id| p.factions[id as usize].interest == 6)
            .count()
    };
    assert!(bread(&hungry) > bread(&fed));
    assert_eq!(
        bread(&stale),
        bread(&fed),
        "stale access is not current evidence"
    );
    assert_eq!(
        fed.sites
            .iter()
            .map(|s| s.economy.goods)
            .collect::<Vec<_>>(),
        hungry
            .sites
            .iter()
            .map(|s| s.economy.goods)
            .collect::<Vec<_>>()
    );
}
