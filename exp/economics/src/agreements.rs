//! Shared accepted-agreement views and consequence evaluation.
//! Domain receipts remain authoritative; derived status is never stored twice.
use crate::{finance, membership::Role, model::*};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Identity {
    Employment(u32),
    Membership {
        member: AgentId,
        organization: AgentId,
        role: Role,
    },
    Land(u32),
    Process(u64),
    Loan(u32),
    Forward(AssetId),
    Guarantee(u32),
    /// One bounded cooperative agreement per start month in the current pilot.
    CooperativeExchange(u32),
    Household(u32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Grant {
    Employment(u32),
    Membership { organization: AgentId, role: Role },
    LandUse(u32),
    ProcessOutput(u64),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Consequence {
    CarryArrears,
    /// Preserve the grant for existing work; unpaid amounts remain collectible.
    SuspendNewUse(Grant),
    /// Abort the process, forfeit its outputs and retain already consumed inputs.
    AbortWithoutRefund,
    /// Accepted enforcement terms, not permission to seize without settlement checks.
    RepossessCollateral {
        asset: AssetId,
        creditor: AgentId,
        grace_months: u32,
        settlement: crate::credit::CollateralSettlement,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaymentTerms {
    pub transfer: finance::Transfer,
    pub first_due: u32,
    pub interval_months: u32,
    pub through: u32,
    pub on_unpaid: Consequence,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Obligation {
    pub claim: finance::Obligation,
    pub on_unpaid: Consequence,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Agreement {
    pub identity: Identity,
    pub grantor: Counterparty,
    pub holder: AgentId,
    pub accepted_month: u32,
    /// Inclusive final month. Expiration ends grants, not outstanding claims.
    pub through: Option<u32>,
    pub grants: Vec<Grant>,
    pub payments: Vec<PaymentTerms>,
    pub obligations: Vec<Obligation>,
    pub production: Option<ProductionCommitment>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Pending,
    Active,
    Restricted,
    Expired,
    Completed,
    Failed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Use {
    Start,
    Continue,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Breach {
    /// Index into this view's obligations, retaining claim and due-date provenance.
    pub obligation: usize,
    pub outstanding: i32,
    pub consequence: Consequence,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Evaluation {
    pub status: Status,
    pub breaches: Vec<Breach>,
    pub production_failure: Option<Consequence>,
}

impl Agreement {
    pub fn evaluate(&self, month: u32) -> Evaluation {
        let accepted = month >= self.accepted_month;
        let breaches: Vec<_> = self
            .obligations
            .iter()
            .enumerate()
            .filter(|(_, o)| o.claim.condition.is_met(month, accepted) && o.claim.outstanding() > 0)
            .map(|(obligation, o)| Breach {
                obligation,
                outstanding: o.claim.outstanding(),
                consequence: o.on_unpaid.clone(),
            })
            .collect();
        let status = if !accepted {
            Status::Pending
        } else if self
            .production
            .as_ref()
            .is_some_and(|p| p.instance.status == crate::model::Status::Aborted)
        {
            Status::Failed
        } else if self
            .production
            .as_ref()
            .is_some_and(|p| p.instance.status == crate::model::Status::Completed)
        {
            Status::Completed
        } else if self.through.is_some_and(|end| month > end) {
            Status::Expired
        } else if !breaches.is_empty() {
            Status::Restricted
        } else {
            Status::Active
        };
        Evaluation {
            status,
            breaches,
            production_failure: (status == Status::Failed).then(|| {
                self.production
                    .as_ref()
                    .unwrap()
                    .terms
                    .on_unfulfilled
                    .clone()
            }),
        }
    }

    pub fn permits(&self, month: u32, grant: &Grant, usage: Use) -> bool {
        let evaluation = self.evaluate(month);
        self.grants.contains(grant)
            && matches!(evaluation.status, Status::Active | Status::Restricted)
            && (usage == Use::Continue || !evaluation.breaches.iter().any(|b|
                matches!(&b.consequence, Consequence::SuspendNewUse(target) if target == grant)))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Counterparty {
    Agent(AgentId),
    Environment,
}

/// Technology terms are shared with execution, not a separate farming recipe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProductionTerms {
    pub definition: DefinitionId,
    pub asset_kind: Option<u32>,
    pub stages: Vec<Stage>,
    pub outputs: Vec<Amount>,
    pub on_unfulfilled: Consequence,
}
impl ProductionTerms {
    pub fn from_definition(d: &ProcessDefinition) -> Self {
        Self {
            definition: d.id,
            asset_kind: d.asset_kind,
            stages: d.stages.clone(),
            outputs: d.outputs.clone(),
            on_unfulfilled: Consequence::AbortWithoutRefund,
        }
    }
    pub fn duration(&self) -> u32 {
        self.stages.iter().map(|s| s.months).sum()
    }
    /// Dated minimum inputs/services; entry inputs occur only at the start of a stage.
    pub fn schedule(&self, start: u32) -> Result<Vec<(u32, Vec<Amount>)>, String> {
        let mut rows = Vec::new();
        let mut month = start;
        for stage in &self.stages {
            for elapsed in 0..stage.months {
                let mut amounts = stage.monthly_services.clone();
                if elapsed == 0 {
                    amounts.extend(stage.entry_inputs.clone());
                }
                rows.push((month, amounts));
                month = month.checked_add(1).ok_or("process schedule overflow")?;
            }
        }
        Ok(rows)
    }
    pub fn fail(&self, instance: &mut ProcessInstance) {
        match self.on_unfulfilled {
            Consequence::AbortWithoutRefund => instance.status = crate::model::Status::Aborted,
            Consequence::CarryArrears
            | Consequence::SuspendNewUse(_)
            | Consequence::RepossessCollateral { .. } => {
                unreachable!("production terms use a production consequence")
            }
        }
    }
}

/// Shared inspection entry point. Loan lifecycle and balances retain their domain
/// meaning instead of being coerced into land-use or process status.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum View<'a> {
    Agreement(Box<Agreement>),
    Loan(LoanView<'a>),
    Forward(&'a crate::forward::Contract),
    Guarantee(GuaranteeView),
    Exchange(Box<ExchangeView>),
    Household(HouseholdView<'a>),
}

impl View<'_> {
    /// Dated collection claims, including not-yet-due claims. These are derived
    /// from the authoritative records; inspecting them never accrues or settles.
    pub fn claims(&self) -> Result<Vec<finance::Obligation>, String> {
        match self {
            Self::Agreement(a) => Ok(a.obligations.iter().map(|o| o.claim.clone()).collect()),
            Self::Loan(a) => Ok(a.claim()?.into_iter().collect()),
            Self::Forward(a) => Ok(vec![a.claim()]),
            Self::Guarantee(a) => Ok(a.call.clone().into_iter().collect()),
            // Conditional delivery-versus-payment is an atomic package. Neither
            // leg is an independent debt collectible through the claim waterfall.
            Self::Exchange(_) | Self::Household(_) => Ok(vec![]),
        }
    }
    pub fn identity(&self) -> Identity {
        match self {
            Self::Agreement(a) => a.identity.clone(),
            Self::Loan(a) => Identity::Loan(a.record.id),
            Self::Forward(a) => Identity::Forward(a.id),
            Self::Guarantee(a) => Identity::Guarantee(a.terms.id),
            Self::Exchange(a) => Identity::CooperativeExchange(a.terms.start),
            Self::Household(a) => Identity::Household(a.terms.id),
        }
    }

    /// Reciprocal exchange has no distinguished grantor or holder.
    pub fn grantor(&self) -> Option<Counterparty> {
        match self {
            Self::Agreement(a) => Some(a.grantor),
            Self::Loan(a) => Some(Counterparty::Agent(a.record.creditor)),
            Self::Forward(a) => Some(Counterparty::Agent(a.creditor)),
            Self::Guarantee(a) => Some(Counterparty::Agent(a.terms.guarantor)),
            Self::Exchange(_) | Self::Household(_) => None,
        }
    }

    pub fn holder(&self) -> Option<AgentId> {
        match self {
            Self::Agreement(a) => Some(a.holder),
            Self::Loan(a) => Some(a.record.debtor),
            Self::Forward(a) => Some(a.debtor),
            Self::Guarantee(a) => Some(a.creditor),
            Self::Exchange(_) | Self::Household(_) => None,
        }
    }

    pub fn parties(&self) -> Vec<AgentId> {
        if let Self::Exchange(a) = self {
            return a.terms.choices.keys().copied().collect();
        }
        if let Self::Household(a) = self {
            let mut parties = a.terms.adults.clone();
            parties.push(a.terms.agent);
            parties.extend(a.terms.membership.iter().filter_map(|c| {
                if c.month > a.month {
                    return None;
                }
                match c.action {
                    crate::households::membership::Action::Join { person, .. } => Some(person),
                    _ => None,
                }
            }));
            parties.sort_unstable();
            parties.dedup();
            return parties;
        }
        let mut parties: Vec<_> = self.holder().into_iter().collect();
        if let Some(Counterparty::Agent(agent)) = self.grantor() {
            parties.push(agent);
        }
        if let Self::Guarantee(g) = self {
            parties.push(g.debtor);
        }
        parties.sort_unstable();
        parties.dedup();
        parties
    }

    pub fn accepted_month(&self) -> u32 {
        match self {
            Self::Agreement(a) => a.accepted_month,
            Self::Loan(a) => a.record.opened,
            Self::Forward(a) => a.issued,
            Self::Guarantee(a) => a.accepted_month,
            Self::Exchange(a) => a.terms.start,
            Self::Household(a) => a.terms.formed,
        }
    }
}

/// Accepted atomic exchanges and their committed outcome, derived from receipts.
/// Cancellation does not manufacture arrears or reverse prior deliveries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExchangeView {
    pub terms: crate::cooperation::Contract,
    pub completed: Vec<crate::cooperation::Delivery>,
    pub status: Status,
    pub failure: Option<String>,
}

/// Founding agreement, current authority and lifecycle remain distinct from
/// member/organization finances. This view never consolidates their claims.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HouseholdView<'a> {
    pub terms: &'a crate::households::Agreement,
    pub month: u32,
    pub roster: Vec<AgentId>,
    pub active_members: Vec<AgentId>,
    pub authority: crate::household_governance::Authority,
    pub status: HouseholdState,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HouseholdState {
    Operating,
    Inactive,
    WindingDown,
    Closed,
}

/// Read-only contingent exposure. A callable guarantee is not extra principal
/// owed by the borrower; payment substitutes a recourse creditor in the loan book.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuaranteeView {
    pub accepted_month: u32,
    pub terms: crate::recovery::Guarantee,
    pub debtor: AgentId,
    pub creditor: AgentId,
    pub paid: i32,
    pub call: Option<finance::Obligation>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoanState {
    Stayed,
    Discharged,
    Current,
    /// Observed arrears from a committed collection attempt, not a forecast.
    Overdue {
        since: u32,
    },
    PendingSale {
        listed: u32,
    },
    Deficiency,
    Repaid,
}

/// Borrowed inspection of one authoritative loan at a specific committed boundary.
/// No independent debt, status or accepted terms are persisted by this adapter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoanView<'a> {
    record: &'a crate::credit::Loan,
    deferred_recourse: i32,
    month: u32,
    phase: Phase,
    title_holder: Option<AgentId>,
    listing: Option<&'a crate::resale::PendingSale>,
    writeoffs: &'a [crate::claim_relief::LoanWriteOff],
}

impl<'a> LoanView<'a> {
    /// Accepted terms and current balances from the book, never the offer catalog.
    pub fn record(&self) -> &'a crate::credit::Loan {
        self.record
    }

    /// Explicit full dispositions, including earlier losses when later guarantee
    /// advances reopened this loan. Estate-wide deficiency closure is a separate
    /// route; an empty slice does not imply that all reductions were repayments.
    pub fn writeoffs(&self) -> &'a [crate::claim_relief::LoanWriteOff] {
        self.writeoffs
    }

    /// Phase is the next boundary to execute. No interest or payment is simulated.
    pub fn boundary(&self) -> (u32, Phase) {
        (self.month, self.phase)
    }

    pub fn title_holder(&self) -> Option<AgentId> {
        self.title_holder
    }

    pub fn state(&self) -> LoanState {
        use crate::credit::Status;
        match self.record.status {
            Status::Active => self
                .record
                .first_unpaid
                .map_or(LoanState::Current, |since| LoanState::Overdue { since }),
            Status::PendingSale => LoanState::PendingSale {
                listed: self.listing.expect("validated pending sale view").listed,
            },
            Status::Stayed => LoanState::Stayed,
            Status::Discharged => LoanState::Discharged,
            Status::Enforced => LoanState::Deficiency,
            Status::Repaid => LoanState::Repaid,
        }
    }

    pub fn outstanding(&self) -> Result<Amount, String> {
        Ok(Amount::new(self.record.denomination, self.record.debt()?))
    }

    /// Current collectible claim shared by debtor and creditor. Pending sale pauses
    /// borrower collection without extinguishing outstanding principal or interest.
    /// This is not a payment reservation; Due still accrues and validates normally.
    pub fn claim(&self) -> Result<Option<finance::Obligation>, String> {
        if matches!(
            self.state(),
            LoanState::PendingSale { .. } | LoanState::Repaid
        ) {
            return Ok(None);
        }
        let mut claim = self.record.claim(self.month)?;
        claim.transfer.amount.quantity = claim
            .transfer
            .amount
            .quantity
            .saturating_sub(self.deferred_recourse);
        Ok((claim.outstanding() > 0).then_some(claim))
    }

    pub fn on_default(&self) -> Option<Consequence> {
        let collateral = self.record.collateral.as_ref()?;
        Some(Consequence::RepossessCollateral {
            asset: collateral.asset,
            creditor: self.record.creditor,
            grace_months: self.record.grace_months,
            settlement: collateral.settlement.clone(),
        })
    }
}

/// Inspect accepted contracts involving this holder or grantor. Catalog offers
/// are excluded. Terminal contracts remain inspectable. Assumes validated state.
/// Domain grouping and stable IDs make output independent of catalog row order.
pub fn for_agent<'a>(
    world: &'a World,
    state: &'a State,
    agent: AgentId,
) -> Result<Vec<View<'a>>, String> {
    let mut views: Vec<_> = state
        .memberships
        .values()
        .map(|a| View::Agreement(Box::new(a.contract())))
        .collect();
    views.extend(
        world
            .employment
            .iter()
            .filter(|t| {
                !world.employment_offers.contains(&t.id)
                    || state.employment.earned.keys().any(|(id, _)| *id == t.id)
            })
            .map(|t| {
                let mut agreement = crate::employment::contract(t, &state.employment);
                if world.employment_offers.contains(&t.id) {
                    let months: Vec<_> = state
                        .employment
                        .earned
                        .keys()
                        .filter(|(id, _)| *id == t.id)
                        .map(|(_, m)| *m)
                        .collect();
                    agreement.accepted_month = *months.iter().min().unwrap();
                    agreement.through = months.iter().max().copied();
                }
                View::Agreement(Box::new(agreement))
            }),
    );
    let mut land: Vec<_> = crate::commitments::active(world, state).collect();
    land.sort_by_key(|a| a.id);
    for a in land {
        views.push(View::Agreement(Box::new(a.contract(world, state)?)));
    }
    views.extend(
        state
            .processes
            .values()
            .map(|p| View::Agreement(Box::new(process(world, p)))),
    );
    for record in state.credit.loans.values() {
        if record.debtor != agent && record.creditor != agent {
            continue;
        }
        let listing = state.credit.pending_sales.get(&record.id);
        if (record.status == crate::credit::Status::PendingSale) != listing.is_some() {
            return Err("loan view has inconsistent pending sale".into());
        }
        views.push(View::Loan(LoanView {
            record,
            deferred_recourse: crate::recovery::current_recourse(
                world,
                &state.credit,
                record.id,
                state.month,
            ),
            month: state.month,
            phase: state.phase,
            title_holder: record
                .collateral
                .as_ref()
                .map(|c| {
                    crate::credit::owner(world, state, c.asset)
                        .ok_or("loan view has missing collateral owner")
                })
                .transpose()?,
            listing,
            writeoffs: state
                .credit
                .recovery
                .loan_writeoffs
                .get(&record.id)
                .map(Vec::as_slice)
                .unwrap_or(&[]),
        }));
    }
    views.extend(state.exchange.forwards.values().map(View::Forward));
    let mut guarantees: Vec<_> = world.recovery.guarantees.iter().collect();
    guarantees.sort_by_key(|g| g.id);
    for g in guarantees {
        let Some(accepted_month) =
            crate::recovery::admission::accepted_month(world, &state.credit, g)
        else {
            continue;
        };
        if let Some((debtor, creditor, _)) = g.claim.current_parties(world, &state.credit) {
            views.push(View::Guarantee(GuaranteeView {
                accepted_month,
                terms: g.clone(),
                debtor,
                creditor,
                paid: state
                    .credit
                    .recovery
                    .paid_guarantees
                    .get(&g.id)
                    .copied()
                    .unwrap_or(0),
                call: crate::recovery::guarantee_claim(world, state, g)?,
            }));
        }
    }
    let mut exchanges = std::collections::BTreeMap::new();
    for round in &state.town_market.history {
        let Some(boundary) = &round.cooperation else {
            continue;
        };
        let Some(terms) = &boundary.terms else {
            continue;
        };
        let view = exchanges
            .entry(terms.start)
            .or_insert_with(|| ExchangeView {
                terms: terms.clone(),
                completed: vec![],
                status: Status::Active,
                failure: None,
            });
        view.completed.extend(boundary.completed.iter().cloned());
        if let Some(reason) = &boundary.failure {
            view.status = Status::Failed;
            view.failure = Some(reason.clone());
        } else if round.month == terms.through {
            view.status = Status::Completed;
        }
    }
    views.extend(exchanges.into_values().map(|v| View::Exchange(Box::new(v))));
    let mut households: Vec<_> = world
        .households
        .iter()
        .filter(|h| h.formed <= state.month)
        .collect();
    households.sort_by_key(|h| h.id);
    for terms in households {
        let authority = crate::household_governance::authority(terms, state);
        let active_members: Vec<_> = crate::households::members(terms, state).collect();
        let status = if crate::households::dissolution::closed_at(terms, state.month) {
            HouseholdState::Closed
        } else if crate::households::dissolution::winding_at(terms, state.month).is_some() {
            HouseholdState::WindingDown
        } else if active_members.is_empty() || authority.leader.is_none() {
            HouseholdState::Inactive
        } else {
            HouseholdState::Operating
        };
        views.push(View::Household(HouseholdView {
            terms,
            month: state.month,
            roster: crate::households::membership::roster_at(terms, state.month),
            active_members,
            authority,
            status,
        }));
    }
    views.retain(|a| a.accepted_month() <= state.month && a.parties().contains(&agent));
    Ok(views)
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProductionCommitment {
    pub terms: ProductionTerms,
    pub instance: ProcessInstance,
}

pub fn process(world: &World, instance: &ProcessInstance) -> Agreement {
    Agreement {
        identity: Identity::Process(instance.id),
        grantor: Counterparty::Environment,
        holder: instance.operator,
        accepted_month: instance.start,
        through: Some(instance.reserved_through),
        grants: vec![Grant::ProcessOutput(instance.id)],
        payments: vec![],
        obligations: vec![],
        production: Some(ProductionCommitment {
            terms: ProductionTerms::from_definition(world.definition(instance.definition)),
            instance: instance.clone(),
        }),
    }
}
