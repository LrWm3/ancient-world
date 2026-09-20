//! Reproducible diagnostic; prints Markdown, changes no production policy.
use economics_compute_smoke::{
    commitments::PaymentPolicy,
    compute::Backend,
    equipment::{DurableAsset, Offer},
    model::*,
    scenario::*,
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
};

const RUN_MONTHS: u32 = 60;
const DECISION_MONTHS: u32 = 18;

fn fixture(tool: bool, experience: bool, protected: bool) -> (World, State) {
    let (mut world, mut state) = named("annual-access").unwrap();
    world.decision_horizon = Some(DECISION_MONTHS);
    if protected {
        world.payment_policy = PaymentPolicy::ProtectEssentials;
    }
    // All main comparison arms contain the same state-owned tool endowment.
    state.equipment.insert(
        TOOL,
        DurableAsset {
            attached_to: None,
            id: TOOL,
            owner: STATE_AGENT,
            kind: 2,
            remaining_uses: 6,
            last_used_month: None,
        },
    );
    if tool {
        let (catalog, _) = named("tool-beneficial").unwrap();
        world.techniques.extend(catalog.techniques);
        world.offers.push(Offer {
            id: 1,
            seller: STATE_AGENT,
            asset: TOOL,
            price: Amount::new(GRAIN, 3),
        });
    }
    if experience {
        let (catalog, _) = named("experience-manual").unwrap();
        world.techniques.extend(catalog.techniques);
        world.practice_rules.extend(catalog.practice_rules);
    }
    (world, state)
}

fn report(name: &str, world: World, initial: State) -> Result<(), String> {
    let mut sim = Simulation::new(world.clone(), initial.clone(), Backend::CubeCpu)?;
    sim.run_months(RUN_MONTHS)?;
    let mut reference = Simulation::new(world.clone(), initial.clone(), Backend::Reference)?;
    reference.run_months(RUN_MONTHS)?;
    assert_eq!(sim.state, reference.state);
    assert_eq!(sim.ledger, reference.ledger);
    assert_eq!(sim.reports, reference.reports);
    let mut starts = Vec::new();
    let mut harvests = Vec::new();
    let mut crop_labor = 0;
    let mut fuel_labor = 0;
    let mut eaten = 0;
    let mut rent = 0;
    let mut purchase = 0;
    let mut uses = 0;
    let mut state = initial.clone();
    let mut waits = Vec::new();
    for b in &sim.ledger {
        let grow_start = b.transactions.iter().any(|t| {
            t.process.as_ref().is_some_and(|p| {
                p.before.is_none()
                    && p.after.definition == GROW
                    && p.after.status != Status::Aborted
            })
        });
        if b.phase == Phase::Productive
            && !state
                .processes
                .values()
                .any(|p| p.status == Status::Active && p.asset == Some(PLOT))
            && !grow_start
        {
            let reasons: Vec<_> = b
                .receipts
                .iter()
                .filter(|r| {
                    r.need == Some(NUTRITION) || matches!(r.definition, Some(GROW | PREPARE_FUEL))
                })
                .map(|r| {
                    format!(
                        "definition {:?}, {:?}, completed {}",
                        r.definition, r.reason, r.completed
                    )
                })
                .collect();
            waits.push(format!(
                "{}: grain {}, seed {}, labor {}, access {}, receipt {:?}",
                b.month,
                state.balance(PERSON, GRAIN),
                state.balance(PERSON, SEED),
                state.balance(PERSON, LABOR),
                economics_compute_smoke::commitments::can_start(&world, &state, world.rights[0].id),
                reasons
            ));
        }
        for t in &b.transactions {
            if let Some(p) = &t.process {
                let labor: i32 = t
                    .effects
                    .iter()
                    .filter(|e| e.account == (PERSON, LABOR) && e.delta < 0)
                    .map(|e| -e.delta)
                    .sum();
                if p.after.definition == GROW {
                    crop_labor += labor;
                    if p.before.is_none() && p.after.status != Status::Aborted {
                        starts.push(b.month);
                    }
                    if p.after.status == Status::Completed {
                        harvests.push(b.month);
                    }
                }
                if p.after.definition == PREPARE_FUEL {
                    fuel_labor += labor;
                }
                if p.after.definition == CONSUME {
                    eaten += t
                        .effects
                        .iter()
                        .filter(|e| e.account == (PERSON, GRAIN) && e.delta < 0)
                        .map(|e| -e.delta)
                        .sum::<i32>();
                }
            }
            if t.trade.is_some() {
                purchase += t
                    .effects
                    .iter()
                    .filter(|e| e.account == (PERSON, GRAIN) && e.delta < 0)
                    .map(|e| -e.delta)
                    .sum::<i32>();
            }
            if t.technique_use.as_ref().is_some_and(|u| u.asset.is_some()) {
                uses += 1;
            }
        }
        if b.commitments.is_some() {
            rent += b
                .transactions
                .iter()
                .flat_map(|t| &t.effects)
                .filter(|e| e.account == (PERSON, GRAIN) && e.delta < 0)
                .map(|e| -e.delta)
                .sum::<i32>();
        }
        commit(
            &world,
            &mut state,
            b,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT,
        )?;
    }
    assert_eq!(state, sim.state);
    let idle: i32 = sim
        .reports
        .iter()
        .map(|r| r.balances.get(&LABOR).copied().unwrap_or(0))
        .sum();
    let deficits: i32 = sim.reports.iter().map(|r| r.deficit(NUTRITION)).sum();
    let warmth: i32 = sim.reports.iter().map(|r| r.deficit(WARMTH)).sum();
    let blocked = sim
        .ledger
        .iter()
        .flat_map(|b| &b.receipts)
        .filter(|r| r.reason == Reason::UnpaidObligation)
        .count();
    let arrears = sim
        .reports
        .iter()
        .filter(|r| r.terminal.is_none() && r.obligations.values().any(|o| o.paid < o.owed))
        .count();
    let gross = harvests.len() as i32 * 8;
    assert_eq!(
        initial.balance(PERSON, GRAIN) + gross - eaten - rent - purchase,
        sim.state.balance(PERSON, GRAIN)
    );
    assert_eq!(crop_labor + fuel_labor + idle, 120);
    println!("## {name}\n");
    println!(
        "| Harvests | Food / warmth deficits | Rent | Tool cost / uses | Crop / fuel / idle labor | Ending grain | Active arrears months / blocked requests |"
    );
    println!("| --- | --- | --- | --- | --- | --- | --- |");
    println!(
        "| {} | {deficits} / {warmth} | {rent} | {purchase} / {uses} | {crop_labor} / {fuel_labor} / {idle} | {} | {arrears} / {blocked} |\n",
        harvests.len(),
        sim.state.balance(PERSON, GRAIN)
    );
    println!(
        "- Starts: {starts:?}.\n- Harvests: {harvests:?}.\n- Grain: initial {} + output {gross} - consumed {eaten} - rent {rent} - tool {purchase} = {}.\n- Terminal state: {:?}.\n- Idle plot boundaries:\n",
        initial.balance(PERSON, GRAIN),
        sim.state.balance(PERSON, GRAIN),
        sim.state.terminal
    );
    for line in waits {
        println!("  - {line}");
    }
    println!();
    Ok(())
}

