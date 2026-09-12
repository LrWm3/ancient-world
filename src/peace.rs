//! Bilateral peace offers and finite, dated council payment obligations.
use crate::{civilization::History, governance::Treaty};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Payment {
    pub month: u32,
    pub due: f64,
    pub paid: f64,
    pub arrears: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Peace {
    pub war: u32,
    pub payer: u32,
    pub payee: u32,
    pub offered: u32,
    pub total: f64,
    pub months: u32,
    pub accepted: Option<u32>,
    pub treaty: Option<u32>,
    pub cause: u64,
    pub receipts: Vec<Payment>,
    pub missed: u32,
    pub breached: Option<u32>,
}
impl Peace {
    pub fn paid(&self) -> f64 {
        self.receipts.iter().map(|p| p.paid).sum()
    }
}
impl History {
    pub fn offer_peace(&mut self, war: u32, payer: u32, total: f64, months: u32) -> Result<u32> {
        ensure!(
            total.is_finite() && total > 0. && total <= 1e9 && (1..=60).contains(&months),
            "invalid peace terms"
        );
        let w = self
            .politics
            .as_ref()
            .and_then(|p| p.wars.get(war as usize))
            .ok_or_else(|| anyhow::anyhow!("unknown war"))?;
        ensure!(
            w.ended.is_none() && [w.attacker, w.defender].contains(&payer),
            "offer requires a party to an active war"
        );
        let payee = if payer == w.attacker {
            w.defender
        } else {
            w.attacker
        };
        let g = self
            .governance
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("no diplomacy"))?;
        ensure!(
            !g.peace.iter().any(|p| p.war == war
                && p.breached.is_none()
                && (p.accepted.is_some() || self.month < p.offered + 3)),
            "active peace offer"
        );
        self.event("peace_offer",Some(w.goal),None,format!("Council {payer} offered {total:.2} to council {payee} over {months} months for mutual withdrawal"));
        let cause = self.events.last().unwrap().id;
        let g = self.governance.as_mut().unwrap();
        let id = g.peace.len() as u32;
        g.peace.push(Peace {
            war,
            payer,
            payee,
            offered: self.month,
            total,
            months,
            accepted: None,
            treaty: None,
            cause,
            receipts: vec![],
            missed: 0,
            breached: None,
        });
        Ok(id)
    }
    pub fn accept_peace(&mut self, id: u32, accepting_council: u32) -> Result<()> {
        let g = self
            .governance
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("no diplomacy"))?;
        let p = g
            .peace
            .get(id as usize)
            .ok_or_else(|| anyhow::anyhow!("unknown offer"))?
            .clone();
        ensure!(
            p.accepted.is_none() && self.month < p.offered + 3 && accepting_council == p.payee,
            "offer expired or wrong accepting party"
        );
        ensure!(
            self.politics.as_ref().unwrap().wars[p.war as usize]
                .ended
                .is_none(),
            "war already ended"
        );
        let installment = p.total / p.months as f64;
        let s = self
            .society
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("no councils"))?;
        ensure!(
            s.councils[p.payer as usize].treasury >= installment,
            "first installment is unfunded"
        );
        let mut parties = [p.payer, p.payee];
        parties.sort();
        ensure!(
            !g.treaties
                .iter()
                .any(|t| t.parties == parties && t.expires > self.month),
            "existing protection treaty"
        );
        self.event(
            "peace_signed",
            None,
            None,
            format!(
                "Councils {} and {} accepted funded withdrawal; first payment {installment:.2}",
                p.payer, p.payee
            ),
        );
        let cause = self.events.last().unwrap().id;
        self.events.last_mut().unwrap().causes.push(p.cause);
        let society = self.society.as_mut().unwrap();
        society.councils[p.payer as usize].treasury -= installment;
        society.councils[p.payee as usize].treasury += installment;
        let mut raids = std::mem::take(&mut society.raids);
        for r in &mut raids {
            if r.war == Some(p.war) && !r.returning {
                for siege in &mut self.military.siege.sieges {
                    if siege.army == r.id && siege.ended.is_none() {
                        siege.ended = Some(self.month);
                        siege.reason = "negotiated peace".into();
                    }
                }
                r.returning = true;
                r.occupation_until = None;
                r.arrives = self.month + r.return_duration(society);
                r.cause = cause;
            }
        }
        society.raids = raids;
        let w = &mut self.politics.as_mut().unwrap().wars[p.war as usize];
        w.ended = Some(self.month);
        w.outcome = "negotiated peace".into();
        let g = self.governance.as_mut().unwrap();
        let treaty = g.treaties.len() as u32;
        g.treaties.push(Treaty {
            id: treaty,
            parties,
            signed: self.month,
            expires: self.month + 120,
            expired: None,
            cause,
        });
        let p = &mut g.peace[id as usize];
        p.accepted = Some(self.month);
        p.treaty = Some(treaty);
        p.receipts.push(Payment {
            month: self.month,
            due: installment,
            paid: installment,
            arrears: 0.,
        });
        Ok(())
    }
    pub(crate) fn peace_payments(&mut self) {
        let Some(mut g) = self.governance.take() else {
            return;
        };
        for p in &mut g.peace {
            let Some(start) = p.accepted else { continue };
            if p.breached.is_some()
                || p.paid() >= p.total - 1e-8
                || p.receipts.last().is_some_and(|r| r.month >= self.month)
            {
                continue;
            }
            let scheduled =
                p.total * ((self.month - start + 1).min(p.months) as f64 / p.months as f64);
            let due = (scheduled - p.paid()).max(0.);
            let society = self.society.as_mut().unwrap();
            let paid = due.min(society.councils[p.payer as usize].treasury);
            society.councils[p.payer as usize].treasury -= paid;
            society.councils[p.payee as usize].treasury += paid;
            let arrears = (due - paid).max(0.);
            p.missed = if arrears > 1e-8 { p.missed + 1 } else { 0 };
            p.receipts.push(Payment {
                month: self.month,
                due,
                paid,
                arrears,
            });
            self.event(
                "peace_payment",
                None,
                None,
                format!(
                    "Council {} paid {paid:.2}/{due:.2} to {}; arrears {arrears:.2}",
                    p.payer, p.payee
                ),
            );
            self.events.last_mut().unwrap().causes.push(p.cause);
            if p.missed >= 3 {
                p.breached = Some(self.month);
                let t = &mut g.treaties[p.treaty.unwrap() as usize];
                t.expires = self.month;
                t.expired = Some(self.month);
                for r in &mut g.relations {
                    if r.parties == t.parties {
                        r.trust = (r.trust - 25.).max(0.);
                    }
                }
                self.politics.as_mut().unwrap().wars[p.war as usize].outcome =
                    "peace obligations breached".into();
                self.event(
                    "peace_breached",
                    None,
                    None,
                    format!(
                        "Three unpaid peace installments ended protection; {:.2} remains unpaid",
                        p.total - p.paid()
                    ),
                );
                self.events.last_mut().unwrap().causes.push(p.cause);
            }
        }
        self.governance = Some(g);
    }
}
pub(crate) fn validate(h: &History, terms: &[Peace]) -> Result<()> {
    for p in terms {
        ensure!(
            h.politics
                .as_ref()
                .is_some_and(|x| (p.war as usize) < x.wars.len())
                && [p.payer, p.payee]
                    .iter()
                    .all(|c| (*c as usize) < h.civilizations.len())
                && p.payer != p.payee
                && p.total.is_finite()
                && p.total > 0.
                && p.total <= 1e9
                && (1..=60).contains(&p.months)
                && p.offered <= h.month
                && p.accepted.is_none_or(|m| m >= p.offered && m <= h.month)
                && p.paid() <= p.total + 1e-6
                && (p.cause as usize) < h.events.len()
                && p.accepted.is_some() == p.treaty.is_some()
                && p.accepted.is_some() != p.receipts.is_empty()
                && p.treaty
                    .is_none_or(|id| h.governance.as_ref().is_some_and(|g| g
                        .treaties
                        .get(id as usize)
                        .is_some_and(|t| t.signed == p.accepted.unwrap()
                            && t.parties.contains(&p.payer)
                            && t.parties.contains(&p.payee))))
                && p.breached
                    .is_none_or(|m| p.accepted.is_some_and(|a| m >= a && m <= h.month))
                && p.receipts.iter().all(|r| r.month <= h.month
                    && r.due.is_finite()
                    && r.paid.is_finite()
                    && r.arrears.is_finite()
                    && r.due >= 0.
                    && r.paid >= 0.
                    && r.paid <= r.due + 1e-8
                    && (r.due - r.paid - r.arrears).abs() < 1e-6)
                && p.receipts.windows(2).all(|r| r[0].month < r[1].month),
            "invalid peace obligation"
        );
    }
    Ok(())
}
impl crate::gpu::Generator {
    pub fn offer_peace(&mut self, war: u32, payer: u32, total: f64, months: u32) -> Result<u32> {
        self.validate_living_boundary()?;
        self.civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("no history"))?
            .offer_peace(war, payer, total, months)
    }
    pub fn accept_peace(&mut self, id: u32, council: u32) -> Result<()> {
        self.validate_living_boundary()?;
        self.civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("no history"))?
            .accept_peace(id, council)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "requires hardware GPU"]
    fn peace_requires_acceptance_and_real_installments_and_survives_checkpoint() {
        let mut g = crate::continuity_fixture::world();
        let h = g.civilizations.as_mut().unwrap();
        h.politics
            .as_mut()
            .unwrap()
            .wars
            .push(crate::politics::War {
                name: "Test dispute".into(),
                id: 0,
                attacker: 0,
                defender: 1,
                goal: 1,
                started: h.month,
                ended: None,
                outcome: String::new(),
                cause: 0,
            });
        let total = h.sites[0].economy.finance[0].min(12.) as f64;
        h.sites[0].economy.finance[0] -= total as f32;
        h.society.as_mut().unwrap().councils[0].treasury += total;
        assert!(total > 0.);
        let before: f64 = h
            .society
            .as_ref()
            .unwrap()
            .councils
            .iter()
            .map(|c| c.treasury)
            .sum();
        let id = h.offer_peace(0, 0, total, 4).unwrap();
        assert!(h.politics.as_ref().unwrap().wars[0].ended.is_none());
        assert!(h.accept_peace(id, 0).is_err());
        h.accept_peace(id, 1).unwrap();
        assert!(h.governance.as_ref().unwrap().protected(0, 1, h.month));
        assert_eq!(h.governance.as_ref().unwrap().peace[0].paid(), total / 4.);
        h.peace_payments(); // Same boundary cannot pay twice.
        assert_eq!(h.governance.as_ref().unwrap().peace[0].receipts.len(), 1);
        let mut resumed: crate::civilization::History =
            serde_json::from_slice(&serde_json::to_vec(h).unwrap()).unwrap();
        for _ in 0..3 {
            h.month += 1;
            resumed.month += 1;
            h.peace_payments();
            resumed.peace_payments();
        }
        assert_eq!(
            serde_json::to_value(&*h).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        assert!((h.governance.as_ref().unwrap().peace[0].paid() - total).abs() < 1e-8);
        let after: f64 = h
            .society
            .as_ref()
            .unwrap()
            .councils
            .iter()
            .map(|c| c.treasury)
            .sum();
        assert!((before - after).abs() < 1e-8);
        super::validate(h, &h.governance.as_ref().unwrap().peace).unwrap();

        // A matched first installment followed by three genuinely unfunded dues.
        let mut poor: crate::civilization::History =
            serde_json::from_slice(&serde_json::to_vec(h).unwrap()).unwrap();
        let p = &mut poor.governance.as_mut().unwrap().peace[0];
        p.receipts.truncate(1);
        poor.month = p.accepted.unwrap();
        let cash = poor.society.as_ref().unwrap().councils[0].treasury;
        poor.society.as_mut().unwrap().councils[0].treasury = 0.;
        poor.society.as_mut().unwrap().councils[1].treasury += cash;
        for _ in 0..3 {
            poor.month += 1;
            poor.peace_payments();
        }
        assert!(poor.governance.as_ref().unwrap().peace[0]
            .breached
            .is_some());
        assert!(!poor
            .governance
            .as_ref()
            .unwrap()
            .protected(0, 1, poor.month));
        assert_eq!(
            poor.politics.as_ref().unwrap().wars[0].outcome,
            "peace obligations breached"
        );
        assert!(poor.society.as_ref().unwrap().raids.is_empty());
    }
}
