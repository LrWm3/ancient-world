use economics_compute_smoke::{compute::Backend, model::*, scenario::*, simulation::Simulation};
use std::collections::BTreeMap;

fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() > 1 || args.first().is_some_and(|a| a == "--help") {
        println!(
            "Usage: cargo +1.92.0 run --locked -- [scenario]\nNine-month controls: baseline, short-food, no-seed, no-right, short-right, missed-work, no-need, disabled, continuing-first, new-first\n60-month scenarios: {}\nForaging controls (nine months): {}\nSpecialized activities (36 months): specialized-activities, specialized-32\nHouseholds (24 months): households-32\nTool trading (72 months): trading-32, trading-32-no-forward, trading-32-share, trading-32-plots, trading-32-no-plots\nOpportunity marketplace (36 months): opportunity-farming, opportunity-two-plots, opportunity-one-plot\nAll scenarios use CubeCL CPU settlement.",
            [
                LONG_SCENARIOS,
                CONDITION_SCENARIOS,
                PLANNING_SCENARIOS,
                TOOL_SCENARIOS,
                EXPERIENCE_SCENARIOS,
                AGREEMENT_SCENARIOS,
                ACCESS_SCENARIOS,
                CURRENCY_SCENARIOS,
                MULTI_PERSON_SCENARIOS,
                PAYMENT_SCENARIOS
            ]
            .concat()
            .join(", "),
            FORAGING_SCENARIOS.join(", ")
        );
        return if args.len() > 1 {
            Err("expected at most one scenario".into())
        } else {
            Ok(())
        };
    }
    let name = args.first().map(String::as_str).unwrap_or("baseline");
    let (world, initial) = named(name)?;
    let start_month = initial.month;
    let trading = world.market.is_some();
    let mut simulation = Simulation::new(world, initial, Backend::CubeCpu)?;
    let months = scenario_months(name);
    simulation.run_months(months)?;
    println!(
        "# Economics CPU run: {name}\n\nDuration: {months} months, starting at month {start_month}. Backend: CubeCL CPU; deterministic conditions.\n"
    );
    for membership in simulation.state.memberships.values() {
        println!(
            "- Agent {} accepted membership offer {} with organization {} (role {}) in month {}.",
            membership.member,
            membership.source_offer,
            membership.organization,
            membership.role,
            membership.accepted_month
        );
    }
    for round in simulation
        .ledger
        .iter()
        .filter_map(|b| b.allocation.as_ref())
    {
        println!(
            "- Offers {:?} allocation at month {}: {:?}, seed {}.",
            round.offers, round.context.round, round.policy, round.context.seed
        );
        println!("  - Final applicant/offer assignments: {:?}.", round.awards);
        for receipt in &round.receipts {
            println!(
                "  - Applicant {}: priority {}, requested {}, grant {}, {:?}.",
                receipt.claim.id,
                receipt.claim.priority,
                receipt.claim.requested,
                receipt.offered,
                receipt.outcome
            );
        }
    }
    if trading {
        println!(
            "- Stock and money amounts use hundredths: 100 ticks = one unit. Cash-market labor uses quarter units; legacy controls retain whole labor units."
        );
        println!(
            "- Tool providers: {}; quote/output share setting: {}%.",
            economics_compute_smoke::trading_scenario::DEFAULT_PROVIDERS,
            simulation.world.market.as_ref().unwrap().capture_percent
        );
        println!(
            "- Delivered tool contracts: {}.",
            simulation.state.exchange.contracts.len()
        );
        if let Some(policy) = economics_compute_smoke::forward::policy(&simulation.world) {
            println!(
                "- State buy/sell/forward prices: 0.75 / 1.50 / 0.50 of base value; tax amounts and issuance unchanged."
            );
            let advances: i64 = simulation
                .state
                .exchange
                .forwards
                .values()
                .map(|c| i64::from(c.advance.quantity))
                .sum();
            let overdue: i64 = simulation
                .state
                .exchange
                .forwards
                .values()
                .filter(|c| c.due < simulation.state.month)
                .map(|c| i64::from(c.goods.quantity - c.delivered))
                .sum();
            println!(
                "- Upfront coin pricing over {} projected months; state forwards enabled: {}. No ongoing tool royalties.",
                policy.months, policy.enabled
            );
            println!(
                "- Production forwards: {}; advanced {} coin ticks; overdue {} commodity ticks.",
                simulation.state.exchange.forwards.len(),
                advances,
                overdue
            );
        } else {
            for assessment in economics_compute_smoke::trading_scenario::assess(
                &simulation,
                economics_compute_smoke::trading_scenario::DEFAULT_PROVIDERS,
            ) {
                println!(
                    "- Provider {}: final-24-month food royalties {} ticks; material barter cost {} ticks; recurring support screen {}.",
                    assessment.provider,
                    assessment.food_income,
                    assessment.material_spend,
                    assessment.supported
                );
            }
        }
    }
    for household in &simulation.world.households {
        let living: Vec<_> =
            economics_compute_smoke::households::members(household, &simulation.state).collect();
        let inventory: Vec<_> = simulation
            .world
            .resources
            .iter()
            .filter_map(|r| {
                let quantity = simulation.state.balance(household.agent, r.id);
                (quantity > 0).then(|| format!("{}: {}", r.name, quantity))
            })
            .collect();
        println!(
            "- Household {}: living adults {:?}; pooled inventory [{}].",
            household.agent,
            living,
            inventory.join(", ")
        );
    }
    for ((agent, kind), points) in &simulation.state.practice {
        println!("- Agent {agent}, competency {kind}: {points} practice points.");
    }
    for o in simulation.state.obligations.values() {
        println!(
            "- Agreement {} due {}: owed {}, settled {}, native paid {}, arrears {}.",
            o.agreement,
            o.due,
            o.owed,
            o.paid,
            o.in_kind_paid,
            o.owed - o.paid
        );
    }
    for a in simulation.state.accepted_agreements.values() {
        println!(
            "- Accepted access offer {} in month {}; annual payment {}.",
            a.id, a.activated, a.payment.quantity
        );
    }
    println!("- Payment policy: {:?}.", simulation.world.payment_policy);
    let arrears_months = simulation
        .reports
        .iter()
        .filter(|r| r.obligations.values().any(|o| o.paid < o.owed))
        .count();
    let active_arrears_months = simulation
        .reports
        .iter()
        .filter(|r| r.terminal.is_none() && r.obligations.values().any(|o| o.paid < o.owed))
        .count();
    let blocked = simulation
        .ledger
        .iter()
        .flat_map(|b| &b.receipts)
        .filter(|r| r.reason == Reason::UnpaidObligation)
        .count();
    println!(
        "- Closing participant-months with arrears: {arrears_months} (active: {active_arrears_months}); blocked new-work requests: {blocked}."
    );
    // Columns follow the catalog rather than treating nutrition as the only need.
    let resources: BTreeMap<_, _> = simulation
        .world
        .resources
        .iter()
        .map(|r| (r.id, r))
        .collect();
    let needs: BTreeMap<_, _> = simulation
        .world
        .participants
        .iter()
        .flat_map(|p| &p.needs)
        .map(|n| (n.resource, n))
        .collect();
    let balances: Vec<_> = resources
        .values()
        .filter(|r| {
            r.kind != ResourceKind::Fulfillment
                && simulation
                    .reports
                    .iter()
                    .any(|row| row.balances.contains_key(&r.id))
        })
        .collect();
    print!("| Month | Agent | Closing status |");
    for resource in &balances {
        print!(" {} |", resource.name);
    }
    for &id in needs.keys() {
        print!(
            " {} fulfilled | {} deficit |",
            resources[&id].name, resources[&id].name
        );
    }
    println!();
    println!("|{}", " --- |".repeat(3 + balances.len() + 2 * needs.len()));
    for row in &simulation.reports {
        print!(
            "| {} | {} | {} |",
            row.month,
            row.agent,
            row.terminal
                .as_ref()
                .map(|t| t.state.as_str())
                .unwrap_or("active")
        );
        for resource in &balances {
            print!(
                " {} |",
                row.balances.get(&resource.id).copied().unwrap_or(0)
            );
        }
        for &id in needs.keys() {
            print!(" {} | {} |", row.fulfilled(id), row.deficit(id));
        }
        println!();
    }
    println!("\n## Process and decision trace\n");
    for batch in &simulation.ledger {
        if let Some(household) = &batch.household {
            for reservation in household.reservations.iter().filter(|r| r.allocated > 0) {
                println!(
                    "- Month {} {:?}: household {} grants member {} {} ticks of resource {} for {:?}.",
                    batch.month,
                    batch.phase,
                    reservation.request.household,
                    reservation.request.member,
                    reservation.allocated,
                    reservation.request.resource,
                    reservation.request.purpose
                );
            }
            for decision in household.labor.iter().filter(|d| d.granted > 0) {
                println!(
                    "- Month {}: household {} assigns {} labor ticks to {:?}; projected work value {} -> {}.",
                    batch.month,
                    decision.household,
                    decision.granted,
                    decision.recipient,
                    decision.baseline_value,
                    decision.projected_value
                );
            }
        }
        if let Some(request) = &batch.plot_request {
            println!("- Month {}: plot request {:?}.", batch.month, request);
        }
        if let Some(s) = &batch.commitments
            && (!s.protected.is_empty() || s.obligations.values().any(|o| o.paid < o.owed))
        {
            println!(
                "- Month {} {:?}: payment policy {:?}, protected stock {:?}, unpaid obligations {}.",
                batch.month,
                batch.phase,
                s.policy,
                s.protected,
                s.obligations.values().filter(|o| o.paid < o.owed).count()
            );
        }
        if let Some(id) = batch.accept_access {
            println!(
                "- Month {} {:?}: accept access offer {}.",
                batch.month, batch.phase, id
            );
        }
        if batch.phase == Phase::Productive {
            for receipt in &batch.receipts {
                let process = receipt
                    .definition
                    .map(|id| simulation.world.definition(id).name.as_str())
                    .unwrap_or("no new process");
                let need = receipt
                    .need
                    .map(|id| resources[&id].name.as_str())
                    .unwrap_or("fixture intent");
                println!(
                    "- Month {}: agent {}, {process}, need {need}: {:?}; work requested {}, allocated {}, completed {}.",
                    batch.month,
                    receipt.agent,
                    receipt.reason,
                    receipt.requested,
                    receipt.allocated,
                    receipt.completed
                );
            }
        }
        for transaction in &batch.transactions {
            if transaction.delivery.is_some() || transaction.forward.is_some() {
                println!(
                    "- Month {}: delivery {:?}; forward {:?}.",
                    batch.month, transaction.delivery, transaction.forward
                );
            }
            let effects: Vec<_> = transaction
                .effects
                .iter()
                .map(|e| {
                    format!(
                        "agent {} {} {:+}",
                        e.account.0, resources[&e.account.1].name, e.delta
                    )
                })
                .collect();
            println!(
                "- Month {} {:?}: {}; {}{}{}{}.",
                batch.month,
                batch.phase,
                transaction.cause,
                effects.join(", "),
                transaction
                    .trade
                    .as_ref()
                    .map(|t| format!("; offer {} accepted by {}", t.offer, t.buyer))
                    .unwrap_or_default(),
                transaction
                    .technique_use
                    .as_ref()
                    .map(|u| format!("; technique {}, equipment {:?}", u.technique, u.asset))
                    .unwrap_or_default(),
                transaction
                    .process
                    .as_ref()
                    .map(|p| format!(
                        "; instance {} -> {:?}, stage {}, elapsed {}",
                        p.after.id,
                        p.after.status,
                        p.after.stage + 1,
                        p.after.elapsed
                    ))
                    .unwrap_or_default()
            );
        }
    }
    println!("\n## Consequence forecasts\n");
    println!(
        "Predictions assume the named rollout policy continues. Later replanning or unobserved shocks can change later outcomes."
    );
    for batch in &simulation.ledger {
        if let Some(decision) = &batch.decision {
            let chosen = &decision.alternatives[decision.selected];
            let observed: Vec<_> = chosen
                .outcomes
                .iter()
                .filter_map(|prediction| {
                    simulation
                        .reports
                        .iter()
                        .find(|actual| {
                            actual.month == prediction.month && actual.agent == prediction.agent
                        })
                        .map(|actual| (prediction, actual))
                })
                .collect();
            let matches = observed
                .iter()
                .filter(|(prediction, actual)| prediction == actual)
                .count();
            let first_matches = observed
                .iter()
                .filter(|(prediction, _)| prediction.month == decision.month)
                .all(|(prediction, actual)| prediction == actual);
            println!(
                "- Month {} through {}: chose alternative {}; first month matches actual: {}; observed forecast rows matching: {}/{}.",
                decision.month,
                decision.through,
                decision.selected,
                first_matches,
                matches,
                observed.len()
            );
            println!(
                "  - Search: {}; generated {}, rejected {}, limit {}, budget exhausted {}.",
                decision.search_name,
                decision.candidates_generated,
                decision.candidates_rejected,
                decision.search_budget.max_candidates,
                decision.search_budget_exhausted
            );
            for reason in &decision.rejection_reasons {
                println!("  - Rejected commitment plan: {reason}");
            }
            for (i, candidate) in decision.alternatives.iter().enumerate() {
                println!(
                    "  - Alternative {i}: steps {:?}; work {:?}; reason {}; score {:?}; first work {:?}.",
                    candidate.plan.steps,
                    candidate.plan.work,
                    candidate.plan.explanation,
                    candidate.score,
                    candidate.first_work
                );
                for agreement in &candidate.commitments.processes {
                    println!(
                        "    - Commitment {:?}: {:?} by forecast end.",
                        agreement.identity,
                        agreement.evaluate(decision.through).status
                    );
                }
            }
        }
    }
    println!("\n## Conditions and lifecycle\n");
    for batch in &simulation.ledger {
        if let Some(settlement) = &batch.maintenance {
            for c in &settlement.changes {
                println!(
                    "- Month {}: agent {}, {} supplied {}/{}, deprivation {} -> {}, adverse months {}, condition modifier {} per mille.",
                    batch.month,
                    c.subject,
                    resources[&c.resource].name,
                    c.supplied,
                    c.required,
                    c.before.deprivation,
                    c.after.deprivation,
                    c.after.adverse_months,
                    c.capacity_permille
                );
            }
            for t in &settlement.transitions {
                println!(
                    "- Month {}: agent {} became {} due to {}.",
                    t.month, t.subject, t.state, resources[&t.reason].name
                );
            }
        }
    }
    println!("\n## Totals\n");
    for &id in needs.keys() {
        let deficit: i64 = simulation
            .reports
            .iter()
            .map(|r| i64::from(r.deficit(id)))
            .sum();
        println!("- Total unmet {}: {deficit}.", resources[&id].name);
    }
    for d in &simulation.world.definitions {
        if d.execution != Execution::Productive {
            continue;
        }
        let instances: Vec<_> = simulation
            .state
            .processes
            .values()
            .filter(|p| p.definition == d.id && p.reserved_through >= start_month)
            .collect();
        let completed: Vec<_> = instances
            .iter()
            .filter(|p| p.status == Status::Completed)
            .map(|p| p.reserved_through)
            .collect();
        let aborted = instances
            .iter()
            .filter(|p| p.status == Status::Aborted)
            .count();
        let active = instances
            .iter()
            .filter(|p| p.status == Status::Active)
            .count();
        println!(
            "- {}: completed at months {:?}; aborted {aborted}; active {active}.",
            d.name, completed
        );
    }
    for asset in simulation.state.equipment.values() {
        println!(
            "- Equipment {}: owner {}, remaining uses {}, last used {:?}, plot {:?}.",
            asset.id, asset.owner, asset.remaining_uses, asset.last_used_month, asset.attached_to
        );
    }
    for offer in &simulation.world.offers {
        println!(
            "- Offer {}: filled {}; seller {} holds {} payment units.",
            offer.id,
            simulation.state.filled_offers.contains(&offer.id),
            offer.seller,
            simulation.state.balance(offer.seller, offer.price.resource)
        );
    }
    println!("- Committed batches: {}.", simulation.ledger.len());
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("Simulation failed: {error}");
        std::process::exit(1);
    }
}
