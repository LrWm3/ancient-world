# Institutional allocation: controlled scarcity

This is game-balance evidence, not a historical or sociological calibration.
`Stable` remains the default. `Rotating` is an opt-in alternative within each
institutional action class; it does not change the cultural grant or election-first
priority. See [the implementation boundary](governance-duty-audit.md).

## Twelve-quarter fixture

Seed 42, terrain 32/ecology 16, five starting civilizations, Quadro RTX 5000.
Four test schools share two eligible local representatives and equal opening
readiness (0.5). Each receives a declared treasury transferred from town finance.
Their upkeep requests are 0.025 worker-months. Population, food and environmental
exposure remain fixed: this isolates service allocation, not whole-history balance.

| Quarterly allowance | Priority | Work after 12 quarters, per institution | Final readiness | Operational institutions |
|---:|---|---|---|---:|
| 0.025 | Stable | 0.300, 0, 0, 0 | 1, 0, 0, 0 | 1/4 |
| 0.025 | Rotating | 0.075 each | 0.14 each | 0/4 |
| 0.050 | Stable | 0.300, 0.300, 0, 0 | 1, 1, 0, 0 | 2/4 |
| 0.050 | Rotating | 0.150 each | 0.74 each | 4/4 |

Both policies consume identical total labor at a given allowance and preserve total
money exactly in this fixture. Every quarterly reserved plan also resumes identically
through serialization. A separate test reverses input request order and verifies
that each policy produces the same ordered requests; rotation gives each institution
first access three times in twelve quarters with a fixed eligible request set.

These results support keeping a choice of policies. Rotation reduces permanent
exclusion, but spreading insufficient service across all institutions can leave
none operational. It is not a universal improvement. Conversely, concentrated
priority can unnecessarily exclude viable institutions when enough total service
exists to sustain broader participation. Changing membership, finances, buildings,
request sizes or the eligible request set can change these outcomes.

## Whole-history diagnostics

The balance runner accepts `--rotating-institutions`. Its monthly observations
accumulate requested/granted/used institutional work separately for elections and
upkeep. It also counts site-quarters with multiple requests in each class, and those
with an aggregate grant shortfall. Shortfall counts include unavailable participants;
they must not be interpreted solely as a shortage of town labor.

These counts establish whether a policy comparison encountered competition.
Population differences alone cannot show which allocation mechanism caused them.
Raw run output belongs under ignored `output/`.

Verification also includes 115 regular library tests (105 hardware tests skipped),
four hardware institution fixtures, and full frozen-history monthly/batched/
checkpoint equivalence on seeds 17, 81 and 256. Seed 256 enables rotation; the
other arms retain their prior policy. This checks continuation, not balance.
