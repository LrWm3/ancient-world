//! Existing-account adapters. Quotes inspect both representations before either
//! balance is written, so a failed transfer cannot debit only one counterparty.
use super::{Account, CurrencyId, SHARED_CURRENCY};
use crate::civilization::History;
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};

const MAX_TRANSFER_REFINEMENTS: usize = 8;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Financing {
    pub principal_received: f64,
    pub principal_paid: f64,
    pub interest_received: f64,
    pub interest_paid: f64,
}
impl Financing {
    pub fn net_cash(&self) -> f64 {
        self.principal_received - self.principal_paid + self.interest_received - self.interest_paid
    }
    pub fn net_interest(&self) -> f64 {
        self.interest_received - self.interest_paid
    }
    pub fn validate(&self) -> bool {
        [
            self.principal_received,
            self.principal_paid,
            self.interest_received,
            self.interest_paid,
        ]
        .iter()
        .all(|x| x.is_finite() && *x >= 0.)
    }
    fn record(&mut self, received: bool, principal: f64, interest: f64) {
        if received {
            self.principal_received += principal;
            self.interest_received += interest;
        } else {
            self.principal_paid += principal;
            self.interest_paid += interest;
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Balance {
    Single(f32),
    Double(f64),
}
impl Balance {
    fn value(self) -> f64 {
        match self {
            Self::Single(x) => x as f64,
            Self::Double(x) => x,
        }
    }
    fn with(self, value: f64) -> Self {
        match self {
            Self::Single(_) => Self::Single(value as f32),
            Self::Double(_) => Self::Double(value),
        }
    }
    fn adjacent(self, up: bool) -> Self {
        match self {
            Self::Single(x) => Self::Single(f32::from_bits(if up {
                x.to_bits() + 1
            } else {
                x.to_bits().saturating_sub(1)
            })),
            Self::Double(x) => Self::Double(f64::from_bits(if up {
                x.to_bits() + 1
            } else {
                x.to_bits().saturating_sub(1)
            })),
        }
    }
}

/// Choose a representable amount common to both accounts. This may leave tiny
/// untransferred balances; it never silently absorbs them into income or debt.
fn quote(from: Balance, to: Balance, requested: f64) -> Result<(Balance, Balance, f64)> {
    ensure!(
        requested.is_finite() && requested >= 0.,
        "invalid requested transfer"
    );
    ensure!(
        [from.value(), to.value()]
            .iter()
            .all(|v| v.is_finite() && *v >= 0.),
        "invalid account cash"
    );
    let mut candidate = requested.min(from.value());
    for _ in 0..MAX_TRANSFER_REFINEMENTS {
        if candidate == 0. {
            break;
        }
        let mut credit = to.with(to.value() + candidate);
        ensure!(credit.value().is_finite(), "recipient cash overflow");
        if credit.value() - to.value() > candidate {
            credit = credit.adjacent(false);
        }
        let credited = credit.value() - to.value();
        if credited <= 0. {
            break;
        }
        let mut debit = from.with(from.value() - credited);
        if from.value() - debit.value() > credited {
            debit = debit.adjacent(true);
        }
        let debited = from.value() - debit.value();
        if debited == credited {
            return Ok((debit, credit, debited));
        }
        if debited <= 0. || debited >= candidate {
            break;
        }
        candidate = debited;
    }
    // Incompatible representation steps may admit no useful exact transfer.
    // Decline without modifying either account; do not accumulate rounding money.
    Ok((from, to, 0.))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transfer {
    pub month: u32,
    pub from: Account,
    pub to: Account,
    pub currency: CurrencyId,
    pub requested: f64,
    pub principal: f64,
    pub interest: f64,
}
impl Transfer {
    pub fn amount(&self) -> f64 {
        self.principal + self.interest
    }
}

impl History {
    fn credit_balance(&self, account: Account) -> Result<Balance> {
        Ok(match account {
            Account::Town(id) => Balance::Single(
                self.sites
                    .get(id as usize)
                    .filter(|s| s.id == id && !s.abandoned)
                    .context("unavailable town account")?
                    .economy
                    .finance[0],
            ),
            Account::Council(id) => Balance::Double(
                self.society
                    .as_ref()
                    .and_then(|s| s.councils.get(id as usize))
                    .filter(|c| c.civilization == id)
                    .context("missing council account")?
                    .treasury,
            ),
            Account::Institution(id) => Balance::Double(
                self.culture
                    .as_ref()
                    .and_then(|c| c.institutions.get(id as usize))
                    .filter(|n| n.id == id && n.active)
                    .context("unavailable institution account")?
                    .treasury,
            ),
            Account::Operator(id) => Balance::Double(
                self.enterprises
                    .as_ref()
                    .and_then(|e| e.firms.get(id as usize))
                    .filter(|f| f.id == id && f.closed.is_none())
                    .context("unavailable operator account")?
                    .cash,
            ),
        })
    }
    pub fn credit_account_cash(&self, account: Account) -> Result<f64> {
        Ok(self.credit_balance(account)?.value())
    }

    fn set_credit_balance(
        &mut self,
        account: Account,
        balance: Balance,
        received: bool,
        principal: f64,
        interest: f64,
    ) {
        let value = balance.value();
        match account {
            Account::Town(id) => self.sites[id as usize].economy.finance[0] = value as f32,
            Account::Council(id) => {
                self.society.as_mut().unwrap().councils[id as usize].treasury = value
            }
            Account::Institution(id) => {
                self.culture.as_mut().unwrap().institutions[id as usize].treasury = value
            }
            Account::Operator(id) => {
                let f = &mut self.enterprises.as_mut().unwrap().firms[id as usize];
                f.cash = value;
                f.financing.record(received, principal, interest);
            }
        }
    }

    /// Internal settlement primitive, not underwriting authorization. Interest is
    /// paid first within the request, with the rest classified as principal.
    pub fn transfer_credit_cash(
        &mut self,
        from: Account,
        to: Account,
        currency: CurrencyId,
        principal: f64,
        interest: f64,
    ) -> Result<Transfer> {
        ensure!(from != to, "self transfer is not permitted");
        ensure!(
            currency == SHARED_CURRENCY,
            "account currency is not supported"
        );
        ensure!(
            [principal, interest]
                .iter()
                .all(|x| x.is_finite() && *x >= 0.),
            "invalid transfer components"
        );
        let requested = principal + interest;
        let (debit, credit, actual) = quote(
            self.credit_balance(from)?,
            self.credit_balance(to)?,
            requested,
        )?;
        let paid_interest = actual.min(interest);
        let paid_principal = actual - paid_interest;
        // Preflight cumulative operator counters too, before either balance changes.
        for (account, received) in [(from, false), (to, true)] {
            if let Account::Operator(id) = account {
                let mut flows = self.enterprises.as_ref().unwrap().firms[id as usize]
                    .financing
                    .clone();
                flows.record(received, paid_principal, paid_interest);
                ensure!(flows.validate(), "operator financing overflow");
            }
        }
        self.set_credit_balance(from, debit, false, paid_principal, paid_interest);
        self.set_credit_balance(to, credit, true, paid_principal, paid_interest);
        Ok(Transfer {
            month: self.month,
            from,
            to,
            currency,
            requested,
            principal: paid_principal,
            interest: paid_interest,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_cash_transfer_across_precisions_and_scales() {
        for a in [0., 0.1, 100., 1e6, 1e20] {
            for b in [0., 0.3, 23., 1e8, 1e25] {
                for amount in [0., 1e-12, 0.01, 17.1, 1e20] {
                    for (from, to) in [
                        (Balance::Single(a as f32), Balance::Double(b)),
                        (Balance::Double(a), Balance::Single(b as f32)),
                        (Balance::Single(a as f32), Balance::Single(b as f32)),
                        (Balance::Double(a), Balance::Double(b)),
                    ] {
                        let (debit, credit, paid) = quote(from, to, amount).unwrap();
                        assert_eq!(from.value() - debit.value(), paid);
                        assert_eq!(credit.value() - to.value(), paid);
                        assert!(paid <= amount && paid <= from.value());
                    }
                }
            }
        }
    }
    #[test]
    fn financing_is_not_operating_revenue() {
        let mut flows = Financing::default();
        flows.record(true, 100., 0.);
        assert_eq!(flows.net_interest(), 0.);
        flows.record(false, 25., 2.);
        assert_eq!(flows.net_cash(), 73.);
        assert_eq!(flows.net_interest(), -2.);
        assert!(flows.validate());
    }
    #[test]
    fn invalid_inputs_and_unrepresentable_small_transfers() {
        assert!(quote(Balance::Double(10.), Balance::Double(0.), f64::NAN).is_err());
        assert!(quote(Balance::Double(10.), Balance::Double(0.), -1.).is_err());
        assert!(quote(Balance::Double(1e39), Balance::Single(f32::MAX), 1e39).is_err());
        let (_, _, paid) = quote(Balance::Single(1e20), Balance::Double(0.), 1.).unwrap();
        assert_eq!(paid, 0.);
    }
}
