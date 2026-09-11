//! Practical learning progress shared by paid work and bounded informal exposure.
use super::*;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Study {
    pub progress: f32,
    pub source: Option<u64>,
}
/// Conditional outcome of one requested successor lesson, captured before allocation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LessonExpectation {
    pub student: u32,
    pub topic: u32,
    pub opening_progress: f32,
    pub expected_gain: f32,
    pub expected_acquisition: bool,
    pub actual_gain: f32,
    pub actual_acquisition: bool,
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
    pub(super) fn lesson_expectation(&self, topic: u32, support: f32) -> LessonExpectation {
        let mut study = self.studies.get(&topic).cloned().unwrap_or_default();
        let opening_progress = study.progress;
        let completed = study.advance(0.1, self.traits[3], support);
        LessonExpectation {
            student: self.person,
            topic,
            opening_progress,
            expected_gain: study.progress - opening_progress,
            expected_acquisition: completed,
            actual_gain: 0.,
            actual_acquisition: false,
        }
    }

    pub(super) fn study_topic(&mut self, topic: u32, support: f32) -> (bool, f32, Option<u64>) {
        let study = self.studies.entry(topic).or_default();
        let completed = study.advance(0.1, self.traits[3], support);
        (completed, study.progress, study.source)
    }
    /// Informal observation is not paid instruction or additional worker-months.
    /// A single bounded exposure across all topics/channels prevents route fan-out.
    pub(super) fn observe_topic(
        &mut self,
        topic: u32,
        month: u32,
        exposure: f32,
    ) -> Option<(bool, f32, Option<u64>)> {
        if topic >= 12
            || self.knowledge.contains(&topic)
            || self.last_learning_exposure.is_some_and(|m| m >= month)
            || !exposure.is_finite()
            || exposure <= 0.
        {
            return None;
        }
        self.last_learning_exposure = Some(month);
        let study = self.studies.entry(topic).or_default();
        let completed = study.advance(exposure.min(0.05), self.traits[3], 0.);
        Some((completed, study.progress, study.source))
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
    fn agent() -> Agent {
        Agent {
            person: 0,
            traits: [0.; 6],
            skills: [0.; 4],
            occupation: String::new(),
            goal: String::new(),
            knowledge: BTreeSet::new(),
            known_places: BTreeSet::new(),
            knowledge_sources: BTreeMap::new(),
            studies: BTreeMap::new(),
            instruction_work: 0.,
            last_learning_exposure: None,
            last_campaign: None,
            relations: BTreeMap::new(),
            actions: 0,
        }
    }
    #[test]
    fn informal_exposure_is_partial_bounded_and_combines_with_paid_study() {
        let mut a = agent();
        let first = a.observe_topic(4, 12, 0.025).unwrap();
        assert!(!first.0 && first.1 < 0.1);
        a.study_source(4, 7, false);
        assert!(a.observe_topic(5, 12, 0.05).is_none());
        let mut restored: Agent =
            serde_json::from_str(&serde_json::to_string(&a).unwrap()).unwrap();
        assert!(restored.observe_topic(4, 12, 0.05).is_none());
        let (_, progress, source) = restored.observe_topic(4, 24, 0.025).unwrap();
        assert!(progress > first.1);
        assert_eq!(source, Some(7));
        let (_, paid, _) = restored.study_topic(4, 0.);
        assert!(paid - progress > progress - first.1);
        assert_eq!(restored.instruction_work, 0.);
        for month in 25..40 {
            let (done, _, _) = restored.observe_topic(4, month, 0.05).unwrap();
            restored.study_source(4, 8, done);
            if done {
                break;
            }
        }
        assert!(restored.knowledge.contains(&4));
        assert_eq!(restored.knowledge_sources[&4], 8);
        assert!(restored.observe_topic(4, 41, 0.05).is_none());
    }
    #[test]
    fn lesson_projection_is_read_only_and_partial_progress_changes_completion() {
        let mut a = agent();
        a.studies.insert(
            4,
            Study {
                progress: 0.8,
                source: Some(3),
            },
        );
        let before = serde_json::to_value(&a).unwrap();
        let expected = a.lesson_expectation(4, 0.);
        assert_eq!(serde_json::to_value(&a).unwrap(), before);
        assert!((expected.expected_gain - 0.2).abs() < 1e-6);
        assert!(expected.expected_acquisition);
        assert_eq!(expected.actual_gain, 0.);
        let (completed, progress, _) = a.study_topic(4, 0.);
        assert!(completed);
        assert!((progress - expected.opening_progress - expected.expected_gain).abs() < 1e-6);
    }
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
