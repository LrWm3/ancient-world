//! Accepted access agreements and dated obligations; no party-role branches.
use crate::finance::{self, Condition, FailureRule, Transfer};
use crate::model::*;
use std::collections::{BTreeMap, BTreeSet};

pub const MONTHS_PER_YEAR: u32 = 12;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaymentPolicy {
    #[default]
    DebtFirst,
    ProtectEssentials,
}

/// An accepted offer. Its referenced right supplies the access and duration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Agreement {
    pub id: u32,
    pub right: u32,
    pub creditor: AgentId,
    pub debtor: AgentId,
    pub activated: u32,
    pub payment: Amount,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Obligation {
    pub agreement: u32,
    pub due: u32,
    pub owed: i32,
    pub paid: i32,
    /// Native commodity units actually received; coin settlement must not mint.
    pub in_kind_paid: i32,
}
impl Obligation {
    pub fn claim(&self, agreement: &Agreement) -> finance::Obligation {
        finance::Obligation {
            transfer: Transfer {
                from: agreement.debtor,
                to: agreement.creditor,
                amount: Amount::new(agreement.payment.resource, self.owed),
            },
            settled: self.paid,
            condition: Condition::OnOrAfterMonth(self.due),
            failure: FailureRule::BlockNewUse,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settlement {
    pub policy: PaymentPolicy,
    pub protected: BTreeMap<Account, i32>,
    pub obligations: BTreeMap<(u32, u32), Obligation>,
    pub transactions: Vec<Transaction>,
}

impl Agreement {
    pub fn contract(
        &self,
        world: &World,
        state: &State,
    ) -> Result<crate::agreements::Agreement, String> {
        use crate::agreements::{
            Agreement, Consequence, Grant, Identity, Obligation, PaymentTerms,
        };
        let right = world
            .rights
            .iter()
            .find(|r| r.id == self.right)
            .ok_or("missing agreement right")?;
        let consequence = Consequence::SuspendNewUse(Grant::LandUse(self.right));
        Ok(Agreement {
            identity: Identity::Land(self.id),
            grantor: crate::agreements::Counterparty::Agent(self.creditor),
            production: None,
            holder: self.debtor,
            accepted_month: self.activated,
            through: Some(right.through),
            grants: vec![Grant::LandUse(self.right)],
            payments: vec![PaymentTerms {
                transfer: Transfer {
                    from: self.debtor,
                    to: self.creditor,
                    amount: self.payment.clone(),
                },
                first_due: self
                    .activated
                    .checked_add(MONTHS_PER_YEAR)
                    .ok_or("due date overflow")?,
                interval_months: MONTHS_PER_YEAR,
                through: right.through,
                on_unpaid: consequence.clone(),
            }],
            obligations: state
                .obligations
                .values()
                .filter(|o| o.agreement == self.id)
                .map(|o| Obligation {
                    claim: o.claim(self),
                    on_unpaid: consequence.clone(),
                })
                .collect(),
        })
    }
}

pub fn can_start(world: &World, state: &State, right: u32) -> bool {
    if world.access_offers.iter().any(|o| o.right == right)
        && !state.accepted_agreements.values().any(|a| a.right == right)
    {
        return false;
    }
    active(world, state).filter(|a| a.right == right).all(|a| {
        a.contract(world, state).is_ok_and(|contract| {
            contract.permits(
                state.month,
                &crate::agreements::Grant::LandUse(right),
                crate::agreements::Use::Start,
            )
        })
    })
}

pub fn evaluate(world: &World, state: &State) -> Result<Settlement, String> {
    let mut obligations = state.obligations.clone();
    if state.phase == Phase::Due {
        for a in active(world, state) {
            for terms in a.contract(world, state)?.payments {
                let mut due = terms.first_due;
                while due <= state.month && due <= terms.through {
                    obligations.entry((a.id, due)).or_insert(Obligation {
                        agreement: a.id,
                        due,
                        owed: terms.transfer.amount.quantity,
                        paid: 0,
                        in_kind_paid: 0,
                    });
                    due = due
                        .checked_add(terms.interval_months)
                        .ok_or("due date overflow")?;
                }
            }
        }
    }
    // Oldest due first, stable agreement ID ties. Receipts cannot fund another
    // payment in this boundary: each debtor draws only its opening stock.
    let mut order: Vec<_> = obligations.keys().copied().collect();
    order.sort_by_key(|(agreement, due)| (*due, *agreement));
    let mut budget = state.balances.clone();
    let protected = protected_stock(world, state)?;
    for (account, quantity) in &protected {
        *budget.entry(*account).or_default() -= quantity;
    }
    let mut transactions = Vec::new();
    let mut stored = crate::storage::usage(world, &state.balances);
    for key in order {
        let o = obligations.get_mut(&key).unwrap();
        let a = active(world, state)
            .find(|a| a.id == o.agreement)
            .ok_or("unknown obligation agreement")?;
        if state.terminal.contains_key(&a.debtor) {
            continue;
        }
        let available = budget.entry((a.debtor, a.payment.resource)).or_default();
        let claim = o.claim(a);
        let paid = claim.payable(
            state.month,
            true,
            *available,
            crate::storage::room(world, &stored, a.creditor, a.payment.resource),
        );
        let previously_paid = o.in_kind_paid;
        if paid > 0 {
            *available -= paid;
            o.paid += paid;
            o.in_kind_paid += paid;
            transactions.push(Transaction {
                cause: format!("agreement {} due {}: pay {}", a.id, o.due, paid),
                effects: claim.payment(paid)?,
                process: None,
                technique_use: None,
                trade: None,
                stock_trade: None,
                forward: None,
                delivery: None,
                royalty: None,
            });
            crate::storage::apply(world, &mut stored, &transactions.last().unwrap().effects);
            if let Some(rule) = world.issuance.iter().find(|r| r.agreement == a.id) {
                let issued = o.in_kind_paid / rule.collected_per_token
                    - previously_paid / rule.collected_per_token;
                if issued > 0 {
                    transactions.push(Transaction {
                        cause: format!("issue currency: agreement {} due {}", a.id, o.due),
                        effects: vec![Effect {
                            account: (a.creditor, rule.token),
                            delta: issued,
                        }],
                        process: None,
                        technique_use: None,
                        trade: None,
                        stock_trade: None,
                        forward: None,
                        delivery: None,
                        royalty: None,
                    });
                }
            }
        }
        if let Some(alternative) = world.activities.coin_payments.get(&a.id) {
            let available = budget.entry((a.debtor, alternative.resource)).or_default();
            let units = (o.owed - o.paid).min(*available / alternative.coins_per_unit);
            if units > 0 {
                let coins = units
                    .checked_mul(alternative.coins_per_unit)
                    .ok_or("coin payment overflow")?;
                *available -= coins;
                o.paid += units;
                transactions.push(Transaction {
                    cause: format!(
                        "agreement {} due {}: pay {} units with {} coins",
                        a.id, o.due, units, coins
                    ),
                    effects: Transfer {
                        from: a.debtor,
                        to: a.creditor,
                        amount: Amount::new(alternative.resource, coins),
                    }
                    .effects()?,
                    process: None,
                    technique_use: None,
                    trade: None,
                    stock_trade: None,
                    forward: None,
                    delivery: None,
                    royalty: None,
                });
            }
        }
    }
    Ok(Settlement {
        policy: world.payment_policy,
        protected,
        obligations,
        transactions,
    })
}

pub fn validate(world: &World, state: &State) -> Result<(), String> {
    if world
        .open_access_offers
        .iter()
        .any(|id| !world.access_offers.iter().any(|a| a.id == *id))
    {
        return Err("unknown open access offer".into());
    }
    let mut ids = BTreeSet::new();
    let mut rights = BTreeSet::new();
    for a in world.agreements.iter().chain(&world.access_offers) {
        let right = world
            .rights
            .iter()
            .find(|r| r.id == a.right)
            .ok_or("unknown agreement right")?;
        if !ids.insert(a.id)
            || !rights.insert(a.right)
            || a.activated == 0
            || a.activated.checked_add(MONTHS_PER_YEAR).is_none()
            || a.activated != right.from
            || right.holder != a.debtor
            || (world.open_access_offers.contains(&a.id)
                && (a.debtor != a.creditor || right.output_owner != a.creditor))
            || (a.creditor == a.debtor && !world.open_access_offers.contains(&a.id))
            || a.payment.quantity <= 0
            || !world
                .assets
                .iter()
                .any(|s| s.id == right.asset && s.owner == a.creditor)
            || !world.agents.iter().any(|s| s.id == a.creditor)
            || !world
                .resources
                .iter()
                .any(|r| r.id == a.payment.resource && r.kind == ResourceKind::Stock)
        {
            return Err("invalid access agreement".into());
        }
    }
    for (&id, a) in &state.accepted_agreements {
        let template = world
            .access_offers
            .iter()
            .find(|o| o.id == id)
            .ok_or("unknown accepted offer")?;
        let mut expected = template.clone();
        expected.activated = a.activated;
        if world.open_access_offers.contains(&id) {
            if a.debtor == a.creditor || !world.agents.iter().any(|p| p.id == a.debtor) {
                return Err("invalid open offer applicant".into());
            }
            expected.debtor = a.debtor;
        }
        let right = world
            .rights
            .iter()
            .find(|r| r.id == a.right)
            .ok_or("missing accepted right")?;
        if *a != expected
            || a.activated < template.activated
            || a.activated > state.month
            || a.activated >= right.through
            || a.activated.checked_add(MONTHS_PER_YEAR).is_none()
        {
            return Err("invalid accepted agreement".into());
        }
    }
    for (&key, o) in &state.obligations {
        let a = active(world, state)
            .find(|a| a.id == o.agreement)
            .ok_or("unknown obligation")?;
        let through = world
            .rights
            .iter()
            .find(|r| r.id == a.right)
            .unwrap()
            .through;
        if key != (o.agreement, o.due)
            || o.due > state.month
            || o.due > through
            || o.due <= a.activated
            || !(o.due - a.activated).is_multiple_of(MONTHS_PER_YEAR)
            || o.owed != a.payment.quantity
            || o.paid < 0
            || o.paid > o.owed
            || o.in_kind_paid < 0
            || o.in_kind_paid > o.paid
        {
            return Err("invalid annual obligation".into());
        }
    }
    Ok(())
}

pub fn active<'a>(world: &'a World, state: &'a State) -> impl Iterator<Item = &'a Agreement> {
    world
        .agreements
        .iter()
        .chain(state.accepted_agreements.values())
}

/// Catalog rights attached to offers are dormant until acceptance commits.
pub fn acceptance(world: &World, state: &State, id: u32) -> Result<Agreement, String> {
    let offer = world
        .access_offers
        .iter()
        .find(|o| o.id == id)
        .ok_or("unknown access offer")?;
    if world.open_access_offers.contains(&id) {
        return Err("open access offer requires an explicit applicant".into());
    }
    acceptance_for(world, state, id, offer.debtor)
}

pub fn acceptance_for(
    world: &World,
    state: &State,
    id: u32,
    applicant: AgentId,
) -> Result<Agreement, String> {
    let template = world
        .access_offers
        .iter()
        .find(|o| o.id == id)
        .ok_or("unknown access offer")?;
    if !world.agents.iter().any(|a| a.id == applicant)
        || applicant == template.creditor
        || (!world.open_access_offers.contains(&id) && applicant != template.debtor)
    {
        return Err("invalid access applicant".into());
    }
    let mut bound = template.clone();
    bound.debtor = applicant;
    let offer = &bound;
    if !crate::opportunities::permits(
        world,
        state,
        offer.debtor,
        crate::opportunities::Action::LandAccess,
    ) || world
        .transaction_policy
        .as_ref()
        .is_some_and(|p| offer.creditor != p.authority)
    {
        return Err("state policy denies land access".into());
    }
    let right = world
        .rights
        .iter()
        .find(|r| r.id == offer.right)
        .ok_or("unknown offered right")?;
    if state.phase != Phase::Acquire
        || state.accepted_agreements.contains_key(&id)
        || state.month < offer.activated
        || state.month >= right.through
        || state.terminal.contains_key(&offer.debtor)
        || state.terminal.contains_key(&offer.creditor)
        || active(world, state).any(|a| {
            world
                .rights
                .iter()
                .any(|r| r.id == a.right && r.asset == right.asset && r.through >= state.month)
        })
        || state
            .processes
            .values()
            .any(|p| p.asset == Some(right.asset) && p.status == Status::Active)
    {
        return Err("unavailable access offer".into());
    }
    let mut agreement = offer.clone();
    agreement.activated = state.month;
    agreement
        .activated
        .checked_add(MONTHS_PER_YEAR)
        .ok_or("due date overflow")?;
    Ok(agreement)
}

// Only consequence-bearing requirements are protected. Substitutable stocks
// use the same whole-lot allocation and payment earmarks as consumption.
pub fn protected_stock(world: &World, state: &State) -> Result<BTreeMap<Account, i32>, String> {
    let mut protected = BTreeMap::new();

    for p in &world.participants {
        if world.payment_policy == PaymentPolicy::DebtFirst
            && crate::households::parent(world, state, p.agent).is_none()
        {
            continue;
        }
        if state.terminal.contains_key(&p.agent) {
            continue;
        }
        let opening = crate::substitution::stocks(state, p.agent);
        let mut stocks = opening.clone();
        let earmarks = crate::substitution::earmarks(world, state, p.agent);
        let mut needs: Vec<_> = p.needs.iter().collect();
        needs.sort_by_key(|n| (n.priority, n.resource));
        for need in needs {
            if crate::households::parent(world, state, p.agent).is_none()
                && !world
                    .condition_rules
                    .iter()
                    .any(|r| r.subject == p.agent && r.provision == need.resource)
            {
                continue;
            }
            let remaining =
                i128::from((need.quantity - state.balance(p.agent, need.resource)).max(0));
            let recipes = crate::substitution::recipes(world, need.resource);
            if recipes.len() == 1 {
                // Preserve partial protection when the sole indivisible recipe
                // is currently underfunded, as in the original payment policy.
                let d = recipes[0];
                let input = &d.stages[0].entry_inputs[0];
                let output = i128::from(d.outputs[0].quantity);
                let required = ((remaining + output - 1) / output) * i128::from(input.quantity);
                let stock = stocks.entry(input.resource).or_default();
                *stock -= required.min(*stock);
            } else {
                crate::substitution::allocate(
                    world,
                    need.resource,
                    remaining,
                    &mut stocks,
                    &earmarks,
                );
            }
        }
        for (resource, before) in opening {
            let used = before - stocks.get(&resource).copied().unwrap_or(0);
            if used > 0 {
                protected.insert((p.agent, resource), used as i32);
            }
        }
    }
    Ok(protected)
}

/// Known outgoing stock claims for the candidate window. Paid installments are
/// already reflected in opening stock; offers and speculative income are absent.
/// Overdue outstanding claims count once at the opening boundary, even after
/// their right expires. This is demand pressure, not a settlement prediction.
pub(crate) fn projected_claims(
    world: &World,
    state: &State,
    debtor: AgentId,
    resource: ResourceId,
    horizon: u64,
) -> BTreeMap<u64, i128> {
    let start = u64::from(state.month);
    let end = start + horizon;
    let mut claims = BTreeMap::new();
    for contract in state
        .exchange
        .forwards
        .values()
        .filter(|c| c.debtor == debtor && c.goods.resource == resource)
    {
        let due = u64::from(contract.due).max(start);
        if due < end {
            *claims.entry(due).or_default() +=
                i128::from(contract.goods.quantity - contract.delivered);
        }
    }
    for a in active(world, state).filter(|a| a.debtor == debtor && a.payment.resource == resource) {
        for o in state.obligations.values().filter(|o| o.agreement == a.id) {
            let due = u64::from(o.due).max(start);
            if due < end {
                *claims.entry(due).or_default() += i128::from(o.owed - o.paid);
            }
        }
        let through = u64::from(
            world
                .rights
                .iter()
                .find(|r| r.id == a.right)
                .expect("validated agreement right")
                .through,
        );
        let mut due = u64::from(a.activated) + u64::from(MONTHS_PER_YEAR);
        while due < end && due <= through {
            if !state.obligations.contains_key(&(a.id, due as u32)) {
                *claims.entry(due.max(start)).or_default() += i128::from(a.payment.quantity);
            }
            due += u64::from(MONTHS_PER_YEAR);
        }
    }
    claims
}

/// An open template conveys no right until its agreement is accepted.
pub fn holder(world: &World, state: &State, right: &UseRight) -> Option<AgentId> {
    if crate::credit::follows_owner(world, right.id) {
        return crate::credit::owner(world, state, right.asset);
    }
    if let Some(a) = world
        .access_offers
        .iter()
        .find(|a| a.right == right.id && world.open_access_offers.contains(&a.id))
    {
        state.accepted_agreements.get(&a.id).map(|a| a.debtor)
    } else {
        Some(right.holder)
    }
}
pub fn output_owner(world: &World, state: &State, right: &UseRight) -> Option<AgentId> {
    if crate::credit::follows_owner(world, right.id) {
        return holder(world, state, right);
    }
    if world
        .access_offers
        .iter()
        .any(|a| a.right == right.id && world.open_access_offers.contains(&a.id))
    {
        holder(world, state, right)
    } else {
        Some(right.output_owner)
    }
}

#[cfg(test)]
mod candidate_tests {
    use super::*;
    use crate::scenario::*;

    #[test]
    fn dated_claims_include_boundary_and_subtract_paid_amounts() {
        let (w, mut s) = named("annual-access").unwrap();
        s.month = 7;
        assert!(projected_claims(&w, &s, PERSON, GRAIN, 6).is_empty());
        assert_eq!(
            projected_claims(&w, &s, PERSON, GRAIN, 7),
            BTreeMap::from([(13, 1)])
        );
        s.month = 13;
        s.obligations.insert(
            (1, 13),
            Obligation {
                agreement: 1,
                due: 13,
                owed: 3,
                paid: 2,
                in_kind_paid: 2,
            },
        );
        assert_eq!(
            projected_claims(&w, &s, PERSON, GRAIN, 13),
            BTreeMap::from([(13, 1), (25, 1)])
        );
        s.obligations.get_mut(&(1, 13)).unwrap().paid = 3;
        assert_eq!(
            projected_claims(&w, &s, PERSON, GRAIN, 1)
                .values()
                .sum::<i128>(),
            0
        );
        assert!(projected_claims(&w, &s, STATE_AGENT, GRAIN, 13).is_empty());
        assert!(projected_claims(&w, &s, PERSON, FUEL, 13).is_empty());
    }

    #[test]
    fn expired_right_stops_new_dues_but_preserves_arrears_once() {
        let (mut w, mut s) = named("annual-access").unwrap();
        w.rights[0].through = 13;
        s.month = 26;
        s.obligations.insert(
            (1, 13),
            Obligation {
                agreement: 1,
                due: 13,
                owed: 3,
                paid: 1,
                in_kind_paid: 1,
            },
        );
        assert_eq!(
            projected_claims(&w, &s, PERSON, GRAIN, 24),
            BTreeMap::from([(26, 2)])
        );
    }

    #[test]
    fn unaccepted_offer_has_no_claim_until_actual_activation() {
        let (mut w, mut s) = named("annual-access").unwrap();
        let mut agreement = w.agreements.remove(0);
        // No accepted contract: the right or a posted offer alone cannot owe rent.
        assert!(projected_claims(&w, &s, PERSON, GRAIN, 40).is_empty());
        agreement.activated = 8;
        s.accepted_agreements.insert(agreement.id, agreement);
        assert_eq!(
            projected_claims(&w, &s, PERSON, GRAIN, 20),
            BTreeMap::from([(20, 1)])
        );
    }
}
