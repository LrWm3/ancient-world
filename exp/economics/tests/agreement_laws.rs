use economics_compute_smoke::{
    commitments,
    compute::Backend,
    credit,
    laws::{self, AgreementForm, Reason},
    model::*,
    opportunities::{self, Action, PERSON_TYPE},
    scenario::*,
    settlement::{self, DEFAULT_EFFECT_LIMIT},
    simulation::Simulation,
};
use std::collections::BTreeSet;

fn lease(forms: Option<BTreeSet<AgreementForm>>, backend: Backend) -> Simulation {
    let (mut w, s) = opportunities::scenario().unwrap();
    w.transaction_policy.as_mut().unwrap().agreement_forms = forms;
    Simulation::new(w, s, backend).unwrap()
}
fn mortgage(forms: Option<BTreeSet<AgreementForm>>, backend: Backend) -> Simulation {
    let (mut w, s) = credit::scenario("repaid").unwrap();
    let (policy_world, _) = opportunities::scenario().unwrap();
    let mut policy = policy_world.transaction_policy.unwrap();
    policy.permissions = BTreeSet::from([(PERSON_TYPE, Action::FinancedPurchase)]);
    policy.agreement_forms = forms;
    w.transaction_policy = Some(policy);
    Simulation::new(w, s, backend).unwrap()
}
fn acquire(s: &mut Simulation) {
    while s.state.phase != Phase::Acquire {
        s.step().unwrap();
    }
}
#[test]
fn recognition_is_separate_from_permission_and_legacy_is_explicit() {
    for forms in [
        None,
        Some(BTreeSet::new()),
        Some(BTreeSet::from([AgreementForm::LandUseLease])),
        Some(BTreeSet::from([AgreementForm::FinancedAssetPurchase])),
    ] {
        let mut l = lease(forms.clone(), Backend::CubeCpu);
        let mut m = mortgage(forms.clone(), Backend::CubeCpu);
        let land = forms
            .as_ref()
            .is_none_or(|f| f.contains(&AgreementForm::LandUseLease));
        let purchase = forms
            .as_ref()
            .is_none_or(|f| f.contains(&AgreementForm::FinancedAssetPurchase));
        assert!(opportunities::permits(
            &l.world,
            &l.state,
            PERSON,
            Action::LandAccess
        ));
        assert!(opportunities::permits(
            &m.world,
            &m.state,
            PERSON,
            Action::FinancedPurchase
        ));
        assert_eq!(
            opportunities::discover(&l.world, &l.state, PERSON)
                .iter()
                .any(|o| matches!(o, opportunities::Opportunity::StateAccess(_))),
            land
        );
        assert_eq!(
            !credit::discover(&m.world, &m.state, PERSON).is_empty(),
            purchase
        );
        if !land {
            assert_eq!(
                laws::evaluate_agreement(&l.world, &l.state, PERSON, AgreementForm::LandUseLease)
                    .reasons,
                vec![Reason::UnrecognizedForm {
                    form: AgreementForm::LandUseLease
                }]
            );
        }
        acquire(&mut l);
        assert_eq!(commitments::acceptance(&l.world, &l.state, 1).is_ok(), land);
        l.run_months(1).unwrap();
        m.run_months(1).unwrap();
        assert_eq!(!l.state.accepted_agreements.is_empty(), land);
        assert_eq!(!m.state.credit.loans.is_empty(), purchase);
        assert_eq!(
            credit::owner(&m.world, &m.state, PLOT),
            Some(if purchase { PERSON } else { STATE_AGENT })
        );
    }
}

#[test]
fn recognition_does_not_grant_permission_or_fund_a_purchase() {
    let mut m = mortgage(
        Some(BTreeSet::from([AgreementForm::FinancedAssetPurchase])),
        Backend::Reference,
    );
    m.world
        .transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .clear();
    assert!(
        !laws::evaluate_agreement(
            &m.world,
            &m.state,
            PERSON,
            AgreementForm::FinancedAssetPurchase
        )
        .allowed
    );
    m.run_months(1).unwrap();
    assert!(m.state.credit.loans.is_empty());
    let mut m = mortgage(None, Backend::Reference);
    m.world
        .credit
        .as_mut()
        .unwrap()
        .endowments
        .iter_mut()
        .find(|e| e.agent == PERSON)
        .unwrap()
        .amount
        .quantity = 1;
    m.run_months(1).unwrap();
    assert!(m.state.credit.loans.is_empty());
}

