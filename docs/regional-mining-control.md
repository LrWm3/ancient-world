# Regional work plans control shared extraction

A regional survey can now activate a local mining work plan for an existing parent
source through `Generator::activate_regional_mine(region, site, [ore, clay])`.
The limits are kg per month. They constrain the existing GPU extraction kernel;
they are not promised output, a resource import, a new reserve or an additional
labor budget. Production still needs available workers, inputs and economic demand.

Only one regional plan can control a source at a time. Other settlement claimants
receive no extraction allowance from that source until control retires. The owning
site receives at most its local limit, the source's remaining inventory, and the
existing per-month allocation ceiling. Its mine-closure policy still applies.
Setting the local limit to zero suspends extraction without deleting the resource.

```text
regional survey + local work limits
    → canonical source access arbitration
    → existing GPU mining, labor and goods accounting
    → canonical remaining/extracted quantities
    → refreshed regional survey and future town production
```

`set_regional_mining_limit(site, limits)` changes the plan at a completed boundary.
`retire_regional_mine(site)` releases access without adding anything back to the
source. Retired plans retain exact extraction totals, site, source and start/end
months in the archive; these totals are diagnostic subsets of canonical extraction,
not independently spendable stocks. Each plan has its activation event as a stable
identity; limit changes and retirement link to that event. Validation rejects duplicate
identities, invalid event references and extraction history exceeding the source.
Regional surveys include active and retired plans for their parent cells.

Activation rejects unknown or abandoned sites, overlapping control, invalid limits,
out-of-region sites, stale world clocks and changed source inventories or mineral
identities. A survey is never accepted as a replacement inventory. Older archives
have empty active/retired control collections, preserving existing allocation.
An abandoned owner stops extracting through the existing town lifecycle; its control
must be explicitly retired before another claimant resumes. This is an administrative
work-plan decision, not a new legal-ownership system.

## Verification

A small analytical fixture checks two towns sharing nine kg each of ore and clay.
A regional plan permits only one town to draw at most one/two kg per month. Partial
withdrawal leaves the exact unspent quantities canonical. Zero limits stop both
claimants; retirement restores shared access to the depleted balance. Serialized
continuation agrees, and duplicated retired extraction is rejected.

The GPU integration fixture creates a real survey, rejects altered quantities and
stale activation, and runs monthly extraction under 0.1/0.2 kg limits. Twelve single
steps match a saved-and-reloaded twelve-month batch. Made goods and source depletion
respect the limits and existing economic ledgers. Suspension stops extraction;
retirement preserves the balance and enables renewed production. The existing
shared-source survey, market and checkpoint fixture also runs unchanged.

```sh
mise exec rust@1.89.0 -- cargo test --lib resources::tests
mise exec rust@1.89.0 -- cargo test --test resources -- --ignored
```

## Remaining cross-scale work

This activates **source access**, not a complete local simulation. Ownership is at
the parent-source scale; nearby regional cells do not imply separately excavatable
ore bodies. There is no voxel editing, local workforce simulation or regional mine
control panel yet. Production remains the existing monthly town GPU model. Full
batch provenance through mixed recipes, shipments, purchases and archaeological
materials remains open. These limits matter: this change establishes a real local
control-to-production connection without claiming to finish the whole regional world.

Logs and source hashes: [Artifact retention policy](evidence/README.md).
