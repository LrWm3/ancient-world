//! Partial practical learning. Only paid instruction/study invokes this path.
use super::*;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Study {
    pub progress: f32,
    pub source: Option<u64>,
}
impl Study {
    fn advance(&mut self, work: f32, curiosity: f32, support: f32) -> bool {
        // Three novice lessons, potentially two with curiosity and experienced
        // instruction. These are game pacing constants, not measured pedagogy.
        self.progress =
            (self.progress + work.max(0.) / 0.3 * (1. + 0.5 * curiosity + 0.5 * support)).min(1.);
        let completed = self.progress >= 1. - 1e-6;
        if completed {
            self.progress = 1.;
        }
        completed
    }
}
impl Agent {
    pub(super) fn study_topic(&mut self, topic: u32, support: f32) -> (bool, f32, Option<u64>) {
        let study = self.studies.entry(topic).or_default();
        let completed = study.advance(0.1, self.traits[3], support);
        (completed, study.progress, study.source)
    }
    pub(super) fn study_source(&mut self, topic: u32, event: u64, completed: bool) {
        if completed {
            self.studies.remove(&topic);
            self.knowledge.insert(topic);
            self.knowledge_sources.insert(topic, event);
        } else {
            self.studies.get_mut(&topic).unwrap().source = Some(event);
        }
    }
    pub(super) fn instruction_support(&self) -> f32 {
        self.instruction_work / (1. + self.instruction_work)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn partial_learning_requires_work_and_preserves_progress() {
        let mut novice = Study::default();
        assert!(!novice.advance(0., 1., 1.));
        assert_eq!(novice.progress, 0.);
        assert!(!novice.advance(0.1, 0., 0.));
        novice.source = Some(7);
        let mut resumed: Study =
            serde_json::from_value(serde_json::to_value(&novice).unwrap()).unwrap();
        assert!(!resumed.advance(0.1, 0., 0.));
        assert_eq!(resumed.source, Some(7));
        assert!(resumed.advance(0.1, 0., 0.));
        assert_eq!(resumed.progress, 1.);
        let mut supported = Study::default();
        assert!(!supported.advance(0.1, 1., 1.));
        assert!(supported.progress > novice.progress);
        assert!(supported.advance(0.1, 1., 1.));
    }
}
