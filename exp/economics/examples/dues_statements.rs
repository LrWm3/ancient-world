//! One annual land bill paid partly in grain and partly in accepted coins.
use economics_compute_smoke::{
    activities::CoinPayment,
    commitments::{Agreement, MONTHS_PER_YEAR},
    compute::Backend,
    financial_reporting::Audit,
    model::{Amount, Resource, ResourceKind},
    scenario::{self, GRAIN, PERSON, PLOT, STATE_AGENT, TOKEN},
    simulation::Simulation,
};
use std::collections::BTreeMap;
const AGREEMENT_ID: u32 = 1;
const BILL_UNITS: i32 = 2;
const NATIVE_UNIT_VALUE: i128 = 3;
const OPENING_GRAIN_COST: i128 = 2;
const TENDER_COINS_PER_UNIT: i32 = 2;
const OPENING_COINS: i32 = 4;
fn main() -> Result<(), String> {
    let (mut world, mut state) = scenario::baseline();
    world.participants.clear();
    world.definitions.clear();
    world.rights[0].through = 1 + MONTHS_PER_YEAR;
    world.resources.push(Resource {
        id: TOKEN,
        name: "reporting coins".into(),
        kind: ResourceKind::Stock,
    });
    world.agreements.push(Agreement {
        id: AGREEMENT_ID,
        right: world.rights[0].id,
        creditor: STATE_AGENT,
        debtor: PERSON,
        activated: 1,
        payment: Amount::new(GRAIN, BILL_UNITS),
    });
    world.activities.coin_payments.insert(
        AGREEMENT_ID,
        CoinPayment {
            resource: TOKEN,
            coins_per_unit: TENDER_COINS_PER_UNIT,
        },
    );
    state.month = 1 + MONTHS_PER_YEAR;
    state.balances = BTreeMap::from([((PERSON, GRAIN), 1), ((PERSON, TOKEN), OPENING_COINS)]);
    let month = state.month;
    let mut audit = Audit::with_dues(
        &world,
        &state,
        TOKEN,
        BTreeMap::from([(PLOT, 0)]),
        BTreeMap::from([((PERSON, GRAIN), OPENING_GRAIN_COST)]),
        BTreeMap::from([(AGREEMENT_ID, NATIVE_UNIT_VALUE)]),
    )?;
    let mut sim = Simulation::new(world, state, Backend::CubeCpu)?;
    while sim.state.month == month {
        audit.step(&mut sim)?;
    }
    for agent in [PERSON, STATE_AGENT] {
        println!(
            "{}",
            audit
                .book()
                .statements(agent, month, month)?
                .markdown(TOKEN)
        );
    }
    Ok(())
}
