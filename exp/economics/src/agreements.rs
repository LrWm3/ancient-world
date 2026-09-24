//! Shared accepted-agreement views and consequence evaluation.
//! Domain receipts remain authoritative; derived status is never stored twice.
use crate::{finance, membership::Role, model::*};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Identity {
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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Grant {
    Membership { organization: AgentId, role: Role },
    LandUse(u32),
    ProcessOutput(u64),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Consequence {
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
            Consequence::SuspendNewUse(_) | Consequence::RepossessCollateral { .. } => {
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
        }
    }
    pub fn identity(&self) -> Identity {
        match self {
            Self::Agreement(a) => a.identity.clone(),
            Self::Loan(a) => Identity::Loan(a.record.id),
            Self::Forward(a) => Identity::Forward(a.id),
            Self::Guarantee(a) => Identity::Guarantee(a.terms.id),
        }
    }

    pub fn grantor(&self) -> Counterparty {
        match self {
            Self::Agreement(a) => a.grantor,
            Self::Loan(a) => Counterparty::Agent(a.record.creditor),
            Self::Forward(a) => Counterparty::Agent(a.creditor),
            Self::Guarantee(a) => Counterparty::Agent(a.terms.guarantor),
        }
    }

    pub fn holder(&self) -> AgentId {
        match self {
            Self::Agreement(a) => a.holder,
            Self::Loan(a) => a.record.debtor,
            Self::Forward(a) => a.debtor,
            Self::Guarantee(a) => a.creditor,
        }
    }

    pub fn accepted_month(&self) -> u32 {
        match self {
            Self::Agreement(a) => a.accepted_month,
            Self::Loan(a) => a.record.opened,
            Self::Forward(a) => a.issued,
            Self::Guarantee(a) => a.terms.from,
        }
    }
}

/// Read-only contingent exposure. A callable guarantee is not extra principal
/// owed by the borrower; payment substitutes a recourse creditor in the loan book.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuaranteeView {
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
    month: u32,
    phase: Phase,
    title_holder: Option<AgentId>,
    listing: Option<&'a crate::resale::PendingSale>,
}

impl<'a> LoanView<'a> {
    /// Accepted terms and current balances from the book, never the offer catalog.
    pub fn record(&self) -> &'a crate::credit::Loan {
        self.record
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
        let claim = self.record.claim(self.month)?;
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
    world: &World,
    state: &'a State,
    agent: AgentId,
) -> Result<Vec<View<'a>>, String> {
    let mut views: Vec<_> = state
        .memberships
        .values()
        .map(|a| View::Agreement(Box::new(a.contract())))
        .collect();
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
        }));
    }
    views.extend(state.exchange.forwards.values().map(View::Forward));
    let mut guarantees: Vec<_> = world.recovery.guarantees.iter().collect();
    guarantees.sort_by_key(|g| g.id);
    for g in guarantees {
        if let Some(loan) = state.credit.loans.get(&g.loan) {
            views.push(View::Guarantee(GuaranteeView {
                terms: g.clone(),
                debtor: loan.debtor,
                creditor: loan.creditor,
                paid: state
                    .credit
                    .recovery
                    .paid_guarantees
                    .get(&g.id)
                    .copied()
                    .unwrap_or(0),
                call: crate::recovery::guarantee_claim(world, &state.credit, state.month, g)?,
            }));
        }
    }
    views.retain(|a| {
        a.accepted_month() <= state.month
            && (a.holder() == agent
                || a.grantor() == Counterparty::Agent(agent)
                || matches!(a, View::Guarantee(g) if g.debtor == agent))
    });
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
