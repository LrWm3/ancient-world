# Household observance and local congregations

Religion now follows the participating household in personal cultural decisions,
rather than substituting the settlement's majority affiliation. This connects
existing migration, religious diffusion and schisms to actual local practice.

## Personal observance

`Culture::resident_tradition(history, site, actor)` returns the affiliation of a
living, adult, locally present household representative. Travelers, lost
households and actors absent from the town cannot practice there through this
query. Without the household system, the existing local founding-leader proxy
uses the settlement affiliation. No new individual religion stock is introduced.

Pilgrimage uses this affiliation to choose a sacred site, patron-linked departure
cause, tradition-tagged events and the pilgrim's attributed account. Existing
route, flood, duration, provision, offering and work requirements still apply.
Pilgrimage does not convert the actor, recipients or the home town. Practical
learning on a visit remains possible across faiths, when a local teacher holds
knowledge the traveler lacks.

The same household affiliation labels personal cultural actions that already
carry a tradition, including religious institution founding and dedicated
objects. Political administration and the annual town-majority summary remain
separate. Human values are still properties of evolving traditions, not dictated
by patron biology.

## Distinct congregations

The former one-active-institution-per-kind rule meant a majority religious order
could block minority organization. Religious founding now checks for an existing
active institution of the same tradition at that site. Other institution kinds
retain their existing one-per-kind rule.

A congregation requires at least two locally present adult representatives of
its founder's faith. All existing financial and material gates remain: more than
500 money in town cash, 25 transferred into the institutional treasury, two kg
of bricks embodied in the foundation, and 0.2 worker-months from the existing
cultural allowance. No extra subsidy, material, population or labor pool is added.
Names identify both town and tradition, and the inspector labels the tradition.

Annual new recruitment into a religious institution follows that institution's
faith. Existing membership is not erased when a person later changes belief:
organizational affiliation is not forced conversion, and past institutional ties
can persist. Other institutions can still recruit across traditions. Upkeep uses
the existing local staffing and readiness rules; teaching and sponsorship retain
their operational requirements.

Religious bodies still use the shared town economy's established funding rules.
This is not household tithing, private congregational property, interfaith conflict,
legal toleration, or a full voluntary-membership model. Annual deactivation and
leadership succession are unchanged. Town majorities can still change through
recorded household affiliation and contact; founding an order does not recolor
the town immediately.

## Verification and compatibility

The controlled founding fixture starts with a majority order, then assigns two
local households a second existing faith. It checks coexistence, correct founder
and foundation tradition, cash conservation, finite brick withdrawal, unchanged
household affiliations, no duplicate active congregation, and rejection with one
adherent or absent bricks. Annual recruitment and serialized continuation agree.
These are declared affiliation and work-opportunity fixtures, not naturally
emerged conversion histories or a full demographic migration experiment.

The pilgrimage fixture makes the town-majority shrine local and the minority
shrine reachable in a neighboring town. A minority pilgrimage must succeed while
retaining affiliation, recording the right tradition and account, and preserving
food/material ledgers. Flood closure and insufficient work still block it.

```sh
mise exec rust@1.89.0 -- cargo test --lib minority_congregations -- --ignored --nocapture
mise exec rust@1.89.0 -- cargo test --lib pilgrimage_and_recovery -- --ignored --nocapture
```

No archive fields or GPU layouts change. Old institutions retain their identities,
affiliations and memberships. Future personal decisions and recruitment use the
corrected household rule. Exact history continuation across executable revisions
is not promised; same-revision serialized continuation is tested.

### Recorded validation

52 ordinary all-target tests passed, with 121 hardware/long-running tests ignored
by that command. Thirteen GPU-backed tests ran explicitly: the new congregation
fixture, the strengthened pilgrimage fixture, nine culture regressions and two
institutional upkeep fixtures. The religious-relief obligation unit test also
passed. Formatting and Clippy with warnings denied passed. Hardware was the
Quadro RTX 5000 Max-Q on Vulkan. Inspector text was compiled but not manually
exercised in a window.

[Artifact retention policy](evidence/README.md)
preserve the reproduction commands. This is controlled verification of plural
practice and finite founding, not a multi-seed calibration of religious diversity.