#[test]
fn pending_purchase_cannot_bypass_withdrawn_recognition() {
    let mut l = lease(Some(BTreeSet::new()), Backend::CubeCpu);
    acquire(&mut l);
    let before = l.state.clone();
    let mut forged = Batch::empty(&l.state);
    forged.accept_access = Some(1);
    assert!(
        settlement::commit(
            &l.world,
            &mut l.state,
            &forged,
            Backend::CubeCpu,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(before, l.state);
    let mut m = mortgage(None, Backend::CubeCpu);
    acquire(&mut m);
    let mut batch = Batch::empty(&m.state);
    batch.credit = credit::evaluate(&m.world, &m.state).unwrap();
    batch.transactions = batch.credit.as_ref().unwrap().transactions.clone();
    let before = m.state.clone();
    m.world.transaction_policy.as_mut().unwrap().agreement_forms = Some(BTreeSet::new());
    assert!(
        settlement::commit(
            &m.world,
            &mut m.state,
            &batch,
            Backend::CubeCpu,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(before, m.state);
    m.step().unwrap();
    assert!(m.state.credit.loans.is_empty());
}

#[test]
fn existing_leases_and_debts_survive_withdrawn_recognition() {
    for is_credit in [false, true] {
        let mut original = if is_credit {
            mortgage(None, Backend::Reference)
        } else {
            lease(None, Backend::Reference)
        };
        original.run_months(1).unwrap();
        let mut restricted_world = original.world.clone();
        restricted_world
            .transaction_policy
            .as_mut()
            .unwrap()
            .agreement_forms = Some(BTreeSet::new());
        let mut restricted =
            Simulation::new(restricted_world, original.state.clone(), Backend::CubeCpu).unwrap();
        let months = if is_credit { 5 } else { 14 };
        let report_start = original.reports.len();
        let ledger_start = original.ledger.len();
        original.run_months(months).unwrap();
        for _ in 0..months {
            restricted.run_months(1).unwrap();
        }
        assert_eq!(original.state, restricted.state);
        assert_eq!(&original.reports[report_start..], restricted.reports);
        assert_eq!(&original.ledger[ledger_start..], restricted.ledger);
        if is_credit {
            assert!(
                restricted
                    .state
                    .credit
                    .loans
                    .values()
                    .all(|l| l.debt().unwrap() == 0)
            );
        } else {
            assert!(!restricted.state.obligations.is_empty());
            assert!(
                restricted
                    .state
                    .obligations
                    .values()
                    .all(|o| o.paid == o.owed)
            );
            assert!(
                restricted
                    .state
                    .processes
                    .values()
                    .any(|p| p.definition == GROW && p.status == Status::Completed)
            );
        }
    }
}

#[test]
fn denied_forms_match_reference_batches_and_cpu_checkpoint_continuation() {
    for is_credit in [false, true] {
        let make = |backend| {
            if is_credit {
                mortgage(Some(BTreeSet::new()), backend)
            } else {
                lease(Some(BTreeSet::new()), backend)
            }
        };
        let mut reference = make(Backend::Reference);
        let mut cpu = make(Backend::CubeCpu);
        reference.run_months(6).unwrap();
        for _ in 0..6 {
            acquire(&mut cpu);
            cpu.state = Simulation::new(cpu.world.clone(), cpu.state.clone(), Backend::CubeCpu)
                .unwrap()
                .state;
            cpu.run_months(1).unwrap();
        }
        assert_eq!(reference.state, cpu.state);
        assert_eq!(reference.ledger, cpu.ledger);
        assert_eq!(reference.reports, cpu.reports);
    }
}

#[test]
fn withdrawing_recognition_does_not_block_collateral_enforcement() {
    let mut control = mortgage(None, Backend::Reference);
    control.world.credit.as_mut().unwrap().transfers.clear();
    control.run_months(1).unwrap();
    let mut w = control.world.clone();
    w.transaction_policy.as_mut().unwrap().agreement_forms = Some(BTreeSet::new());
    let mut restricted = Simulation::new(w, control.state.clone(), Backend::CubeCpu).unwrap();
    control.run_months(6).unwrap();
    restricted.run_months(6).unwrap();
    assert_eq!(control.state, restricted.state);
    assert_eq!(
        credit::owner(&restricted.world, &restricted.state, PLOT),
        Some(STATE_AGENT)
    );
    assert!(
        restricted
            .ledger
            .iter()
            .filter_map(|b| b.credit.as_ref())
            .flat_map(|b| &b.events)
            .any(|e| matches!(e, credit::Event::Enforced { .. }))
    );
}
