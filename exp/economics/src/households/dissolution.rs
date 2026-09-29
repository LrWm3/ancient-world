//! Opt-in solvent wind-down. Debt servicing retains its existing execution path;
//! residual stock is released only after claims, assets and commitments are cleared.
use super::*;
use membership::Action;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub enum Blocker {
    Loan,
    LandAgreement,
    Forward,
    Guarantee,
    Recovery,
    Employment,
    Asset,
    Right,
    Process,
    StandingExchange,
    NonStockBalance,
    RecipientUnavailable,
    Storage,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub household: AgentId,
    pub recipient: AgentId,
    pub blockers: Vec<Blocker>,
    pub distributed: BTreeMap<ResourceId, i32>,
}

pub fn winding_at(a: &Agreement, month: u32) -> Option<(u32, AgentId, AgentId)> {
    a.membership.iter().find_map(|c| match c.action {
        Action::WindDown { person, recipient } if c.month <= month => {
            Some((c.month, person, recipient))
        }
        _ => None,
    })
}
pub fn closed_at(a: &Agreement, month: u32) -> bool {
    a.membership
        .iter()
        .any(|c| c.month <= month && matches!(c.action, Action::Dissolve { .. }))
}

/// The last member accepts the founding dissolution terms; no governor is needed
/// to exercise this reserved exit right. Property and membership persist for now.
pub fn request(
    world: &mut World,
    state: &State,
    household: AgentId,
    person: AgentId,
) -> Result<(), String> {
    let a = world
        .households
        .iter()
        .find(|a| a.agent == household)
        .ok_or("unknown household")?;
    membership::boundary(a, state)?;
    if !a.governance.constitution.allow_dissolution
        || members(a, state).collect::<Vec<_>>() != vec![person]
    {
        return Err(
            "wind-down requires the last living member and constitutional permission".into(),
        );
    }
    let recipient = a.governance.charter.residual_recipient.unwrap_or(person);
    membership::accept(
        world,
        state,
        household,
        Action::WindDown { person, recipient },
    )
}

/// Release the final affiliation and storage only after a completed wind-down
/// month. No balances, contracts, assets or reporting entities are deleted.
pub fn finish(
    world: &mut World,
    state: &State,
    household: AgentId,
    person: AgentId,
) -> Result<(), String> {
    let a = world
        .households
        .iter()
        .find(|a| a.agent == household)
        .ok_or("unknown household")?;
    membership::boundary(a, state)?;
    if winding_at(a, state.month).is_none_or(|(_, p, _)| p != person)
        || closed_at(a, state.month)
        || state.terminal.contains_key(&person)
        || !blockers(world, state, household).is_empty()
        || state
            .balances
            .iter()
            .any(|((id, _), q)| *id == household && *q != 0)
    {
        return Err(
            "household dissolution requires a living last member and cleared property/claims"
                .into(),
        );
    }
    membership::accept(world, state, household, Action::Dissolve { person })
}

