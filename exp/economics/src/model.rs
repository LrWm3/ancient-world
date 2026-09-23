//! Small typed tables; IDs stay stable when records are reordered.
use crate::maintenance::{Condition, ConditionRule, MaintenanceSettlement, TerminalTransition};
use std::collections::BTreeMap;

pub type AgentId = u32;
pub type ResourceId = u32;
pub type DefinitionId = u32;
pub type AssetId = u32;
pub type Account = (AgentId, ResourceId);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceKind {
    Stock,
    Capacity,
    Fulfillment,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resource {
    pub id: ResourceId,
    pub name: String,
    pub kind: ResourceKind,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Agent {
    pub id: AgentId,
    pub name: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Amount {
    pub resource: ResourceId,
    pub quantity: i32,
}
impl Amount {
    pub fn new(resource: ResourceId, quantity: i32) -> Self {
        Self { resource, quantity }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Requirement {
    pub resource: ResourceId,
    pub quantity: i32,
    /// Smaller ranks receive priority; units of different needs are not summed.
    pub priority: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
/// Optional activity component shared by people, institutions and other agent types.
pub struct Participant {
    pub agent: AgentId,
    pub capacity: Amount,
    pub needs: Vec<Requirement>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Asset {
    pub id: AssetId,
    pub owner: AgentId,
    pub kind: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UseRight {
    pub id: u32,
    pub holder: AgentId,
    pub asset: AssetId,
    pub from: u32,
    pub through: u32,
    pub output_owner: AgentId,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Execution {
    Productive,
    Consumption,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stage {
    pub name: String,
    pub months: u32,
    pub entry_inputs: Vec<Amount>,
    pub monthly_services: Vec<Amount>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessDefinition {
    pub id: DefinitionId,
    pub name: String,
    pub execution: Execution,
    pub enabled: bool,
    pub asset_kind: Option<u32>,
    pub stages: Vec<Stage>,
    pub outputs: Vec<Amount>,
}
impl ProcessDefinition {
    pub fn duration(&self) -> u32 {
        self.stages.iter().map(|s| s.months).sum()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Active,
    Completed,
    Aborted,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessInstance {
    pub id: u64,
    pub definition: DefinitionId,
    pub operator: AgentId,
    pub beneficiary: AgentId,
    pub goal: Option<ResourceId>,
    pub asset: Option<AssetId>,
    pub right: Option<u32>,
    pub start: u32,
    pub reserved_through: u32,
    pub stage: usize,
    pub elapsed: u32,
    pub status: Status,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Open,
    Acquire,
    Due,
    ClearArrears,
    Productive,
    Consumption,
    Close,
}
impl Phase {
    pub fn next(self, month: u32) -> (u32, Self) {
        match self {
            Self::Open | Self::Acquire | Self::Due => (month, Self::Productive),
            Self::Productive | Self::ClearArrears => (month, Self::Consumption),
            Self::Consumption => (month, Self::Close),
            Self::Close => (month + 1, Self::Open),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub credit: crate::credit::Book,
    pub marketplaces: BTreeMap<AgentId, crate::marketplace::Memory>,
    pub memberships:
        BTreeMap<(AgentId, AgentId, crate::membership::Role), crate::membership::Agreement>,
    pub household_remainders: BTreeMap<Account, i32>,
    pub exchange: crate::exchange::ExchangeState,
    pub month: u32,
    pub phase: Phase,
    pub next_batch: u64,
    pub balances: BTreeMap<Account, i32>,
    pub processes: BTreeMap<u64, ProcessInstance>,
    pub conditions: BTreeMap<Account, Condition>,
    pub terminal: BTreeMap<AgentId, TerminalTransition>,
    pub equipment: BTreeMap<AssetId, crate::equipment::DurableAsset>,
    pub practice: BTreeMap<(AgentId, u32), u32>,
    pub filled_offers: std::collections::BTreeSet<u32>,
    pub pending_production: Option<Box<Batch>>,
    pub obligations: BTreeMap<(u32, u32), crate::commitments::Obligation>,
    pub accepted_agreements: BTreeMap<u32, crate::commitments::Agreement>,
}
impl State {
    pub fn balance(&self, owner: AgentId, resource: ResourceId) -> i32 {
        self.balances.get(&(owner, resource)).copied().unwrap_or(0)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Priority {
    ContinuingFirst,
    NewFirst,
    /// Requirement rank before continuation status, with stable IDs breaking ties.
    NeedFirst,
    /// Diagnostic rollout policy: prefer one provision kind in productive allocation.
    NeedFirstFor(ResourceId),
    /// Bounded consequence forecasts; commit only the selected current action.
    ConsequenceAware,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduledStart {
    pub month: u32,
    pub agent: AgentId,
    pub definition: DefinitionId,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct World {
    pub work_choice: Option<crate::work_choice::Config>,
    pub credit: Option<crate::credit::Config>,
    pub marketplaces: Vec<crate::marketplace::Marketplace>,
    pub negotiation: Option<crate::negotiation::Session>,
    pub need_orders: Option<crate::need_orders::Policy>,
    pub pool_market: Option<crate::pool_market::Config>,
    pub competition: Option<crate::competition::Config>,
    /// Open access templates bind their debtor and right holder only on acceptance.
    pub open_access_offers: std::collections::BTreeSet<u32>,
    pub agent_search: BTreeMap<AgentId, crate::search::SearchConfig>,
    pub transaction_policy: Option<crate::opportunities::Policy>,
    pub households: Vec<crate::households::Agreement>,
    pub market: Option<crate::exchange::Market>,
    pub activities: crate::activities::Activities,
    pub agents: Vec<Agent>,
    pub participants: Vec<Participant>,
    pub condition_rules: Vec<ConditionRule>,
    pub resources: Vec<Resource>,
    pub assets: Vec<Asset>,
    pub rights: Vec<UseRight>,
    pub definitions: Vec<ProcessDefinition>,
    /// Buffer/score horizon; contract-linked candidates also cover lead time.
    pub horizon: u32,
    pub priority: Priority,
    // Testable exogenous monthly capacities; never read by the forecaster.
    pub capacity_overrides: BTreeMap<(u32, AgentId), i32>,
    pub scheduled_starts: Vec<ScheduledStart>,
    pub storage: crate::storage::Storage,
    pub issuance: Vec<crate::currency::Issuance>,
    pub bids: Vec<crate::currency::Bid>,
    pub pools: Vec<crate::pools::Pool>,
    pub pool_inputs: Vec<crate::pools::PoolInput>,
    pub offers: Vec<crate::equipment::Offer>,
    pub techniques: Vec<crate::equipment::Technique>,
    pub practice_rules: Vec<crate::equipment::PracticeRule>,
    pub agreements: Vec<crate::commitments::Agreement>,
    pub access_offers: Vec<crate::commitments::Agreement>,
    pub decision_horizon: Option<u32>,
    pub payment_policy: crate::commitments::PaymentPolicy,
}
impl World {
    pub fn definition(&self, id: DefinitionId) -> &ProcessDefinition {
        self.definitions
            .iter()
            .find(|d| d.id == id)
            .expect("validated definition")
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Effect {
    pub account: Account,
    pub delta: i32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessChange {
    pub before: Option<ProcessInstance>,
    pub after: ProcessInstance,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transaction {
    pub forward: Option<crate::forward::Event>,
    pub delivery: Option<crate::exchange::Delivery>,
    pub royalty: Option<crate::exchange::Royalty>,
    pub cause: String,
    pub effects: Vec<Effect>,
    pub process: Option<ProcessChange>,
    pub technique_use: Option<crate::equipment::TechniqueUse>,
    pub trade: Option<crate::equipment::Trade>,
    pub stock_trade: Option<crate::currency::StockTrade>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reason {
    Selected,
    NoDeficit,
    ActiveOutputSufficient,
    NoKnownChain,
    NoBeneficialProcess,
    MissingStock,
    MissingEquipment,
    MissingRight,
    UnpaidObligation,
    RightTooShort,
    Occupied,
    InsufficientCapacity,
    InsufficientStorage,
    Disabled,
    NotPermitted,
    Inactive,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub agent: AgentId,
    pub need: Option<ResourceId>,
    pub definition: Option<DefinitionId>,
    pub reason: Reason,
    pub requested: i32,
    pub allocated: i32,
    pub completed: i32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Batch {
    pub work_choice: Option<crate::work_choice::Decision>,
    pub credit: Option<crate::credit::Boundary>,
    pub negotiation: Option<crate::negotiation::Round>,
    pub pool_market: Option<crate::pool_market::Round>,
    pub additional_access: Vec<(u32, AgentId)>,
    pub additional_memberships: Vec<(u32, AgentId)>,
    pub allocation: Option<crate::competition::Round>,
    pub access_applicant: Option<AgentId>,
    pub accept_membership: Option<(u32, AgentId)>,
    pub household: Option<crate::households::Boundary>,
    pub id: u64,
    pub month: u32,
    pub phase: Phase,
    pub transactions: Vec<Transaction>,
    pub receipts: Vec<Receipt>,
    pub maintenance: Option<MaintenanceSettlement>,
    pub decision: Option<crate::planning::Decision>,
    pub production_plan: Option<Box<Batch>>,
    pub commitments: Option<crate::commitments::Settlement>,
    pub plot_request: Option<crate::plots::Request>,
    pub accept_access: Option<u32>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonthReport {
    pub obligations: BTreeMap<(u32, u32), crate::commitments::Obligation>,
    pub accepted_agreements: BTreeMap<u32, crate::commitments::Agreement>,
    pub month: u32,
    pub agent: AgentId,
    pub balances: BTreeMap<ResourceId, i32>,
    pub needs: BTreeMap<ResourceId, NeedReport>,
    pub conditions: BTreeMap<ResourceId, Condition>,
    pub terminal: Option<TerminalTransition>,
    pub equipment: BTreeMap<AssetId, crate::equipment::DurableAsset>,
    pub practice: BTreeMap<(AgentId, u32), u32>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NeedReport {
    pub desired: i32,
    pub fulfilled: i32,
    pub deficit: i32,
}
impl MonthReport {
    pub fn fulfilled(&self, resource: ResourceId) -> i32 {
        self.needs.get(&resource).map(|n| n.fulfilled).unwrap_or(0)
    }
    pub fn deficit(&self, resource: ResourceId) -> i32 {
        self.needs.get(&resource).map(|n| n.deficit).unwrap_or(0)
    }
}

impl Batch {
    pub fn empty(state: &State) -> Self {
        Self {
            work_choice: None,
            credit: None,
            negotiation: None,
            household: None,
            id: state.next_batch,
            month: state.month,
            phase: state.phase,
            transactions: vec![],
            receipts: vec![],
            maintenance: None,
            decision: None,
            production_plan: None,
            commitments: None,
            accept_access: None,
            access_applicant: None,
            additional_access: vec![],
            additional_memberships: vec![],
            allocation: None,
            pool_market: None,
            accept_membership: None,
            plot_request: None,
        }
    }
}
