# Evolving naming vocabularies

A civilization's naming vocabulary can acquire eponyms: a locally associated person,
town, institution, patron or object supplies a new word for a concept. Other
civilizations can borrow these words through route contact. This only changes
future name generation, not production, beliefs, political allegiance or existing
entity labels. It is a fictional naming convention, not a language-acquisition model.

## Three active options

Each changed concept has at most three pronounced forms. The original word begins
as the oldest option. New distinct forms enter at the end; a fourth removes the
oldest. Selection weights are 1, 2 and 4 from oldest to newest (1:2 with two options).
Thus a new word is favored without instantly replacing everything. An old form
remains available until displaced, rather than expiring on a timer. Duplicate
exposure does not add an entry or refresh its age.

Names choose forms using their stable entity key, meaning and word slot. A loan or
proper-name fragment is already pronounced, so it does not undergo the recipient's
sound changes again. The linking particle is excluded from evolution. Descriptive
collision bynames use active lexical options too.

Every newly coined name stores the actual lexical choices and their provenance.
That snapshot survives retirement of the word. Previously registered labels are
returned verbatim; neither towns nor people get retroactive renamings. Archived
proto-roots remain available as etymological baselines even after leaving active
use for a concept. Language display names remain unchanged.

## Local sources

Annual updates allow at most one adoption per civilization. A deterministic 20%
annual opportunity considers borrowing; otherwise a separate one-in-seven gate
considers local eponyms (about 11.4% of all years). An opportunity with no eligible
source makes no change. These rates and thresholds are game conventions:

- An occupied town at least five years old, with at least 1,000 kg cumulative output
  of the associated timber, metal, ceramics, grain, cloth or fish product.
- An active local institution at least five years old with at least 50 cumulative
  expenditure, associated with sanctuary, market, craft or learning by its purpose.
- A patron whose recorded service has ended, associated with guiding.
- A civilization's named actor with at least five recorded actions, associated with
  their recorded farming, navigation, craft or teaching occupation. The threshold
  measures activity, not proof of excellence in that profession.
- A surviving, accessible object with a recorded knowledge topic, associated with
  books. Lost, destroyed and remotely held objects are excluded.

One eligible source/concept pair is selected; it contributes a non-particle word
fragment of up to 16 characters. This is not a fame or prestige model. Different
produced materials can give the same town several candidate associations, and
there is no claim that population size determines actual linguistic influence.

## Borrowing

An open, unflooded society route between occupied towns of different civilizations
provides a contact observation. Multiple routes do not multiply annual credit.
After three consecutive annual observations, the neighbor's latest eponymic forms
become eligible for borrowing. An annual observation without that route resets
contact credit; temporary closures between annual observations are not integrated.

This uses route access as an opportunity for contact, not measured trade volume or
a model of bilingual speakers. Remote contact without a qualifying route, conquered
language minorities, and frequency-weighted cargo exposure are not implemented.
Borrowing selects among eligible neighbor/concept pairs and retains the original
namesake plus the immediately transmitting civilization. Subsequent borrowing can
carry the word onward, but all civilizations read the previous annual vocabulary
snapshot: a word cannot traverse multiple edges within one annual update.

## Records and explorer

`Language.lexicon` stores active options; `contact_years` and `lexicon_month` preserve
contact memory and guard against processing an annual boundary twice. Missing old
archive fields default to empty state without inventing prior adoptions.
`NameRecord.words` stores the exact word choices; old records may have no such
snapshot. Adoption events retain the source entity and civilization IDs.

The civilization's naming panel exposes current words, selection weights, adoption
months, sources and borrowing origins. Name hover text includes the word choices
used when that name was created, even if the active vocabulary has since changed.

## Checks

The weighted fixture draws 7,000 choices from three words and observed counts
1,002 / 2,009 / 3,989, close to its configured 1:2:4 ratio. It verifies duplicate
suppression, oldest-option removal, existing-name stability and deterministic
serialization continuation.

A controlled three-civilization GPU history fixture uses two declared route edges.
It compares open and closed contact, requires at least three annual observations,
rejects same-year two-hop propagation, checks unchanged town labels, repeated
boundary idempotence and serialized continuation. This isolates the lexical
mechanism; it does not evaluate route construction itself.

The existing seeds 17, 81 and 256 naming fixture checks ordinary ten-year histories,
valid event references and seed 17's full-history save/resume versus monthly steps.

```sh
mise exec rust@1.89.0 -- cargo test --lib naming -- --include-ignored --nocapture
```

Final evaluation: the ten-year worlds produced 5, 1 and 3 evolved civilization/concept
entries across seeds 17, 81 and 256 respectively. Several civilizations had none;
rare adoption is intentional. These totals do not imply natural borrowing occurred
in those small worlds; the controlled route fixture establishes that mechanism.
All eight naming tests passed, including the two GPU fixtures; all 50 ordinary
library tests passed (51 hardware tests ignored by that command). Clippy, formatting
and the source-artifact check passed. The first integrated run exposed missing
civilization/site subject validation for adoption events; those reference types
were added and the complete naming suite passed afterward.
