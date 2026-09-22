//! ZIP margin learning adapted to the pilot's paired quotes and midpoint trades.
//! Fixed-point arithmetic and seeded target perturbations make replay exact.
use crate::marketplace::Side;

pub const SCALE: i64 = 1_000_000;
const DEFAULT_LEARNING_RATE: u32 = 300_000;
const DEFAULT_MOMENTUM: u32 = 50_000;
const DEFAULT_RELATIVE_TARGET: u32 = 50_000;
const DEFAULT_ABSOLUTE_TICKS: u32 = 1;
const DEFAULT_SEED: u64 = 7;
const RNG_INCREMENT: u64 = 0x9e3779b97f4a7c15;
const RNG_MIX_A: u64 = 0xbf58476d1ce4e5b9;
const RNG_MIX_B: u64 = 0x94d049bb133111eb;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    /// Fractions in millionths.
    pub learning_rate: u32,
    pub momentum: u32,
    pub relative_target: u32,
    pub absolute_ticks: u32,
    pub seed: u64,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            learning_rate: DEFAULT_LEARNING_RATE,
            momentum: DEFAULT_MOMENTUM,
            relative_target: DEFAULT_RELATIVE_TARGET,
            absolute_ticks: DEFAULT_ABSOLUTE_TICKS,
            seed: DEFAULT_SEED,
        }
    }
}
impl Config {
    pub fn valid(self) -> bool {
        self.learning_rate > 0
            && self.learning_rate <= SCALE as u32
            && self.momentum < SCALE as u32
            && self.relative_target <= SCALE as u32
            && self.absolute_ticks <= i32::MAX as u32
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Rejected { side: Side, price: i32 },
    Trade { price: i32 },
    SettlementFailed { price: i32 },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Learning {
    /// Signed profit margin in millionths: buyer <= 0, seller >= 0.
    pub margin: i64,
    /// Previous price adjustment in millionths of a payment unit.
    pub previous_adjustment: i64,
    pub random_state: u64,
    pub updates: u64,
}
fn random(state: &mut u64) -> u64 {
    *state = state.wrapping_add(RNG_INCREMENT);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(RNG_MIX_A);
    z = (z ^ (z >> 27)).wrapping_mul(RNG_MIX_B);
    z ^ (z >> 31)
}
impl Learning {
    pub fn new(config: Config, limit: i32, quote: i32, identity: &[u64]) -> Self {
        let mut random_state = config.seed;
        for id in identity {
            random_state ^= id;
            random(&mut random_state);
        }
        Self {
            margin: i64::from(quote) * SCALE / i64::from(limit) - SCALE,
            previous_adjustment: 0,
            random_state,
            updates: 0,
        }
    }
    fn value(&self, limit: i32) -> i128 {
        i128::from(limit) * (i128::from(SCALE) + i128::from(self.margin))
    }
    pub fn quote(&self, limit: i32, tick: i32, side: Side) -> i32 {
        let unit = i128::from(tick) * i128::from(SCALE);
        let price = ((self.value(limit) + unit / 2) / unit) * i128::from(tick);
        let max = i32::MAX / tick * tick;
        let price = price.clamp(i128::from(tick), i128::from(max)) as i32;
        match side {
            Side::Buy => price.min(limit),
            Side::Sell => price.max(limit),
        }
    }
    pub fn valid(&self, side: Side) -> bool {
        match side {
            Side::Buy => (-SCALE..=0).contains(&self.margin),
            Side::Sell => (0..=i64::from(i32::MAX) * SCALE).contains(&self.margin),
        }
    }
    /// Observes only public prices; private limits of other traders are absent.
    pub fn observe(&mut self, config: Config, side: Side, limit: i32, tick: i32, event: &Event) {
        let own = self.quote(limit, tick, side);
        let (price, up) = match *event {
            Event::Rejected {
                side: Side::Buy,
                price,
            } if side == Side::Buy && own <= price => (price, true),
            Event::Rejected {
                side: Side::Sell,
                price,
            } if side == Side::Sell && own >= price => (price, false),
            // A completed bilateral trade permits everyone to compare their
            // own quote to its price: executable traders seek more surplus,
            // uncompetitive traders move toward the observed price.
            Event::Trade { price } => (
                price,
                match side {
                    Side::Buy => own < price,
                    Side::Sell => own <= price,
                },
            ),
            _ => return,
        };
        let rel = random(&mut self.random_state) % (u64::from(config.relative_target) + 1);
        let abs = random(&mut self.random_state) % (SCALE as u64 + 1);
        let displacement = i128::from(price) * i128::from(rel)
            + i128::from(tick) * i128::from(config.absolute_ticks) * i128::from(abs);
        let target =
            i128::from(price) * i128::from(SCALE) + if up { displacement } else { -displacement };
        let current = self.value(limit);
        let error = (target - current) * i128::from(config.learning_rate) / i128::from(SCALE);
        let adjustment = (i128::from(config.momentum) * i128::from(self.previous_adjustment)
            + (i128::from(SCALE) - i128::from(config.momentum)) * error)
            / i128::from(SCALE);
        // Bound the model's finite price space and reservation limit. These are
        // explicit adaptations for integer, nonnegative physical settlement.
        let max = i32::MAX / tick * tick;
        let (low, high) = match side {
            Side::Buy => (tick, limit),
            Side::Sell => (limit, max),
        };
        let next = (current + adjustment).clamp(
            i128::from(low) * i128::from(SCALE),
            i128::from(high) * i128::from(SCALE),
        );
        self.margin = (next / i128::from(limit) - i128::from(SCALE)) as i64;
        self.previous_adjustment =
            (next - current).clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64;
        self.updates = self.updates.saturating_add(1);
    }
}
