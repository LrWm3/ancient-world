# Household affiliation for individual cultural participants

Adult family members were eligible for the rotating cultural actor selection, but
`resident_tradition` searched households only by head. Non-head actors could have
a present teacher and available institutional knowledge yet fail the affiliation
check before a study request was emitted. Other cultural actions used the same
lookup.

The lookup now uses `History::person_presence`, the existing shared identity and
household resolver. A candidate must still pass `site_people` eligibility and be
resident at the requested site. Their affiliation comes from their actual
household's `household_faith` entry. This does not create individual conversion
state or assign the town majority's tradition. Missing affiliation remains unknown.

Household movement preserves this association at the destination. Death, duty
absence and wrong-site requests do not confer cultural eligibility. Existing
head-based behavior and the no-society site-affiliation path remain available.
No new archive field or population authority is introduced.

The controlled fixture introduces a sparse adult family identity without making
it a household owner. It checks a minority household tradition, a qualified local
teacher, rotation to that student and emission of an actual study request. It
also checks serialized culture, movement, expedition absence, death and missing
faith records. It tests eligibility, not completion without work or materials.

The previous three-seed lesson-funnel sample did not observe this exclusion among
its selected source-qualified students; it therefore cannot predict the magnitude
of this correction in long histories. Broader cultural participation and balance
still need measurement. Opportunity-aware actor selection and working-core
readiness are separate unresolved changes.

Verification: all three targeted GPU checks (adult-kin affiliation/study,
existing institutional learning, and pilgrimage/recovery accounting) passed.
127 regular library tests passed, with 110 hardware tests ignored in that run.
All-target Clippy passed with warnings denied, and frozen monthly/batched/
checkpoint equivalence passed on seeds 17/81/256. Generated logs remain under
ignored `output/kin-faith-*.log`. The first fixture attempt omitted enabling
genealogy; that setup was corrected before these successful checks.
