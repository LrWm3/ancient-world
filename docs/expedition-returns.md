# Expedition collections and local applications

New ecological voyages can bring back typed botanical collections alongside
faultroot resin and mineral specimens. Archaeological voyages can recover several
small kinds of everyday object. These are finite collections and practical game
opportunities, not discoveries that explain humanity's banishment or unlock lost
supertechnology.

## Botanical collections

| Collection | Application after destructive study | Where it goes |
| --- | --- | --- |
| Silver bast fibers | Prepare usable fiber | Existing textile, craft and market inventory |
| Ironberry pigment material | Prepare writing supplies | Existing copying, teaching and scholarly consumption |
| Marsh-thread planting samples | Habitat-gated establishment trial | Existing fiber-crop seed compartment |

Half of each newly collected organic batch is assigned to one of these profiles;
the rest remains available for the existing remedy workflow. Profile is stable for
a source cell and world seed. The accessible organic source baseline and expedition
cargo allowance are unchanged. Typed material is a **subset** of organic cargo,
not another collection added on top. All three currently use the same coarse
organic C/N/P composition. This is a material-accounting simplification, not a
botanical chemistry claim.

Rescue transfers retain typed cargo and its source cell. Lost or rejected cargo
is discarded through the original specimen ledger. Delivery moves typed material
into workshop collection stocks and removes it from ordinary resin stock. Older
archives default to zero typed stock; their existing resin is not reclassified.

A collection can be stored, studied without production, or applied after study.
The expedition explorer exposes these policies and stock/trial/output totals.
The API is `Generator::set_botanical_use(site, kind, Use)`, where kind is 0–2 in
the table order. A policy cannot replace already-reserved current work.

## Work and conservation

Researchers first consume 0.25 kg in destructive trials, using the existing
research reservation and named-participation path. Collection work follows
existing resin/mineral work within that same budget. Each kg studied or processed
costs two worker-months, 0.1 kg tools, and 0.2 kg fuel. Work requests capture maximum
processing quantities; execution cannot exceed them or actual resources.

Applications process at most 0.25 kg per kind per month. Product yield is at most
half the input mass and is further bounded by each output constituent. Unused
C/N/P goes into managed detritus; fuel and tool consumption follow existing
ledgers. A learned method without another batch produces nothing. Stock, receipts,
source references and use choices persist in checkpoints.

Cultivation requires a managed fiber-crop plot, suitable temperature and rainfall,
available water, and nondepleted soil N/P. Successful trials transfer finite seed
mass into the existing flax crop's seed compartment and consume trial water.
Subsequent growth runs through ordinary GPU land, sunlight, nutrient, water and
harvest calculations. This version represents the introduction as a compatible
fiber-crop type with expedition provenance, not a separate species or persistent
cultivar trait vector. It adds neither farmland nor a yield multiplier. Unsuitable
trials consume their samples into detritus and return no viable seed. Invasions,
local propagation of separately identified species and microbial inoculation are
not implemented here.

## Minor archaeology and follow-up voyages

Finds now include ceramic fragments, textile remnants, trade weights and tool
fragments. Each imports one finite 0.125 kg object with the corresponding catalog
material, recorded origin, custodian and provenance. Existing institutional study,
accounts and artifact ownership continue to operate on it. Interpretations remain
attributed and incomplete. The object remains available for preservation and
ownership changes; studying it does not mint ordinary goods or grant technology.

Demonstrated botanical production can motivate an automatic ecological voyage to
its actual known source when local collection stocks and relevant goods are low.
The source must still have accessible organic material. Funding, crew, provisions,
cooldowns and hazards remain authoritative. Departure links to the application
event that justified the request. Urgent remedy and mineral needs keep priority.
This is a supply follow-up, not a global search for an optimal unexplored location.

## Verification

```sh
cargo test --lib
cargo test --lib botanical -- --include-ignored --test-threads=1
cargo test --test discoveries -- --ignored --test-threads=1
cargo test --lib expedition_heritage -- --include-ignored --test-threads=1
cargo clippy --all-targets -- -D warnings
```

Controlled botanical trials use seeds 17, 81 and 256, checking no-work and cold
controls, constituent transfer, finite yields, persistence, and depletion blocking
follow-up requests. The existing discovery suite exercises real collection,
rescue, return, processing and checkpoint accounting. Results and remaining
limitations are recorded below after the run.

### Results (2026-09-11)

- The two return tests passed, including the GPU-backed fixture on seeds 17, 81
  and 256. It uses controlled fertile frontier patches and funded workshops, not
  naturally selected prosperous towns. Collection, duplicate delivery, rescue
  handoff, discarded cargo, real research planning and source/workshop ledgers pass.
- Zero available work leaves collections untouched. Cold planting trials consume
  material but produce no viable seed. Suitable trials create bounded seed stocks;
  fiber and writing outputs plus detritus account for input C/N/P. Serialization
  preserves continuation. Exhausted sources cannot justify botanical follow-ups.
- Tiny transfer checks use an isolated detritus stock: subtracting two large f32
  detritus totals obscured the initial sub-kilogram fixture's signal. Whole-world
  budgets retain their existing scale-aware tolerances.
- The older `tests/discoveries.rs` suite remains failing: three ecological fixtures
  cannot launch a suitable funded voyage after their 240-month setup; the mineral
  fixture never reaches positive phosphorus application. The healthy-town launch
  failure and mineral-production failure both reproduce at pre-change commit
  `6fac126`. These are not counted as successful validation of this expansion.

The new fixtures establish bounded transfers and controlled effects. Natural
collection frequency, long-run research competition and voyage profitability have
not been calibrated. The older voyage fixture prerequisites need a separate review.

The final regular library run passed 103 tests (92 hardware-dependent tests ignored
in that run). The focused archaeological study fixture passed. All-target Clippy
with warnings denied and the source-artifact check passed.
