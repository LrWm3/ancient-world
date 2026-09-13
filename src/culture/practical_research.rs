//! Bounded local experimentation, sharing named cultural work and knowledge provenance.
use super::*;
const RESEARCH_PRIORITY_QUARTERS: u32 = 2;
const MIN_QUOTE: f32 = 0.01;
const EXPERIMENT_WORK: f32 = 0.1;
const EXPERIMENT_MATERIAL_KG: f32 = 0.1;
const FULL_RESEARCH_WORK: f32 = 3.;
const MAX_TREASURY_SHARE: f64 = 0.01;
const WAGE_FOOD_KG_PER_WORKER_MONTH: f64 = 18.;
// Topic, prerequisite, trial material, product motivating research.
const TECHNIQUES: [(u32, u32, &str, &str); 3] = [
    (5, 0, "flax", "cloth"),
    (4, 0, "ore", "metal"),
    (6, 0, "wheat", "preserved_food"),
];
impl Culture {
    pub(super) fn continuing_researcher(
        &self,
        h: &History,
        site: u32,
        people: &[u32],
    ) -> Option<u32> {
        if !self.practical_research
            || !(h.month / 3 + site).is_multiple_of(RESEARCH_PRIORITY_QUARTERS)
        {
            return None;
        }
        people
            .iter()
            .copied()
            .filter_map(|person| {
                let topic = self.experiment_candidate(h, site, person)?;
                let progress = self.agents[person as usize].studies.get(&topic)?.progress;
                (progress > 0.).then_some((person, progress))
            })
            .max_by(|a, b| a.1.total_cmp(&b.1).then(b.0.cmp(&a.0)))
            .map(|v| v.0)
    }
    pub(super) fn experiment_candidate(&self, h: &History, site: u32, actor: u32) -> Option<u32> {
        if !self.practical_research {
            return None;
        }
        let a = self.agents.get(actor as usize)?;
        let s = &h.sites[site as usize];
        let catalog = h.economy_catalog.as_ref()?;
        let soc = h.society.as_ref()?;
        soc.households.iter().find(|hh| {
            hh.head == actor
                && hh.site == site
                && hh.vacant_since.is_none()
                && !soc.relocation.away(hh.id)
                && soc
                    .household_economy
                    .as_ref()
                    .is_some_and(|e| e.accounts.get(hh.id as usize).is_some())
        })?;
        TECHNIQUES
            .iter()
            .filter_map(|&(topic, pre, input, output)| {
                let material = catalog.index(input)?;
                let product = catalog.index(output)?;
                let cost = self.experiment_cost(h, site, material);
                (!a.knowledge.contains(&topic)
                    && a.knowledge.contains(&pre)
                    && (s.economy.management[3] as u32 & (1 << topic)) == 0
                    && s.economy.targets[product] > s.economy.goods[product]
                    && s.economy.goods[material] >= EXPERIMENT_MATERIAL_KG
                    && soc.councils[h.controller(site) as usize].treasury * MAX_TREASURY_SHARE
                        >= cost)
                    .then_some(topic)
            })
            .max_by(|a_topic, b_topic| {
                let progress = |t: &u32| a.studies.get(t).map_or(0., |v| v.progress);
                progress(a_topic)
                    .total_cmp(&progress(b_topic))
                    .then(b_topic.cmp(a_topic))
            })
    }
    fn experiment_cost(&self, h: &History, site: u32, material: usize) -> f64 {
        let e = &h.sites[site as usize].economy;
        WAGE_FOOD_KG_PER_WORKER_MONTH
            * f64::from(EXPERIMENT_WORK)
            * f64::from(e.prices[crate::economy::FOOD].max(MIN_QUOTE))
            + f64::from(EXPERIMENT_MATERIAL_KG) * f64::from(e.prices[material].max(MIN_QUOTE))
    }
    pub(super) fn experiment_request(
        &self,
        h: &History,
        site: u32,
        actor: u32,
    ) -> Option<(&'static str, f32)> {
        self.experiment_candidate(h, site, actor)
            .map(|_| ("practical experiment", EXPERIMENT_WORK))
    }
    pub(super) fn conduct_experiment(
        &mut self,
        h: &mut History,
        site: u32,
        actor: u32,
        remaining: &mut f32,
    ) {
        let Some(topic) = self
            .work_plans
            .get(site as usize)
            .filter(|p| p.month == h.month && !p.experiment_done)
            .and_then(|p| p.experiment_topic)
        else {
            return;
        };
        if *remaining < EXPERIMENT_WORK
            || !self.work_allowed(site, "practical experiment")
            || self.experiment_candidate(h, site, actor) != Some(topic)
        {
            return;
        }
        let catalog = h.economy_catalog.as_ref().unwrap();
        let material = catalog
            .index(TECHNIQUES.iter().find(|t| t.0 == topic).unwrap().2)
            .unwrap();
        let composition = catalog.composition(material);
        let cost = self.experiment_cost(h, site, material);
        let council = h.controller(site) as usize;
        let wage = WAGE_FOOD_KG_PER_WORKER_MONTH
            * f64::from(EXPERIMENT_WORK)
            * f64::from(h.sites[site as usize].economy.prices[crate::economy::FOOD].max(MIN_QUOTE));
        let soc = h.society.as_mut().unwrap();
        let hh = soc
            .households
            .iter()
            .find(|hh| hh.head == actor && hh.site == site)
            .unwrap()
            .id as usize;
        // Material payment goes to its actual owner (town); wage to the researcher.
        let paid_material = crate::household_economy::deposit(
            &mut h.sites[site as usize].economy.finance[0],
            cost - wage,
        );
        soc.councils[council].treasury -= wage + paid_material;
        let account = &mut soc.household_economy.as_mut().unwrap().accounts[hh];
        account.cash += wage;
        account.wages += wage;
        let e = &mut h.sites[site as usize].economy;
        let old = e.goods[material];
        e.goods[material] = (old - EXPERIMENT_MATERIAL_KG).max(0.);
        let used = old - e.goods[material];
        e.used[material] += used;
        e.reserves[3] += used;
        for (k, r) in composition.iter().enumerate() {
            e.detritus[k] += used * r;
        }
        *remaining -= EXPERIMENT_WORK;
        self.work_plans[site as usize].experiment_done = true;
        let a = &mut self.agents[actor as usize];
        let study = a.studies.entry(topic).or_default();
        let previous = study.source;
        study.progress =
            (study.progress + EXPERIMENT_WORK / FULL_RESEARCH_WORK * (1. + a.traits[3])).min(1.);
        let progress = study.progress;
        let completed = progress >= 1.;
        self.log(h,if completed {"practical_research_completed"} else {"practical_research_progress"},site,Some(actor),None,None,previous,
            format!("{} conducted council-funded {} experiments: {:.0}% progress, {:.2} worker-months, {:.3} kg trial material, {:.2} wages and {:.2} material payment. {}",
                h.people[actor as usize].name,TOPICS[topic as usize],progress*100.,EXPERIMENT_WORK,used,wage,paid_material,
                if completed {"The researcher can now use the technique."} else {"No production technique unlocked yet."}));
        let event = h.events.last().unwrap().id;
        self.agents[actor as usize].study_source(topic, event, completed);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires GPU"]
    fn civic_research_requires_funds_material_and_work_and_retains_knowledge() {
        use crate::{
            catalog::Catalog,
            config::Config,
            gpu::{ContextGpu, Generator},
        };
        let mut g = Generator::new(
            pollster::block_on(ContextGpu::headless()).unwrap(),
            Config {
                resolution: 32,
                ecology_resolution: 16,
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(5).unwrap();
        g.enable_society().unwrap();
        g.enable_politics().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        h.sync_culture();
        h.prepare_household_retail();
        h.month = 12;
        let mut c = h.culture.take().unwrap();
        c.practical_research = true;
        for a in &mut c.agents {
            a.knowledge.remove(&5);
            a.knowledge.insert(0);
        }
        let flax = h.economy_catalog.as_ref().unwrap().index("flax").unwrap();
        h.sites[0].economy.management[3] = 0.;
        h.sites[0].economy.targets[18] = 100.;
        h.sites[0].economy.goods[18] = 0.;
        h.sites[0].economy.goods[flax] = 10.;
        let council = h.controller(0) as usize;
        let cash = crate::household_economy::withdraw(&mut h.sites[0].economy.finance[0], 10000.);
        h.society.as_mut().unwrap().councils[council].treasury += cash;
        let plan = c.plan_work(h, 0);
        let actor = plan.actor.unwrap();
        assert_eq!(plan.experiment_topic, Some(5));
        c.work_plans = vec![plan];
        let before = h.economy_residuals();
        let mut no_work = 0.;
        c.conduct_experiment(h, 0, actor, &mut no_work);
        assert!(!c.agents[actor as usize].studies.contains_key(&5));
        let treasury = h.society.as_ref().unwrap().councils[council].treasury;
        h.society.as_mut().unwrap().councils[council].treasury = 0.;
        assert_eq!(c.experiment_candidate(h, 0, actor), None);
        h.society.as_mut().unwrap().councils[council].treasury = treasury;
        h.sites[0].economy.goods[flax] = 0.;
        assert_eq!(c.experiment_candidate(h, 0, actor), None);
        h.sites[0].economy.goods[flax] = 10.;
        for _ in 0..31 {
            if c.agents[actor as usize].knowledge.contains(&5) {
                break;
            }
            c.work_plans[0].month = h.month;
            c.work_plans[0].experiment_done = false;
            let mut work = EXPERIMENT_WORK;
            c.conduct_experiment(h, 0, actor, &mut work);
            assert_eq!(work, 0.);
            if !c.agents[actor as usize].knowledge.contains(&5) {
                let month = h.month;
                h.month = 12;
                assert_eq!(c.continuing_researcher(h, 0, &[actor]), Some(actor));
                h.month = 15;
                assert_eq!(c.continuing_researcher(h, 0, &[actor]), None);
                h.month = month;
            }
            let held = h.money_residual();
            let progress = c.agents[actor as usize].studies.get(&5).map(|v| v.progress);
            let mut duplicate = EXPERIMENT_WORK;
            c.conduct_experiment(h, 0, actor, &mut duplicate);
            assert_eq!(duplicate, EXPERIMENT_WORK);
            assert_eq!(held, h.money_residual());
            assert_eq!(
                progress,
                c.agents[actor as usize].studies.get(&5).map(|v| v.progress)
            );
            h.month += 3;
        }
        assert!(c.agents[actor as usize].knowledge.contains(&5));
        assert!(c.agents[actor as usize].knowledge_sources.contains_key(&5));
        for (a, b) in before.into_iter().zip(h.economy_residuals()) {
            assert!((a - b).abs() < 1e-5, "{a} {b}");
        }
        let encoded = serde_json::to_value(&c).unwrap();
        let decoded: Culture = serde_json::from_value(encoded.clone()).unwrap();
        assert_eq!(encoded, serde_json::to_value(decoded).unwrap());
    }
}
