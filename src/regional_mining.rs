//! Regional control of existing sources, not a second extraction inventory.
use crate::{
    gpu::{Generator, Stage},
    region::Region,
    resources::Resources,
};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegionalMine {
    /// Stable identity and causal root: the activation event.
    pub id: u64,
    pub cell: u32,
    pub site: u32,
    pub opened_month: u32,
    pub retired_month: Option<u32>,
    /// Ore/clay kg per month: ceilings, never guaranteed output or stored mass.
    pub monthly_limit: [f32; 2],
    /// Diagnostic subset of canonical source extraction while controlled.
    pub extracted: [f64; 2],
}

impl Resources {
    pub(crate) fn regional_limit(&self, cell: u32, site: u32) -> [f32; 2] {
        self.regional_mines
            .get(&cell)
            .map_or([f32::INFINITY; 2], |m| {
                if m.site == site {
                    m.monthly_limit
                } else {
                    [0.; 2]
                }
            })
    }
}

fn validate_limit(limit: [f32; 2]) -> Result<()> {
    ensure!(
        limit.iter().all(|x| x.is_finite() && *x >= 0.),
        "invalid regional mining limit"
    );
    Ok(())
}

impl Generator {
    /// Delegate one already surveyed parent source to a local work plan.
    /// Other claimants wait until retirement; actual labor and extraction remain on GPU.
    pub fn activate_regional_mine(
        &mut self,
        region: &Region,
        site: u32,
        limit: [f32; 2],
    ) -> Result<()> {
        validate_limit(limit)?;
        self.validate_living_boundary()?;
        ensure!(
            self.progress.stage == Stage::Boundary,
            "regional control requires a completed boundary"
        );
        let h = self
            .civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("no history"))?;
        ensure!(
            region.seed == self.config.seed
                && region.epoch == self.progress.epoch
                && region.ecological_month == self.ecology.clock.month
                && region.history_month == Some(h.month),
            "stale or foreign regional survey"
        );
        let s = h
            .sites
            .get(site as usize)
            .ok_or_else(|| anyhow::anyhow!("unknown mining site"))?;
        ensure!(!s.abandoned, "cannot activate an abandoned mining site");
        let cell = s.cell;
        ensure!(
            region.cells.iter().any(|c| c.route[2] == cell),
            "site outside regional survey"
        );
        let r = h
            .resources
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("enable shared resources first"))?;
        ensure!(
            !r.regional_mines.contains_key(&cell),
            "source already under regional control"
        );
        let source = r
            .sources
            .get(&cell)
            .ok_or_else(|| anyhow::anyhow!("unknown source"))?;
        let observed = region
            .resource_sources
            .iter()
            .find(|s| s.cell == cell)
            .ok_or_else(|| anyhow::anyhow!("source absent from survey"))?;
        ensure!(
            observed.remaining == source.remaining
                && observed.extracted == source.extracted
                && observed.initial == source.initial
                && observed.mineral == source.mineral
                && observed.ore_good == source.ore_good,
            "source changed since survey"
        );
        r.regional_mines.insert(
            cell,
            RegionalMine {
                id: h.events.len() as u64,
                cell,
                site,
                opened_month: h.month,
                retired_month: None,
                monthly_limit: limit,
                extracted: [0.; 2],
            },
        );
        h.event("regional_mine_activated", Some(site), None,
            format!("Local work plan controls shared source {cell}; ore/clay ceilings {:.2}/{:.2} kg per month; other claimants wait, no resource inventory transferred", limit[0], limit[1]));
        Ok(())
    }

    pub fn set_regional_mining_limit(&mut self, site: u32, limit: [f32; 2]) -> Result<()> {
        validate_limit(limit)?;
        self.validate_living_boundary()?;
        ensure!(
            self.progress.stage == Stage::Boundary,
            "regional control requires a completed boundary"
        );
        let h = self
            .civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("no history"))?;
        let cell = h
            .sites
            .get(site as usize)
            .ok_or_else(|| anyhow::anyhow!("unknown mining site"))?
            .cell;
        let mine = h
            .resources
            .as_mut()
            .and_then(|r| r.regional_mines.get_mut(&cell))
            .ok_or_else(|| anyhow::anyhow!("no active regional mine"))?;
        ensure!(mine.site == site, "site does not control this source");
        if mine.monthly_limit == limit {
            return Ok(());
        }
        mine.monthly_limit = limit;
        let cause = mine.id;
        h.event(
            "regional_mining_limit",
            Some(site),
            None,
            format!(
                "Local ore/clay work ceilings changed to {:.2}/{:.2} kg per month",
                limit[0], limit[1]
            ),
        );
        h.events.last_mut().unwrap().causes.push(cause);
        Ok(())
    }

    pub fn retire_regional_mine(&mut self, site: u32) -> Result<()> {
        self.validate_living_boundary()?;
        ensure!(
            self.progress.stage == Stage::Boundary,
            "regional control requires a completed boundary"
        );
        let h = self
            .civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("no history"))?;
        let cell = h
            .sites
            .get(site as usize)
            .ok_or_else(|| anyhow::anyhow!("unknown mining site"))?
            .cell;
        let r = h
            .resources
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("no source registry"))?;
        let m = r
            .regional_mines
            .get(&cell)
            .ok_or_else(|| anyhow::anyhow!("no active regional mine"))?;
        ensure!(m.site == site, "site does not control this source");
        let used = m.extracted;
        let cause = m.id;
        let mut retired = r.regional_mines.remove(&cell).unwrap();
        retired.retired_month = Some(h.month);
        r.retired_regional_mines.push(retired);
        h.event("regional_mine_retired", Some(site), None,
            format!("Shared source {cell} returned to monthly claimant allocation after {:.2}/{:.2} kg ore/clay extraction; remaining inventory and depletion retained", used[0], used[1]));
        h.events.last_mut().unwrap().causes.push(cause);
        Ok(())
    }
}
