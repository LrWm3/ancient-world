//! Static legal constraints under the transaction policy's single authority.
//! Laws constrain grants; they do not supply rights, resources, or feasibility.
use crate::{
    model::*,
    opportunities::{Action, AgentType, Policy},
};

/// Recognized shapes of newly entered agreements, not permissions to act.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AgreementForm {
    LandUseLease,
    FinancedAssetPurchase,
}
impl AgreementForm {
    fn action(self) -> Action {
        match self {
            Self::LandUseLease => Action::LandAccess,
            Self::FinancedAssetPurchase => Action::FinancedPurchase,
        }
    }
}
/// An absent catalog preserves older scenarios. An explicit empty catalog
/// recognizes neither of the supported forms.
pub fn recognizes(w: &World, form: AgreementForm) -> bool {
    w.transaction_policy.as_ref().is_none_or(|p| {
        p.agreement_forms
            .as_ref()
            .is_none_or(|forms| forms.contains(&form))
    })
}
/// New agreement entry only. Existing rights, servicing and enforcement do not
/// call this check; withdrawing recognition does not retroactively void contracts.
pub fn evaluate_agreement(w: &World, s: &State, agent: AgentId, form: AgreementForm) -> Decision {
    let mut decision = evaluate(w, s, agent, form.action());
    if !recognizes(w, form) {
        decision.allowed = false;
        decision
            .reasons
            .retain(|r| !matches!(r, Reason::Granted | Reason::UnrestrictedLegacy));
        decision.reasons.push(Reason::UnrecognizedForm { form });
    }
    decision
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Requirement {
    Prohibited,
    Membership(crate::membership::Role),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    pub id: u32,
    pub name: String,
    /// None applies to every classified type.
    pub agent_type: Option<AgentType>,
    pub action: Action,
    pub requirement: Requirement,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reason {
    UnrestrictedLegacy,
    Granted,
    Unclassified,
    NoGrant,
    UnrecognizedForm {
        form: AgreementForm,
    },
    Prohibited {
        rule: u32,
    },
    MembershipRequired {
        rule: u32,
        role: crate::membership::Role,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decision {
    pub authority: Option<AgentId>,
    pub allowed: bool,
    /// Sorted by rule ID, independent of catalog order.
    pub reasons: Vec<Reason>,
}
pub(crate) fn member(p: &Policy, s: &State, agent: AgentId, role: crate::membership::Role) -> bool {
    s.memberships.values().any(|m| {
        m.member == agent
            && m.role == role
            && m.contract().permits(
                s.month,
                &crate::agreements::Grant::Membership {
                    organization: p.authority,
                    role,
                },
                crate::agreements::Use::Start,
            )
    })
}
pub fn evaluate(w: &World, s: &State, agent: AgentId, action: Action) -> Decision {
    let Some(p) = &w.transaction_policy else {
        return Decision {
            authority: None,
            allowed: true,
            reasons: vec![Reason::UnrestrictedLegacy],
        };
    };
    let Some(kind) = p.agent_types.get(&agent) else {
        return Decision {
            authority: Some(p.authority),
            allowed: false,
            reasons: vec![Reason::Unclassified],
        };
    };
    let granted = p.permissions.contains(&(*kind, action))
        || p.membership_permissions
            .iter()
            .any(|(role, a)| *a == action && member(p, s, agent, *role));
    let mut reasons = vec![];
    if !granted {
        reasons.push(Reason::NoGrant);
    }
    let mut rules: Vec<_> = p
        .laws
        .iter()
        .filter(|r| r.action == action && r.agent_type.is_none_or(|t| t == *kind))
        .collect();
    rules.sort_by_key(|r| r.id);
    for r in rules {
        match r.requirement {
            Requirement::Prohibited => reasons.push(Reason::Prohibited { rule: r.id }),
            Requirement::Membership(role) if !member(p, s, agent, role) => {
                reasons.push(Reason::MembershipRequired { rule: r.id, role })
            }
            Requirement::Membership(_) => {}
        }
    }
    let allowed = reasons.is_empty();
    if allowed {
        reasons.push(Reason::Granted);
    }
    Decision {
        authority: Some(p.authority),
        allowed,
        reasons,
    }
}
pub(crate) fn validate(w: &World, p: &Policy) -> Result<(), String> {
    let mut ids = std::collections::BTreeSet::new();
    for r in &p.laws {
        if !ids.insert(r.id)
            || r.name.trim().is_empty()
            || matches!(r.action, Action::Process(id) if !w.definitions.iter().any(|d| d.id == id))
        {
            return Err("invalid legal rule: duplicate ID, empty name or unknown process".into());
        }
    }
    Ok(())
}