fn main() -> Result<(), String> {
    println!(
        "# Production bottleneck audit\n\n60 months, CubeCL CPU and reference equality; main comparison decision horizon 18, configured candidate horizon 6 (contract-linked inputs extend for lead time); diagnostic overrides are labelled.\n"
    );
    let witness_only = std::env::args().any(|a| a == "--calendar-only");
    let starts_only = std::env::args().any(|a| a == "--starts-only");
    if !witness_only {
        if !starts_only {
            for protected in [false, true] {
                for (name, tool, experience) in [
                    ("baseline", false, false),
                    ("tool-offer", true, false),
                    ("experience", false, true),
                    ("both", true, true),
                ] {
                    let (w, s) = fixture(tool, experience, protected);
                    report(
                        &format!(
                            "{name} / {}",
                            if protected { "protected" } else { "debt first" }
                        ),
                        w,
                        s,
                    )?;
                }
            }
            // Endowment intervention, separate from the controlled eight-arm comparison.
            let (mut w, mut s) = fixture(true, false, false);
            w.offers.clear();
            s.equipment.get_mut(&TOOL).unwrap().owner = PERSON;
            report("diagnostic: already-owned tool", w, s)?;
        }
        // Bypass only autonomous start generation using the existing dated-intent API.
        for protected in [false, true] {
            let (mut w, s) = fixture(false, false, protected);
            for month in (7..=55).step_by(6) {
                w.scheduled_starts.push(ScheduledStart {
                    month,
                    agent: PERSON,
                    definition: GROW,
                });
            }
            report(
                &format!(
                    "diagnostic: scheduled starts / {}",
                    if protected { "protected" } else { "debt first" }
                ),
                w,
                s,
            )?;
        }
    }
    // Feasibility witness, not an autonomous policy: suppress distant buffer
    // wishes and submit a complete dated crop/fuel calendar. The ordinary
    // resolver and settlement still check all grants, inputs and completions.
    let (mut w, s) = fixture(false, false, false);
    w.horizon = 1;
    w.priority = Priority::ContinuingFirst;
    for month in (1..=55).step_by(6) {
        w.scheduled_starts.push(ScheduledStart {
            month,
            agent: PERSON,
            definition: GROW,
        });
        for offset in [1, 2, 3] {
            w.scheduled_starts.push(ScheduledStart {
                month: month + offset,
                agent: PERSON,
                definition: PREPARE_FUEL,
            });
        }
    }
    report(
        "diagnostic: explicit crop/fuel calendar (one-month configured buffer, contract lead-time extension, no forecast)",
        w,
        s,
    )?;
    Ok(())
}
