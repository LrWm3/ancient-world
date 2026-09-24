use economics_compute_smoke::{
    compute::Backend, forward::Event, scenario::*, simulation::Simulation, trading_scenario::*,
};
use std::collections::BTreeMap;

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let months: u32 = args
        .first()
        .map(|s| s.parse())
        .transpose()
        .map_err(|_| "months")?
        .unwrap_or(TRADING_MONTHS);
    let name = args.get(1).map(String::as_str).unwrap_or("trading-32");
    let provider_count = args
        .get(3)
        .map(|s| s.parse::<usize>())
        .transpose()
        .map_err(|_| "providers")?
        .unwrap_or(DEFAULT_PROVIDERS);
    let (mut w, s) = if args.get(3).is_some() {
        cash_scenario(provider_count, name != "trading-32-no-forward")?
    } else {
        named(name)?
    };
    if args.get(4).is_some_and(|s| s == "old-tools") {
        for t in &mut w.techniques {
            if t.equipment_kind.is_some() {
                t.output_multiplier = 1;
                for a in &mut t.services {
                    a.quantity *= TOOL_SERVICE_DIVISOR;
                }
            }
        }
    }
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::Reference)?;
    sim.run_months(months)?;
    let forwards = &sim.state.exchange.forwards;
    let scale = |x: i64| x as f64 / f64::from(STOCK_UNIT);
    let advances: i64 = forwards
        .values()
        .map(|c| i64::from(c.advance.quantity))
        .sum();
    let overdue: i64 = forwards
        .values()
        .filter(|c| c.due < sim.state.month)
        .map(|c| i64::from(c.claim().outstanding()))
        .sum();
    let mut reasons = BTreeMap::new();
    let mut deliveries = BTreeMap::new();
    let mut upfront = 0i64;
    for b in &sim.ledger {
        for t in &b.transactions {
            if let Some(Event::Rejected { reason, .. }) = &t.forward {
                *reasons.entry(format!("{reason:?}")).or_insert(0) += 1;
            }
            if let Some(d) = &t.delivery {
                *deliveries.entry(b.month).or_insert(0) += 1;
                upfront += i64::from(d.purchase.as_ref().unwrap().price.quantity);
            }
        }
    }
    println!("# {name}: {months} months");
    println!(
        "Tool deliveries: {}; upfront sales: {:.2} coins; forwards: {}; advances: {:.2} coins; overdue goods: {:.2}.",
        sim.state.exchange.contracts.len(),
        scale(upfront),
        forwards.len(),
        scale(advances),
        scale(overdue)
    );
    println!("Deliveries/month: {deliveries:?}\nRejected attempts: {reasons:?}");
    println!(
        "Food deficits: {}; warmth deficits: {}; unpaid tax: {:.2}; terminal people: {}.",
        sim.reports
            .iter()
            .map(|r| r.deficit(NUTRITION))
            .sum::<i32>(),
        sim.reports.iter().map(|r| r.deficit(WARMTH)).sum::<i32>(),
        scale(
            sim.state
                .obligations
                .values()
                .map(|o| i64::from(o.owed - o.paid))
                .sum()
        ),
        sim.state.terminal.len()
    );
    for c in forwards.values() {
        let purchase = sim.state.exchange.contracts[&c.id]
            .purchase
            .as_ref()
            .unwrap();
        let spot = &sim
            .world
            .market
            .as_ref()
            .unwrap()
            .cash
            .as_ref()
            .unwrap()
            .prices[&c.goods.resource];
        let cost = i64::from(purchase.price.quantity - c.advance.quantity)
            + i64::from(c.goods.quantity) * i64::from(spot.coins) / i64::from(spot.goods);
        println!(
            "Forecast buyer {}: tool price {:.2}, incremental value {:.2}, economic cost {:.2} coins.",
            c.debtor,
            scale(i64::from(purchase.price.quantity)),
            scale(i64::from(purchase.projection.incremental_value)),
            scale(cost)
        );
        println!(
            "Forward {} buyer {}: issued {}, due {}, commodity {}, paid {:.2}/{:.2}; advance {:.2} coins.",
            c.id,
            c.debtor,
            c.issued,
            c.due,
            c.goods.resource,
            scale(i64::from(c.delivered)),
            scale(i64::from(c.goods.quantity)),
            scale(i64::from(c.advance.quantity))
        );
    }
    for id in providers(provider_count) {
        println!(
            "Provider {id}: grain {:.2}, coins {:.2}.",
            scale(i64::from(sim.state.balance(id, GRAIN))),
            scale(i64::from(sim.state.balance(id, TOKEN)))
        );
    }
    if args.get(2).is_some_and(|s| s == "cpu") {
        let mut cpu = Simulation::new(w, s, Backend::CubeCpu)?;
        cpu.run_months(months)?;
        assert_eq!(cpu.state, sim.state);
        assert_eq!(cpu.ledger, sim.ledger);
        assert_eq!(cpu.reports, sim.reports);
        println!("Full CPU/reference equality.");
    }
    Ok(())
}
