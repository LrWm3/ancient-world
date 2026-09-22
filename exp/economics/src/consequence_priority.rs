//! A scoped ranking adapter over comparable, dated consequence reports.
//! It does not decide eligibility, reserve resources, or change settlement order.
use crate::allocation::{Claim, Context, RankingPolicy};
use std::{cmp::Reverse, collections::BTreeMap};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Existing,
    AvoidHarm,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Harm {
    pub month: u32,
    pub terminal: u64,
    pub impaired: u64,
    pub deprivation: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Assessment {
    pub denied: Vec<Harm>,
    pub accepted: Vec<Harm>,
}
/// Higher severity first (terminal, impairment, deprivation), then earliest
/// avoided harm of that severity, then total reduction in that severity.
/// Economic buffer/work benefits are left to the fallback ranking policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    None,
    Deprivation,
    Impaired,
    Terminal,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Priority {
    pub severity: Reverse<Severity>,
    pub first_month: u32,
    pub magnitude: Reverse<u64>,
}
impl Assessment {
    pub fn priority(&self) -> Result<Priority, String> {
        if self.denied.len() != self.accepted.len()
            || self.denied.windows(2).any(|w| w[0].month >= w[1].month)
            || self
                .denied
                .iter()
                .zip(&self.accepted)
                .any(|(a, b)| a.month != b.month)
        {
            return Err("consequence reports require identical ordered months".into());
        }
        for severity in [
            Severity::Terminal,
            Severity::Impaired,
            Severity::Deprivation,
        ] {
            let value = |h: &Harm| match severity {
                Severity::Terminal => h.terminal,
                Severity::Impaired => h.impaired,
                _ => h.deprivation,
            };
            let mut first = u32::MAX;
            let mut net = 0_i128;
            for (denied, accepted) in self.denied.iter().zip(&self.accepted) {
                let delta = i128::from(value(denied)) - i128::from(value(accepted));
                net += delta;
                if delta > 0 {
                    first = first.min(denied.month);
                }
            }
            if net < 0 {
                // A higher-severity worsening cannot be outweighed by lower
                // severity benefits. Leave this claim to ordinary ranking.
                break;
            }
            if net > 0 {
                return Ok(Priority {
                    severity: Reverse(severity),
                    first_month: first,
                    magnitude: Reverse(
                        u64::try_from(net).map_err(|_| "consequence magnitude overflow")?,
                    ),
                });
            }
        }
        Ok(Priority {
            severity: Reverse(Severity::None),
            first_month: u32::MAX,
            magnitude: Reverse(0),
        })
    }
}
pub struct Ranking {
    ranks: BTreeMap<u64, u32>,
}
impl Ranking {
    pub fn new(
        context: Context,
        fallback: &dyn RankingPolicy,
        claims: &[Claim],
        reports: &BTreeMap<u64, Assessment>,
    ) -> Result<Self, String> {
        let mut ordered = Vec::new();
        for c in claims {
            let priority = reports
                .get(&c.id)
                .ok_or("missing consequence report")?
                .priority()?;
            ordered.push((priority, fallback.key(context, c), c.id));
        }
        ordered.sort();
        let mut ranks = BTreeMap::new();
        for (rank, (_, _, id)) in ordered.into_iter().enumerate() {
            if ranks
                .insert(
                    id,
                    u32::try_from(rank).map_err(|_| "too many ranked claims")?,
                )
                .is_some()
            {
                return Err("duplicate consequence claim".into());
            }
        }
        Ok(Self { ranks })
    }
}
impl RankingPolicy for Ranking {
    fn key(&self, _context: Context, claim: &Claim) -> (u32, u64) {
        (self.ranks[&claim.id], claim.id)
    }
}
