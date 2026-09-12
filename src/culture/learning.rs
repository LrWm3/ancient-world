//! Practical learning progress shared by paid work and bounded informal exposure.
use super::*;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Study {
    pub progress: f32,
    pub source: Option<u64>,
}
/// Conditional outcome of one requested paid lesson, captured before allocation.
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
/// Select one feasible informal encounter. Stable person IDs resolve ties; an
/// in-progress topic wins over starting another. `knows` can read an opening
/// snapshot so newly acquired knowledge cannot relay within this contact pass.
pub(super) fn observation_candidate(
    agents: &[Agent],
    teachers: &[u32],
    students: &[u32],
    month: u32,
    knows: impl Fn(u32, u32) -> bool,
) -> Option<(u32, u32, u32)> {
    let mut students = students.to_vec();
    students.sort_unstable();
    students.dedup();
    for student in students {
        let a = &agents[student as usize];
        if a.last_learning_exposure.is_some_and(|m| m >= month) {
            continue;
        }
        let mut topics: Vec<u32> = (0..12).filter(|t| !a.knowledge.contains(t)).collect();
        topics.sort_by(|a_topic, b_topic| {
            let progress = |t: &u32| a.studies.get(t).map_or(0., |s| s.progress);
            progress(b_topic)
                .total_cmp(&progress(a_topic))
                .then(a_topic.cmp(b_topic))
        });
        for topic in topics {
            if let Some(teacher) = teachers
                .iter()
                .copied()
                .filter(|&teacher| teacher != student && knows(teacher, topic))
                .min()
            {
                return Some((teacher, student, topic));
            }
        }
    }
    None
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
    fn encounters_find_relevant_people_preserve_opening_sources_and_resume_study() {
        let mut agents = vec![agent(); 4];
        for (i, a) in agents.iter_mut().enumerate() {
            a.person = i as u32;
        }
        agents[1].knowledge = BTreeSet::from([4, 9]);
        agents[2].knowledge = BTreeSet::from([4, 9]); // First student has nothing to learn.
        agents[3].studies.insert(
            9,
            Study {
                progress: 0.4,
                source: Some(7),
            },
        );
        let opening: Vec<_> = agents.iter().map(|a| a.knowledge.clone()).collect();
        let before = serde_json::to_value(&agents).unwrap();
        let pick = |agents: &[Agent], teachers: &[u32], students: &[u32]| {
            observation_candidate(agents, teachers, students, 12, |p, t| {
                opening[p as usize].contains(&t)
            })
        };
        assert_eq!(pick(&agents, &[0, 1], &[2, 3]), Some((1, 3, 9)));
        assert_eq!(pick(&agents, &[1, 0], &[3, 2]), Some((1, 3, 9)));
        assert_eq!(serde_json::to_value(&agents).unwrap(), before);
        assert!(pick(&agents, &[0], &[3]).is_none());
        agents[0].knowledge.insert(9); // Learned this pass: not an opening source.
        assert!(pick(&agents, &[0], &[3]).is_none());
        agents[3].observe_topic(9, 12, 0.025).unwrap();
        assert!(pick(&agents, &[0, 1], &[2, 3]).is_none());
        let restored: Vec<Agent> =
            serde_json::from_value(serde_json::to_value(&agents).unwrap()).unwrap();
        assert!(pick(&restored, &[0, 1], &[2, 3]).is_none());
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
