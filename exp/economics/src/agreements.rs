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
            Consequence::SuspendNewUse(_) => {
                unreachable!("production terms use a production consequence")
            }
        }
    }
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