/// Conservative clearance, including receivables and future performance. Merely
/// having no currently due bill is insufficient grounds to give away all assets.
pub fn blockers(w: &World, s: &State, household: AgentId) -> Vec<Blocker> {
    let involved = |a, b| a == household || b == household;
    let mut result = vec![];
    if s.credit.loans.values().any(|l| {
        involved(l.debtor, l.creditor)
            && (l.principal > 0 || l.interest > 0 || l.status == crate::credit::Status::PendingSale)
    }) || w.lending.iter().any(|l| {
        involved(l.debtor, l.terms.creditor)
            && l.month >= s.month
            && !s.credit.loans.contains_key(&l.id)
    }) {
        result.push(Blocker::Loan);
    }
    if crate::commitments::active(w, s).any(|a| {
        involved(a.debtor, a.creditor)
            && (w
                .rights
                .iter()
                .any(|r| r.id == a.right && r.through >= s.month)
                || s.obligations
                    .values()
                    .any(|o| o.agreement == a.id && o.paid < o.owed))
    }) {
        result.push(Blocker::LandAgreement);
    }
    if s.exchange
        .forwards
        .values()
        .any(|c| involved(c.debtor, c.creditor) && c.claim().outstanding() > 0)
    {
        result.push(Blocker::Forward);
    }
    if w.recovery
        .guarantees
        .iter()
        .any(|g| g.guarantor == household && g.through >= s.month)
    {
        result.push(Blocker::Guarantee);
    }
    if w.recovery.proceedings.iter().any(|p| {
        involved(p.debtor, p.estate)
            && s.credit
                .recovery
                .proceedings
                .get(&p.id)
                .is_none_or(|p| p.stage != crate::recovery::Stage::Closed)
    }) {
        result.push(Blocker::Recovery);
    }
    if w.employment.iter().any(|t| {
        involved(t.employer, t.worker)
            && (t.through >= s.month
                || s.employment
                    .earned
                    .iter()
                    .any(|((id, _), e)| *id == t.id && e.claim.outstanding() > 0))
    }) {
        result.push(Blocker::Employment);
    }
    if w.assets
        .iter()
        .any(|a| crate::credit::owner(w, s, a.id) == Some(household))
        || s.equipment.values().any(|a| a.owner == household)
    {
        result.push(Blocker::Asset);
    }
    if w.rights
        .iter()
        .any(|r| involved(r.holder, r.output_owner) && r.through >= s.month)
    {
        result.push(Blocker::Right);
    }
    if s.processes
        .values()
        .any(|p| involved(p.operator, p.beneficiary) && p.status == Status::Active)
    {
        result.push(Blocker::Process);
    }
    // These configured exchange roles require an explicit cancellation/novation
    // adapter before this stock-only wind-down can declare them finished.
    if w.households.iter().any(|h| {
        h.asset_sales
            .iter()
            .any(|sale| sale.month >= s.month && involved(h.agent, sale.buyer))
    }) || w.marketplaces.iter().any(|m| m.agent == household)
        || w.transaction_policy
            .as_ref()
            .is_some_and(|p| p.authority == household)
        || w.bids.iter().any(|b| b.buyer == household)
        || w.access_offers
            .iter()
            .any(|a| involved(a.debtor, a.creditor))
        || w.offers
            .iter()
            .any(|o| o.seller == household && !s.filled_offers.contains(&o.id))
        || s.exchange
            .contracts
            .values()
            .any(|c| involved(c.buyer, c.provider))
        || w.market.as_ref().is_some_and(|m| {
            m.tools.iter().any(|t| involved(t.buyer, t.provider))
                || m.cash.as_ref().is_some_and(|c| c.lender == household)
                || m.reserves.keys().any(|(id, _)| *id == household)
        })
        || w.credit.as_ref().is_some_and(|c| {
            c.application.buyer == household
                || c.offers
                    .iter()
                    .any(|o| involved(o.sale.seller, o.loan.creditor))
                || c.transfers
                    .iter()
                    .any(|t| t.month >= s.month && involved(t.transfer.from, t.transfer.to))
        })
    {
        result.push(Blocker::StandingExchange);
    }
    if s.balances.iter().any(|((id, r), q)| {
        *id == household
            && *q != 0
            && (w.activities.perishable.contains(r)
                || !w
                    .resources
                    .iter()
                    .any(|x| x.id == *r && x.kind == ResourceKind::Stock))
    }) {
        result.push(Blocker::NonStockBalance);
    }
    result
}

pub(super) fn prepare(world: &World, state: &State) -> Result<(Vec<Effect>, Vec<Receipt>), String> {
    let mut effects = vec![];
    let mut receipts = vec![];
    let mut staged = state.clone();
    let mut households: Vec<_> = world.households.iter().collect();
    households.sort_by_key(|a| a.agent);
    for a in households {
        let Some((_, person, recipient)) = winding_at(a, state.month) else {
            continue;
        };
        if closed_at(a, state.month) {
            continue;
        }
        let mut receipt = Receipt {
            household: a.agent,
            recipient,
            blockers: blockers(world, &staged, a.agent),
            distributed: BTreeMap::new(),
        };
        if state.terminal.contains_key(&person)
            || state.terminal.contains_key(&recipient)
            || world
                .households
                .iter()
                .any(|h| h.agent == recipient && winding_at(h, state.month).is_some())
        {
            receipt.blockers.push(Blocker::RecipientUnavailable);
        }
        if receipt.blockers.is_empty() {
            let transfers: Vec<_> = staged
                .balances
                .iter()
                .filter(|((id, _), q)| *id == a.agent && **q > 0)
                .flat_map(|((_, resource), q)| transfer(a.agent, recipient, *resource, *q))
                .collect();
            // Also fit the destination after all of this household's contributed
            // space is released; the next Open must not trap an empty household.
            let mut released = world.clone();
            released
                .households
                .iter_mut()
                .find(|h| h.agent == a.agent)
                .unwrap()
                .membership
                .push(membership::Change {
                    month: state.month,
                    action: Action::Dissolve { person },
                });
            if !crate::storage::fits(
                &released,
                &crate::storage::usage(world, &staged.balances),
                &transfers,
            ) {
                receipt.blockers.push(Blocker::Storage);
            } else {
                for e in &transfers {
                    if e.delta < 0 {
                        receipt.distributed.insert(e.account.1, -e.delta);
                    }
                }
                apply(world, &mut staged, &transfers, Backend::Reference)?;
                effects.extend(transfers);
            }
        }
        receipts.push(receipt);
    }
    Ok((effects, receipts))
}

pub(super) fn validate(world: &World, state: &State) -> Result<(), String> {
    for a in &world.households {
        if closed_at(a, state.month)
            && (!blockers(world, state, a.agent).is_empty()
                || state
                    .balances
                    .iter()
                    .any(|((id, _), q)| *id == a.agent && *q != 0))
        {
            return Err("dissolved household retains or acquired unsettled property/claims".into());
        }
    }
    Ok(())
}
