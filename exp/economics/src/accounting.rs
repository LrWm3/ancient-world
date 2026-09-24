//! Double-entry reporting book. Signed amounts are debit-positive; money is in
//! integer reporting ticks. Domain adapters must reconcile, never invent a plug.
use crate::model::{AgentId, ResourceId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Account {
    Cash,
    RestrictedCash(u32),
    CustodyCash(u32),
    CustodyPayable(u32),
    Inventory(ResourceId),
    PurchasedCapacity(ResourceId),
    Tangible(u32),
    WorkInProgress(u64),
    LoanReceivable(u32),
    ForwardPrepayment(u32),
    DeferredRevenue(u32),
    DuesReceivable(u32, u32),
    DuesPayable(u32, u32),
    DuesIncome,
    DuesExpense,
    SettlementGain,
    SettlementLoss,
    InterestReceivable(u32),
    LoanPayable(u32),
    InterestPayable(u32),
    OpeningEquity,
    Capital,
    MonetaryIssuance,
    ServiceIncome,
    ServiceExpense,
    InterestIncome,
    InterestExpense,
    TransferIncome,
    TransferExpense,
    CreditLoss,
    DebtRelief,
    Sales,
    CostOfSales,
    ConsumptionExpense,
    ProductionExpense,
    ProductionLoss,
    InventoryLoss,
    Depreciation,
    DisposalGain,
    DisposalLoss,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    Asset,
    Liability,
    Equity,
    Income,
    Expense,
}
impl Account {
    pub fn class(&self) -> Class {
        match self {
            Self::Cash
            | Self::RestrictedCash(_)
            | Self::CustodyCash(_)
            | Self::Inventory(_)
            | Self::PurchasedCapacity(_)
            | Self::Tangible(_)
            | Self::WorkInProgress(_)
            | Self::DuesReceivable(_, _)
            | Self::ForwardPrepayment(_)
            | Self::LoanReceivable(_)
            | Self::InterestReceivable(_) => Class::Asset,
            Self::DeferredRevenue(_)
            | Self::DuesPayable(_, _)
            | Self::CustodyPayable(_)
            | Self::LoanPayable(_)
            | Self::InterestPayable(_) => Class::Liability,
            Self::OpeningEquity | Self::Capital | Self::MonetaryIssuance => Class::Equity,
            Self::ServiceIncome
            | Self::DuesIncome
            | Self::SettlementGain
            | Self::InterestIncome
            | Self::TransferIncome
            | Self::DebtRelief
            | Self::Sales
            | Self::DisposalGain => Class::Income,
            _ => Class::Expense,
        }
    }
    pub fn cash(&self) -> bool {
        matches!(self, Self::Cash | Self::RestrictedCash(_))
    }
}
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Flow {
    Operating,
    Investing,
    Financing,
    Internal,
    /// Self-created money, shown separately from external cash flows.
    Issuance,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pub agent: AgentId,
    pub account: Account,
    /// Positive debit, negative credit. Never summed across denominations.
    pub debit: i128,
    pub flow: Option<Flow>,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub id: String,
    pub month: u32,
    pub batch: Option<u64>,
    pub description: String,
    pub lines: Vec<Line>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Book {
    denomination: ResourceId,
    opening_month: u32,
    finalized_through: Option<u32>,
    entries: Vec<Entry>,
    ids: BTreeSet<String>,
    balances: BTreeMap<(AgentId, Account), i128>,
}
const JOURNAL_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct JournalArchive {
    version: u32,
    denomination: ResourceId,
    opening_month: u32,
    finalized_through: Option<u32>,
    entries: Vec<Entry>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Statements {
    pub from: u32,
    pub through: u32,
    pub trial_balance: BTreeMap<Account, i128>,
    pub assets: i128,
    pub liabilities: i128,
    pub equity: i128,
    pub income: BTreeMap<Account, i128>,
    pub expenses: BTreeMap<Account, i128>,
    pub net_income: i128,
    pub opening_equity: i128,
    pub capital_change: i128,
    pub issuance_change: i128,
    pub opening_cash: i128,
    pub closing_cash: i128,
    pub cash_flows: BTreeMap<Flow, i128>,
}
pub(crate) fn add<K: Ord>(map: &mut BTreeMap<K, i128>, key: K, value: i128) -> Result<(), String> {
    let old = map.entry(key).or_default();
    *old = old.checked_add(value).ok_or("accounting amount overflow")?;
    Ok(())
}
impl Book {
    /// Explicit opening recognition, not income. Values are debit-positive assets
    /// and credit-negative liabilities; net recognized assets become opening equity.
    pub fn open(
        denomination: ResourceId,
        positions: BTreeMap<(AgentId, Account), i128>,
    ) -> Result<Self, String> {
        Self::open_at(denomination, 0, positions)
    }
    /// Recognize a snapshot at the end of `month`; earlier results are unavailable.
    pub fn open_at(
        denomination: ResourceId,
        month: u32,
        positions: BTreeMap<(AgentId, Account), i128>,
    ) -> Result<Self, String> {
        let mut lines = vec![];
        let mut net = BTreeMap::new();
        for ((agent, account), debit) in positions {
            if debit == i128::MIN || !matches!(account.class(), Class::Asset | Class::Liability) {
                return Err("opening positions must be assets or liabilities".into());
            }
            add(&mut net, agent, debit)?;
            if debit != 0 {
                lines.push(Line {
                    agent,
                    account,
                    debit,
                    flow: None,
                });
            }
        }
        for (agent, value) in net {
            if value != 0 {
                lines.push(Line {
                    agent,
                    account: Account::OpeningEquity,
                    debit: value.checked_neg().ok_or("opening equity overflow")?,
                    flow: None,
                });
            }
        }
        let mut book = Self {
            denomination,
            opening_month: month,
            finalized_through: None,
            entries: vec![],
            ids: BTreeSet::new(),
            balances: BTreeMap::new(),
        };
        book.publish(
            Entry {
                id: "opening".into(),
                month,
                batch: None,
                description: "Explicit opening recognized positions".into(),
                lines,
            },
            true,
        )?;
        Ok(book)
    }
    pub fn denomination(&self) -> ResourceId {
        self.denomination
    }
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }
    pub fn balances(&self) -> &BTreeMap<(AgentId, Account), i128> {
        &self.balances
    }
    /// Lock completed periods. Standalone callers attest that all entries are posted.
    pub fn finalize_through(&mut self, month: u32) -> Result<(), String> {
        if month <= self.opening_month
            || self.finalized_through.is_some_and(|old| month < old)
            || self.entries.last().is_none_or(|e| month > e.month)
        {
            return Err("invalid finalization boundary".into());
        }
        self.finalized_through = Some(month);
        Ok(())
    }
    pub fn finalized_through(&self) -> Option<u32> {
        self.finalized_through
    }
    pub fn finalized_statements(
        &self,
        agent: AgentId,
        from: u32,
        through: u32,
    ) -> Result<Statements, String> {
        if self.finalized_through.is_none_or(|month| through > month) {
            return Err("reporting period is not finalized".into());
        }
        self.statements(agent, from, through)
    }
    /// Versioned journal only; not a simulation or Audit checkpoint.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(&JournalArchive {
            version: JOURNAL_VERSION,
            denomination: self.denomination,
            opening_month: self.opening_month,
            finalized_through: self.finalized_through,
            entries: self.entries.clone(),
        })
        .map_err(|e| e.to_string())
    }
    /// Rebuild balances and duplicate guards by validating every journal entry.
    pub fn from_json(json: &str) -> Result<Self, String> {
        let archive: JournalArchive = serde_json::from_str(json).map_err(|e| e.to_string())?;
        if archive.version != JOURNAL_VERSION {
            return Err("unsupported journal version".into());
        }
        let opening = archive.entries.first().ok_or("missing opening entry")?;
        if opening.id != "opening"
            || opening.month != archive.opening_month
            || opening.batch.is_some()
            || opening.lines.iter().any(|l| {
                !matches!(l.account.class(), Class::Asset | Class::Liability)
                    && l.account != Account::OpeningEquity
            })
        {
            return Err("invalid opening entry".into());
        }
        let mut book = Self {
            denomination: archive.denomination,
            opening_month: archive.opening_month,
            finalized_through: None,
            entries: vec![],
            ids: BTreeSet::new(),
            balances: BTreeMap::new(),
        };
        for (index, entry) in archive.entries.into_iter().enumerate() {
            book.publish(entry, index == 0)?;
        }
        if let Some(month) = archive.finalized_through {
            book.finalize_through(month)?;
        }
        Ok(book)
    }
    pub fn post(&mut self, entry: Entry) -> Result<(), String> {
        self.publish(entry, false)
    }
    fn publish(&mut self, entry: Entry, opening: bool) -> Result<(), String> {
        if !opening
            && self
                .finalized_through
                .is_some_and(|month| entry.month <= month)
        {
            return Err("accounting period is finalized".into());
        }
        if self.ids.contains(&entry.id)
            || entry.id.is_empty()
            || (!opening && entry.month <= self.opening_month)
            || self.entries.last().is_some_and(|e| e.month > entry.month)
        {
            return Err("duplicate or backdated accounting entry".into());
        }
        let mut totals = BTreeMap::new();
        let mut balances = self.balances.clone();
        let mut internal = BTreeMap::new();
        for l in &entry.lines {
            if (!opening && l.account == Account::OpeningEquity)
                || l.debit == i128::MIN
                || l.debit == 0
                || (!opening && l.account.cash() != l.flow.is_some())
                || (opening && l.flow.is_some())
            {
                return Err("invalid accounting leg or missing cash-flow classification".into());
            }
            add(&mut totals, l.agent, l.debit)?;
            add(&mut balances, (l.agent, l.account.clone()), l.debit)?;
            if l.flow == Some(Flow::Internal) {
                add(&mut internal, l.agent, l.debit)?;
            }
        }
        if totals.values().any(|v| *v != 0) || internal.values().any(|v| *v != 0) {
            return Err(
                "unbalanced entry: each entity and internal cash transfer must balance".into(),
            );
        }
        if balances.iter().any(|((_, a), v)| {
            *v == i128::MIN
                || match a.class() {
                    Class::Asset => *v < 0,
                    Class::Liability => *v > 0,
                    _ => false,
                }
        }) {
            return Err("negative recognized asset or liability".into());
        }
        self.balances = balances;
        self.ids.insert(entry.id.clone());
        self.entries.push(entry);
        Ok(())
    }
    pub fn statements(
        &self,
        agent: AgentId,
        from: u32,
        through: u32,
    ) -> Result<Statements, String> {
        if from <= self.opening_month || through < from {
            return Err("invalid financial reporting period".into());
        }
        let mut s = Statements {
            from,
            through,
            ..Default::default()
        };
        let mut opening = BTreeMap::new();
        for e in self.entries.iter().filter(|e| e.month <= through) {
            for l in e.lines.iter().filter(|l| l.agent == agent) {
                add(&mut s.trial_balance, l.account.clone(), l.debit)?;
                if e.month < from {
                    add(&mut opening, l.account.clone(), l.debit)?;
                    continue;
                }
                match l.account.class() {
                    Class::Income => add(&mut s.income, l.account.clone(), -l.debit)?,
                    Class::Expense => add(&mut s.expenses, l.account.clone(), l.debit)?,
                    Class::Equity => {
                        let movement = if l.account == Account::MonetaryIssuance {
                            &mut s.issuance_change
                        } else {
                            &mut s.capital_change
                        };
                        *movement = movement.checked_sub(l.debit).ok_or("equity overflow")?;
                    }
                    _ => {}
                }
                if let Some(flow) = l.flow {
                    add(&mut s.cash_flows, flow, l.debit)?;
                }
            }
        }
        let sum = |values: Vec<i128>| {
            values.into_iter().try_fold(0_i128, |a, b| {
                a.checked_add(b).ok_or("statement overflow".to_string())
            })
        };
        s.net_income = sum(s.income.values().copied().collect())?
            .checked_sub(sum(s.expenses.values().copied().collect())?)
            .ok_or("income overflow")?;
        s.assets = sum(s
            .trial_balance
            .iter()
            .filter(|(a, _)| a.class() == Class::Asset)
            .map(|(_, v)| *v)
            .collect())?;
        s.liabilities = sum(s
            .trial_balance
            .iter()
            .filter(|(a, _)| a.class() == Class::Liability)
            .map(|(_, v)| -*v)
            .collect())?;
        s.equity = s
            .assets
            .checked_sub(s.liabilities)
            .ok_or("equity overflow")?;
        s.opening_equity = sum(opening
            .iter()
            .filter(|(a, _)| matches!(a.class(), Class::Asset | Class::Liability))
            .map(|(_, v)| *v)
            .collect())?;
        s.opening_cash = sum(opening
            .iter()
            .filter(|(a, _)| a.cash())
            .map(|(_, v)| *v)
            .collect())?;
        s.closing_cash = sum(s
            .trial_balance
            .iter()
            .filter(|(a, _)| a.cash())
            .map(|(_, v)| *v)
            .collect())?;
        let equity = s
            .opening_equity
            .checked_add(s.capital_change)
            .and_then(|v| v.checked_add(s.issuance_change))
            .and_then(|v| v.checked_add(s.net_income))
            .ok_or("equity rollforward overflow")?;
        let cash = s
            .opening_cash
            .checked_add(sum(s.cash_flows.values().copied().collect())?)
            .ok_or("cash rollforward overflow")?;
        if sum(s.trial_balance.values().copied().collect())? != 0
            || equity != s.equity
            || cash != s.closing_cash
        {
            return Err("financial statements do not reconcile".into());
        }
        Ok(s)
    }
}

