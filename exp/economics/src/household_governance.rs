//! Founding templates, static charter parameters and bounded household policy.
use crate::{households::Agreement, model::*};
use std::collections::BTreeSet;
pub mod elections;
pub mod scenario;

pub const PERCENT: i32 = 100;
pub const DEFAULT_LABOR_PERCENT: u32 = 20;
pub const DEFAULT_TERM_MONTHS: u32 = 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Policy {
    NetOutput,
    NeedsFirst,
    /// Current needs, then expected collective cash at the next town book.
    NeedsThenIncome,
    /// Current needs, then accepted collective claim coverage, then net output.
    NeedsThenCommitments {
        months: u32,
    },
    PreserveCommittedWork,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TieBreak {
    MemberId,
    Rotating,
    SignatoryOrder,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Contribution {
    Percent(u32),
    SpareLabor,
}
/// Static founding choice: one authority generates consumption bids per member.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Purchasing {
    Collective,
    Members,
}
pub use crate::governance::Leadership;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Constitution {
    /// Opt-in solvent wind-down under the static residual-recipient charter.
    pub allow_dissolution: bool,
    pub leadership: Leadership,
    pub permitted_policies: BTreeSet<Policy>,
    pub permitted_ties: BTreeSet<TieBreak>,
    /// None permits member-executable productive activities. A subset can narrow
    /// the mandate, but never grants a worker another member's personal rights.
    pub activities: Option<BTreeSet<DefinitionId>>,
}
/// Allocation among member support requests, distinct from creditor collection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DebtSupportPolicy {
    #[default]
    ReservationOrder,
    /// Lowest covered claim rank first, then stable member ID.
    ClaimPriority,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Charter {
    pub purchasing: Purchasing,
    /// Collective buying may cover missing entry inputs of active member processes.
    pub fund_committed_inputs: bool,
    /// Pay current member loan dues from collective stock at Due; no debt assumption.
    pub support_member_loans: bool,
    /// Transfer current earned-wage shortfalls to member employers before Close payroll.
    pub support_member_wages: bool,
    /// Member-authorized surplus may also fill own earned-wage/current-loan/land-dues gaps
    /// when it does not improve consumption or market income. No debt assumption.
    pub accept_payment_support: bool,
    /// Collective bids may acquire denomination stock for current collectible loans.
    pub fund_due_loans: bool,
    /// Collective bids may acquire stock for the household's own current land bills.
    pub fund_land_dues: bool,
    /// Collective bids may acquire goods for own current prepaid deliveries.
    pub fund_forward_deliveries: bool,
    /// Preferred accepted payment route, shared by collection and funding.
    pub land_tender: crate::commitments::TenderPreference,
    /// Collective buying may fund own wages and explicitly supported member wage claims.
    pub fund_earned_wages: bool,
    /// Optional current-delivery target when collective wage funding is enabled.
    /// Forecasts never authorize member assistance or create wage liabilities.
    pub payroll_outlook: crate::employment::PayrollOutlook,
    pub debt_support: DebtSupportPolicy,
    /// Maximum earned wages for new hired hours per month; None disables hiring.
    /// Delivery also requires opening funds after other boundary reservations.
    pub hiring_budget: Option<Amount>,
    /// Optional static target for projected cash after the next town book.
    /// Above this buffer NeedsThenIncome does not request extra income work.
    pub cash_target: Option<Amount>,
    /// None selects the last living member; otherwise a named residual recipient.
    pub residual_recipient: Option<AgentId>,
    /// Founding governor; rotating terms start here in the stable adult-ID ring.
    pub leader: AgentId,
    pub term_months: u32,
    pub election: elections::Rules,
    pub contribution: Contribution,
    pub tie_break: TieBreak,
    pub initial_policy: Policy,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyChange {
    pub month: u32,
    pub authorized_by: AgentId,
    pub policy: Policy,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcceptedPolicy {
    pub issued_month: u32,
    pub change: PolicyChange,
    /// None preserves the previously effective allocation tie-break.
    pub tie_break: Option<TieBreak>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Authority {
    pub household: AgentId,
    pub leader: Option<AgentId>,
    pub leadership: Leadership,
    pub term_start: u32,
    pub election: Option<elections::ElectionResult>,
    pub tie_break: TieBreak,
    pub policy: Policy,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Governance {
    pub constitution: Constitution,
    pub charter: Charter,
    /// Accepted dated instructions; leader chooses policy, not individual jobs.
    pub changes: Vec<AcceptedPolicy>,
    pub ballots: Vec<elections::AcceptedBallot>,
}
impl Governance {
    pub fn contributed(leader: AgentId) -> Self {
        Self {
            constitution: Constitution {
                allow_dissolution: false,
                leadership: Leadership::FixedFounder,
                permitted_policies: [
                    Policy::NetOutput,
                    Policy::PreserveCommittedWork,
                    Policy::NeedsFirst,
                ]
                .into_iter()
                .collect(),
                permitted_ties: [
                    TieBreak::MemberId,
                    TieBreak::Rotating,
                    TieBreak::SignatoryOrder,
                ]
                .into(),
                activities: None,
            },
            charter: Charter {
                purchasing: Purchasing::Collective,
                fund_committed_inputs: false,
                support_member_loans: false,
                support_member_wages: false,
                accept_payment_support: false,
                fund_due_loans: false,
                fund_land_dues: false,
                fund_forward_deliveries: false,
                land_tender: crate::commitments::TenderPreference::default(),
                fund_earned_wages: false,
                payroll_outlook: crate::employment::PayrollOutlook::default(),
                debt_support: DebtSupportPolicy::default(),
                hiring_budget: None,
                cash_target: None,
                residual_recipient: None,
                leader,
                term_months: DEFAULT_TERM_MONTHS,
                election: elections::Rules::default(),
                contribution: Contribution::Percent(DEFAULT_LABOR_PERCENT),
                tie_break: TieBreak::Rotating,
                initial_policy: Policy::PreserveCommittedWork,
            },
            changes: vec![],
            ballots: vec![],
        }
    }
    pub fn rotating(leader: AgentId, term_months: u32) -> Self {
        let mut g = Self::contributed(leader);
        g.constitution.leadership = Leadership::Rotating;
        g.charter.term_months = term_months;
        g
    }
    pub fn elected(leader: AgentId, term_months: u32) -> Self {
        let mut g = Self::rotating(leader, term_months);
        g.constitution.leadership = Leadership::Elected;
        g
    }
    pub fn legacy(leader: AgentId) -> Self {
        let mut g = Self::contributed(leader);
        g.charter.contribution = Contribution::SpareLabor;
        g.charter.tie_break = TieBreak::SignatoryOrder;
        g.charter.initial_policy = Policy::NetOutput;
        g
    }
    pub fn tie_break(&self, month: u32) -> TieBreak {
        self.changes
            .iter()
            .filter(|c| c.issued_month <= month && c.change.month <= month && c.tie_break.is_some())
            .max_by_key(|c| (c.change.month, c.issued_month))
            .and_then(|c| c.tie_break)
            .unwrap_or(self.charter.tie_break)
    }
    pub fn policy(&self, month: u32) -> Policy {
        self.changes
            .iter()
            .filter(|c| c.issued_month <= month && c.change.month <= month)
            .max_by_key(|c| (c.change.month, c.issued_month))
            .map_or(self.charter.initial_policy, |c| c.change.policy)
    }
}

pub fn validate(world: &World, state: &State, a: &Agreement) -> Result<(), String> {
    let g = &a.governance;
    if g.constitution.permitted_policies.iter().any(|p| {
        matches!(p, Policy::NeedsThenCommitments { months }
            if !(1..=crate::need_orders::MAX_RESERVE_MONTHS).contains(months)
                || !matches!(g.charter.contribution, Contribution::Percent(_)))
    }) {
        return Err("commitment policy requires a bounded horizon and percentage labor".into());
    }
    if (g.charter.initial_policy == Policy::NeedsThenIncome
        || g.changes
            .iter()
            .any(|c| c.change.policy == Policy::NeedsThenIncome))
        && (world.town_market.is_none()
            || !matches!(g.charter.contribution, Contribution::Percent(_)))
    {
        return Err("household income policy requires contributed labor and a town book".into());
    }
    if let Some(target) = &g.charter.cash_target {
        let payment = world.town_market.as_ref().and_then(|c| {
            crate::marketplace::venue(world, c.venue)
                .and_then(|v| v.markets.iter().find(|m| m.id == c.market))
                .map(|m| m.payment)
        });
        if target.quantity < 0 || payment != Some(target.resource) {
            return Err("household cash target requires nonnegative town-payment units".into());
        }
    }
    if let Some(budget) = &g.charter.hiring_budget
        && (budget.quantity < 0
            || !world
                .resources
                .iter()
                .any(|r| r.id == budget.resource && r.kind == ResourceKind::Stock)
            || !matches!(g.charter.contribution, Contribution::Percent(_)))
    {
        return Err("household hiring requires a stock budget and percentage labor charter".into());
    }
    elections::validate(a, state)?;
    let mut dates = BTreeSet::new();
    if g.charter
        .residual_recipient
        .is_some_and(|id| id == a.agent || !world.agents.iter().any(|a| a.id == id))
        || !g.constitution.permitted_ties.contains(&g.charter.tie_break)
        || g.charter.term_months == 0
        || !a.adults.contains(&g.charter.leader)
        || !g
            .constitution
            .permitted_policies
            .contains(&g.charter.initial_policy)
        || matches!(g.charter.contribution, Contribution::Percent(p) if p > PERCENT as u32)
        || g.constitution.activities.as_ref().is_some_and(|ids| {
            ids.iter().any(|id| {
                !world
                    .definitions
                    .iter()
                    .any(|d| d.id == *id && d.execution == Execution::Productive)
            })
        })
        || g.changes.iter().any(|c| {
            c.issued_month < a.formed
                || c.issued_month > state.month
                || c.change.month <= c.issued_month
                || !dates.insert((c.issued_month, c.change.month))
                || leader_at_open(a, state, c.issued_month) != Some(c.change.authorized_by)
                || !g.constitution.permitted_policies.contains(&c.change.policy)
                || c.tie_break
                    .is_some_and(|tie| !g.constitution.permitted_ties.contains(&tie))
        })
    {
        return Err("invalid household constitution, charter or policy authority".into());
    }
    // Legacy is a compatibility policy, not an alternative executor for a limited mandate.
    if g.charter.contribution == Contribution::SpareLabor
        && (g.constitution.activities.is_some()
            || g.charter.initial_policy != Policy::NetOutput
            || !g.changes.is_empty())
    {
        return Err("legacy spare labor requires its original unrestricted output policy".into());
    }
    Ok(())
}

/// Governance changes only at month opening. A terminal transition at Close m
/// affects selection from Open m+1, preserving earlier authority evidence.
pub(crate) fn leader_at_open(a: &Agreement, state: &State, month: u32) -> Option<AgentId> {
    let g = &a.governance;
    if month < a.formed
        || g.charter.term_months == 0
        || crate::households::dissolution::winding_at(a, month).is_some()
    {
        return None;
    }
    let roster = crate::households::membership::roster_at(a, month);
    let alive = |id: &AgentId| {
        roster.contains(id) && state.terminal.get(id).is_none_or(|t| t.month >= month)
    };
    if g.constitution.leadership == Leadership::FixedFounder {
        return alive(&g.charter.leader).then_some(g.charter.leader);
    }
    if g.constitution.leadership == Leadership::Elected {
        let term_start =
            a.formed + ((month - a.formed) / g.charter.term_months) * g.charter.term_months;
        let winner = if term_start == a.formed {
            Some(g.charter.leader)
        } else {
            elections::resolve(a, state, term_start).winner
        };
        return winner.filter(alive);
    }
    let term_start =
        a.formed + ((month - a.formed) / g.charter.term_months) * g.charter.term_months;
    // Retain founder as the calendar anchor; entrants wait until the next term.
    let mut ring = crate::households::membership::roster_at(a, term_start);
    if !ring.contains(&g.charter.leader) {
        ring.push(g.charter.leader);
    }
    ring.sort_unstable();
    let initial = ring.iter().position(|id| *id == g.charter.leader)?;
    let term = (month - a.formed) / g.charter.term_months;
    let start = (initial + term as usize % ring.len()) % ring.len();
    (0..ring.len())
        .map(|offset| ring[(start + offset) % ring.len()])
        .find(alive)
}
/// Living current authority; a death does not confer same-month replacement powers.
pub fn leader(a: &Agreement, state: &State) -> Option<AgentId> {
    leader_at_open(a, state, state.month).filter(|id| !state.terminal.contains_key(id))
}
pub fn authority(a: &Agreement, state: &State) -> Authority {
    let g = &a.governance;
    let term_start = if g.constitution.leadership != Leadership::FixedFounder
        && g.charter.term_months > 0
    {
        a.formed
            + (state.month.saturating_sub(a.formed) / g.charter.term_months) * g.charter.term_months
    } else {
        a.formed
    };
    Authority {
        household: a.agent,
        leader: leader(a, state),
        leadership: g.constitution.leadership,
        term_start,
        election: (g.constitution.leadership == Leadership::Elected && term_start > a.formed)
            .then(|| elections::resolve(a, state, term_start)),
        policy: g.policy(state.month),
        tie_break: g.tie_break(state.month),
    }
}

pub fn ordered(a: &Agreement, state: &State) -> Vec<AgentId> {
    let mut people: Vec<_> = crate::households::members(a, state).collect();
    match a.governance.tie_break(state.month) {
        TieBreak::SignatoryOrder => {}
        TieBreak::MemberId => people.sort_unstable(),
        TieBreak::Rotating => {
            people.sort_unstable();
            if !people.is_empty() {
                let offset = (state.month - a.formed) as usize % people.len();
                people.rotate_left(offset);
            }
        }
    }
    people
}

/// Accept a policy instruction now for a future month. Founding documents stay fixed.
pub fn schedule(
    world: &mut World,
    state: &State,
    household: AgentId,
    change: PolicyChange,
) -> Result<(), String> {
    schedule_allocation(world, state, household, change, None)
}

/// An atomic objective/tie instruction; charter defaults and constitutional choices stay fixed.
pub fn schedule_allocation(
    world: &mut World,
    state: &State,
    household: AgentId,
    change: PolicyChange,
    tie_break: Option<TieBreak>,
) -> Result<(), String> {
    let mut candidate = world.clone();
    let a = candidate
        .households
        .iter_mut()
        .find(|a| a.agent == household)
        .ok_or("unknown household")?;
    if change.month <= state.month || leader(a, state) != Some(change.authorized_by) {
        return Err("policy instruction requires a living governor and a future month".into());
    }
    a.governance.changes.push(AcceptedPolicy {
        issued_month: state.month,
        change,
        tie_break,
    });
    crate::households::validate(&candidate, state)?;
    *world = candidate;
    Ok(())
}
