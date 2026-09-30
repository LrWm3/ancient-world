use economics_compute_smoke::{
    accounting::Account,
    claim_relief::{self, Action},
    commitments::Agreement,
    compute::Backend,
    delivery_relief,
    employment::{ArrearsPolicy, Terms},
    finance::ContractId,
    financial_reporting::{Audit, Opening},
    forward,
    households::{self, market::EXAMPLE_HOUSEHOLD as HOME},
    model::*,
    opportunities::{Action as Permission, PERSON_TYPE},
    recovery::{ProceedingTerms, Receipt, Stage},
    scenario::{GRAIN, LABOR, PERSON, TOKEN},
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
};
const WORKER: AgentId = 89;
const BUYER: AgentId = 92;
const ESTATE: AgentId = 999;
const LAND: AssetId = 900;
const FORWARD: u32 = 60000;
fn fixture(relief: bool) -> (World, State) {
    let (mut w, mut s) = households::market::scenario().unwrap();
    w.town_market = None;
    s.town_market = Default::default();
    w.activities.orders.clear();
    s.balances.clear();
    s.balances.insert((HOME, TOKEN), 1);
    s.balances.insert((BUYER, TOKEN), 2);
    w.households[0].governance.charter.support_member_wages = true;
    for p in &mut w.participants {
        p.needs.clear();
        p.capacity.quantity = if p.agent == WORKER { 2 } else { 0 };
    }
    let law = w.transaction_policy.as_mut().unwrap();
    law.permissions
        .insert((PERSON_TYPE, Permission::CapacityTrade));
    let authority = law.authority;
    w.agents.push(Agent {
        id: ESTATE,
        name: "custody".into(),
    });
    w.assets.push(Asset {
        id: LAND,
        owner: authority,
        kind: 1,
    });
    w.rights.push(UseRight {
        id: LAND,
        holder: PERSON,
        asset: LAND,
        from: 1,
        through: 24,
        output_owner: PERSON,
    });
    w.agreements.push(Agreement {
        id: LAND,
        right: LAND,
        creditor: authority,
        debtor: PERSON,
        activated: 1,
        payment: Amount::new(GRAIN, 2),
    });
    w.employment.push(Terms {
        id: 1,
        employer: PERSON,
        worker: WORKER,
        from: 1,
        through: 1,
        capacity: Amount::new(LABOR, 2),
        wage_per_unit: Amount::new(TOKEN, 3),
        on_arrears: ArrearsPolicy::SuspendDelivery,
        rank: 0,
    });
    w.prepaid_deliveries.push(forward::direct::Terms {
        id: FORWARD,
        seller: PERSON,
        buyer: BUYER,
        month: 1,
        due: 3,
        goods: Amount::new(GRAIN, 2),
        prepayment: Amount::new(TOKEN, 2),
    });
    w.recovery.proceedings.push(ProceedingTerms {
        id: 1,
        debtor: PERSON,
        authority,
        estate: ESTATE,
        denomination: TOKEN,
        opening_month: 14,
        earliest_close: 14,
        assets: vec![],
        discharge_deficiency: true,
    });
    if relief {
        let wage = claim_relief::Terms {
            id: 1,
            proceeding: 1,
            contract: ContractId::Wages(1),
            original_due: 2,
            debtor: PERSON,
            creditor: WORKER,
            month: 14,
            expected_due: 2,
            expected_remaining: 3,
            action: Action::WriteOff { quantity: 1 },
        };
        let mut last_wage = wage.clone();
        last_wage.id = 2;
        last_wage.month = 15;
        last_wage.expected_remaining = 2;
        last_wage.action = Action::WriteOff { quantity: 2 };
        let land = claim_relief::Terms {
            id: 3,
            proceeding: 1,
            contract: ContractId::Land(LAND),
            original_due: 13,
            debtor: PERSON,
            creditor: authority,
            month: 14,
            expected_due: 13,
            expected_remaining: 2,
            action: Action::Extend { due: 16 },
        };
        let mut last_land = land.clone();
        last_land.id = 4;
        last_land.month = 17;
        last_land.expected_due = 16;
        last_land.action = Action::WriteOff { quantity: 2 };
        w.recovery.claim_relief = vec![wage, last_wage, land, last_land];
        let delivery = delivery_relief::Terms {
            id: 1,
            proceeding: 1,
            contract: FORWARD,
            debtor: PERSON,
            creditor: BUYER,
            month: 14,
            expected_due: 3,
            expected_remaining: 2,
            action: delivery_relief::Action::Extend { due: 17 },
        };
        let mut last_delivery = delivery.clone();
        last_delivery.id = 2;
        last_delivery.month = 18;
        last_delivery.expected_due = 17;
        last_delivery.action = delivery_relief::Action::WriteOff { quantity: 2 };
        w.recovery.delivery_relief = vec![delivery, last_delivery];
    }
    (w, s)
}
fn audit(w: &World, s: &State) -> Audit {
    Audit::with_opening(
        w,
        s,
        TOKEN,
        Opening {
            assets: w.assets.iter().map(|a| (a.id, 0)).collect(),
            exchange_values: [(GRAIN, 1)].into(),
            dues: Some(economics_compute_smoke::dues_accounting::Valuation(
                [(LAND, 1)].into(),
            )),
            ..Default::default()
        },
    )
    .unwrap()
}
fn through(a: &mut Audit, sim: &mut Simulation, month: u32) {
    while sim.state.month <= month {
        a.step(sim)
            .unwrap_or_else(|e| panic!("{e} at {} {:?}", sim.state.month, sim.state.phase));
    }
}
#[test]
fn household_support_wages_rent_and_prepaid_delivery_share_one_recovery_lifecycle() {
    for relief in [false, true] {
        let (w, s) = fixture(relief);
        let mut a = audit(&w, &s);
        let mut b = a.clone();
        let mut cpu = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
        let mut reversed = w.clone();
        reversed.recovery.claim_relief.reverse();
        reversed.recovery.delivery_relief.reverse();
        let mut reference = Simulation::new(reversed, s.clone(), Backend::Reference).unwrap();
        through(&mut a, &mut cpu, 13);
        assert_eq!(cpu.state.balance(HOME, TOKEN), 0);
        assert_eq!(cpu.state.balance(WORKER, TOKEN), 3);
        assert_eq!(cpu.state.employment.earned[&(1, 1)].claim.outstanding(), 3);
        through(&mut a, &mut cpu, 14);
        let admitted = cpu
            .ledger
            .iter()
            .filter_map(|b| b.credit.as_ref())
            .flat_map(|b| &b.recovery)
            .find_map(|r| {
                if let Receipt::Admitted { claims, .. } = r {
                    Some(claims)
                } else {
                    None
                }
            })
            .unwrap();
        for id in [
            ContractId::Wages(1),
            ContractId::Land(LAND),
            ContractId::Forward(FORWARD),
        ] {
            assert!(admitted.iter().any(|c| c.contract == id));
        }
        let mut resumed = cpu.clone();
        let mut c = a.clone();
        through(&mut a, &mut cpu, 18);
        through(&mut b, &mut reference, 18);
        through(&mut c, &mut resumed, 18);
        assert_eq!(cpu.state, reference.state);
        assert_eq!(cpu.ledger, reference.ledger);
        assert_eq!(cpu.state, resumed.state);
        assert_eq!(a, b);
        assert_eq!(a, c);
        let mut replay = s;
        for batch in &cpu.ledger {
            commit(
                &w,
                &mut replay,
                batch,
                Backend::Reference,
                DEFAULT_EFFECT_LIMIT,
            )
            .unwrap();
        }
        assert_eq!(replay, cpu.state);
        let wage = &cpu.state.employment.earned[&(1, 1)];
        let rent = &cpu.state.obligations[&(LAND, 13)];
        let delivery = &cpu.state.exchange.forwards[&FORWARD];
        assert_eq!(wage.delivered, 2);
        assert_eq!(wage.claim.settled, 3);
        assert_eq!(
            (rent.paid, rent.in_kind_paid, delivery.delivered),
            (0, 0, 0)
        );
        assert_eq!(cpu.state.balance(WORKER, TOKEN), 3);
        assert_eq!(cpu.state.balance(PERSON, TOKEN), 0);
        assert_eq!(cpu.state.balance(ESTATE, TOKEN), 0);
        if relief {
            assert_eq!(cpu.state.credit.recovery.proceedings[&1].closed, Some(18));
            assert_eq!(
                (
                    wage.claim.outstanding(),
                    rent.outstanding(),
                    delivery.claim().outstanding()
                ),
                (0, 0, 0)
            );
            assert_eq!(a.book().balances()[&(PERSON, Account::DebtRelief)], -7);
            assert_eq!(a.book().balances()[&(WORKER, Account::CreditLoss)], 3);
            assert_eq!(a.book().balances()[&(BUYER, Account::CreditLoss)], 2);
        } else {
            assert_eq!(
                cpu.state.credit.recovery.proceedings[&1].stage,
                Stage::Active
            );
            assert_eq!(
                (
                    wage.claim.outstanding(),
                    rent.outstanding(),
                    delivery.claim().outstanding()
                ),
                (3, 2, 2)
            );
            assert!(
                !a.book()
                    .balances()
                    .contains_key(&(PERSON, Account::DebtRelief))
            );
        }
    }
}