impl Statements {
    /// Human-readable export; all values use the book's reporting ticks.
    pub fn markdown(&self, agent: AgentId, denomination: ResourceId) -> String {
        let mut out = format!(
            "# Financial statements: agent {agent}\n\nMonths {}–{}, resource {denomination} reporting ticks.\n\n",
            self.from, self.through
        );
        out.push_str("## Balance sheet\n\n| Account | Amount |\n| --- | ---: |\n");
        for (account, value) in &self.trial_balance {
            match account.class() {
                Class::Asset if *value != 0 => {
                    out.push_str(&format!("| {account:?} | {value} |\n"))
                }
                Class::Liability if *value != 0 => {
                    out.push_str(&format!("| {account:?} | {} |\n", -value))
                }
                _ => {}
            }
        }
        out.push_str(&format!("| Total assets | {} |\n| Total liabilities | {} |\n| Equity, including accumulated results | {} |\n\n", self.assets, self.liabilities, self.equity));
        out.push_str("## Income statement\n\n| Account | Amount |\n| --- | ---: |\n");
        for (account, value) in self.income.iter().chain(self.expenses.iter()) {
            out.push_str(&format!("| {account:?} | {value} |\n"));
        }
        out.push_str(&format!("| Net income | {} |\n\n", self.net_income));
        out.push_str("## Changes in equity\n\n| Movement | Amount |\n| --- | ---: |\n");
        out.push_str(&format!("| Opening equity | {} |\n| Capital contributions less distributions | {} |\n| Monetary issuance | {} |\n| Net income | {} |\n| Closing equity | {} |\n\n", self.opening_equity, self.capital_change, self.issuance_change, self.net_income, self.equity));
        out.push_str("## Cash flows\n\nOwned cash includes restricted estate cash; custodian cash is excluded. Issuance is self-created money, shown separately from external cash flows.\n\n| Movement | Amount |\n| --- | ---: |\n");
        out.push_str(&format!("| Opening cash | {} |\n", self.opening_cash));
        for kind in [
            Flow::Operating,
            Flow::Investing,
            Flow::Financing,
            Flow::Internal,
            Flow::Issuance,
        ] {
            out.push_str(&format!(
                "| {kind:?} | {} |\n",
                self.cash_flows.get(&kind).copied().unwrap_or(0)
            ));
        }
        out.push_str(&format!("| Closing cash | {} |\n\n", self.closing_cash));
        out.push_str("## Trial balance\n\nIncome accounts remain cumulative; period income uses journal movements.\n\n| Account | Debit | Credit |\n| --- | ---: | ---: |\n");
        for (account, value) in &self.trial_balance {
            if *value != 0 {
                out.push_str(&format!(
                    "| {account:?} | {} | {} |\n",
                    (*value).max(0),
                    (-value).max(0)
                ));
            }
        }
        out.push_str("\nChecks passed: debits = credits; assets = liabilities + equity; equity and cash rollforwards reconcile.\n");
        out
    }
}
