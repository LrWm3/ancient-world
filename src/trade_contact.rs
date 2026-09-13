//! Recent completed trade, shared by political interests and cultural contact.
//! Observations are not cargo, money, or a second economic ledger.
use serde::{Deserialize, Serialize};

const CONTACT_WINDOW_MONTHS: u32 = 12;
const STAPLE_KG_PER_PERSON_MONTH: f64 = 18.;
const MIN_EXPOSURE_POPULATION: f32 = 1.;
const MIN_CONTACT_DELIVERY_KG: f64 = 1.;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub month: u32,
    pub from: u32,
    pub to: u32,
    pub kg: f64,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TradeContact {
    pub receipts: Vec<Receipt>,
}
impl TradeContact {
    pub fn prune(&mut self, month: u32) {
        self.receipts
            .retain(|r| month.saturating_sub(r.month) < CONTACT_WINDOW_MONTHS);
    }
    /// Coalesce deliveries on a directed pair in a completed month.
    pub fn observe(&mut self, month: u32, from: u32, to: u32, kg: f64) {
        if !kg.is_finite() || kg <= 0. || from == to {
            return;
        }
        if let Some(r) = self
            .receipts
            .iter_mut()
            .find(|r| r.month == month && r.from == from && r.to == to)
        {
            r.kg += kg;
        } else {
            self.receipts.push(Receipt {
                month,
                from,
                to,
                kg,
            });
        }
    }
    /// Recent throughput relative to a year's staple mass. This is commercial
    /// exposure, not merchant income: imports matter alongside exports.
    pub fn exposure(&self, site: u32, month: u32, population: f32) -> f32 {
        let kg: f64 = self
            .receipts
            .iter()
            .filter(|r| {
                r.month <= month
                    && month - r.month < CONTACT_WINDOW_MONTHS
                    && (r.from == site || r.to == site)
            })
            .map(|r| r.kg)
            .sum();
        let scale = population.max(MIN_EXPOSURE_POPULATION) as f64
            * STAPLE_KG_PER_PERSON_MONTH
            * CONTACT_WINDOW_MONTHS as f64;
        (kg / (kg + scale)) as f32
    }
    /// A month with at least one kilogram delivered supports a contact opportunity.
    /// Tiny split consignments coalesce before applying this threshold.
    pub fn links(&self, month: u32) -> impl Iterator<Item = (u32, u32)> + '_ {
        self.receipts
            .iter()
            .filter(move |r| r.month == month && r.kg >= MIN_CONTACT_DELIVERY_KG)
            .map(|r| (r.from, r.to))
    }
    pub fn validate(&self, month: u32, sites: usize) -> anyhow::Result<()> {
        let mut keys = std::collections::BTreeSet::new();
        anyhow::ensure!(
            self.receipts.iter().all(|r| r.month <= month
                && month - r.month < CONTACT_WINDOW_MONTHS
                && (r.from as usize) < sites
                && (r.to as usize) < sites
                && r.from != r.to
                && r.kg.is_finite()
                && r.kg > 0.
                && keys.insert((r.month, r.from, r.to))),
            "invalid recent trade contact"
        );
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recent_trade_expires_and_split_deliveries_are_equivalent() {
        let mut a = TradeContact::default();
        let mut b = a.clone();
        a.observe(1, 0, 1, 2160.);
        for _ in 0..10 {
            b.observe(1, 0, 1, 216.);
        }
        assert_eq!(a.exposure(0, 12, 10.), 0.5);
        assert_eq!(a.exposure(1, 12, 10.), b.exposure(0, 12, 10.));
        assert_eq!(a.exposure(2, 12, 10.), 0.);
        assert_eq!(a.exposure(0, 13, 10.), 0.);
        assert_eq!(a.links(1).count(), 1);
        assert_eq!(a.links(2).count(), 0);
        a.prune(13);
        assert!(a.receipts.is_empty());
        b.validate(12, 2).unwrap();
        assert!(b.validate(13, 2).is_err());
        let resumed: TradeContact =
            serde_json::from_slice(&serde_json::to_vec(&b).unwrap()).unwrap();
        assert_eq!(resumed.exposure(0, 12, 10.), b.exposure(0, 12, 10.));
    }
}
