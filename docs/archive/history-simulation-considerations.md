# History simulation: systems to consider adding

> **Archived design / legacy reference — 2026-09-09.** This preserves the original proposal and its historical limitations; it is not the current implementation status or an active task list. See the [current civilization guide](../civilizations.md) and [documentation index](../README.md).

## Purpose

This note collects possible additions that could make Ancient World's generated history richer without requiring day-to-day simulation of every person. It is a design menu, not a committed roadmap and not a proposal to reproduce Dwarf Fortress feature for feature.

Ancient World's distinctive strength is its physically grounded world: climate, water, ecology, nutrients, resources, production and transport already create material constraints. New history systems should build on those constraints, preserve explicit accounting and produce events whose causes can be inspected.

The preferred model is aggregate and CPU-first. Population cohorts, distributions and prevalence fields can act as proxies for individual behavior. Dense, regular calculations should be represented in a form that can move to the GPU if scale or profiling eventually justifies it. Sparse graphs, unique records and event arbitration should remain on the CPU.

## Design principles

- Prefer conserved stocks, bounded flows and explicit transfers over free statistical bonuses.
- Generate historical events from persistent changes in simulated state, not isolated random rolls.
- Keep authoritative state structured; prose is a presentation of that state.
- Preserve minorities and distributions rather than reducing every settlement to one average value.
- Separate population, culture, religion, organization and political control.
- Use stable integer IDs and deterministic randomness keyed by world seed, system, date and entity.
- Advance systems through completed snapshots: generate proposals, arbitrate conflicts, reserve resources, then commit.
- Keep dense numerical state separate from descriptive metadata and sparse historical records.
- Add semantic depth where it creates consequences for survival, migration, trade, politics or the environment.

## Population proxies

### Cohorts

Represent residents as conserved population stocks rather than ordinary individual agents. Useful dimensions include:

- Age: infant, child, young adult, mature adult and elder.
- Livelihood: farming, herding/fishing, extraction, craft, transport/trade, administration and military service.
- Material condition: secure, strained, deprived and displaced.
- Health: healthy, sick, injured and disabled.

Not every dimension needs to be crossed into one large tensor. Age-by-livelihood cohorts can coexist with separate wealth, health and food-security distributions. Monthly transitions cover birth, aging, recruitment, occupational change, illness, recovery, injury, death and displacement while conserving population.

### Wealth and food-security distributions

Settlement averages conceal inequality and unequal exposure to crisis. Fixed histograms can track population from destitute through elite, plus food-security states from secure through starving. Wages, prices, taxation, inheritance, disaster, relief and ownership move population shares between bins.

These distributions can support histories of impoverishment, elite formation, differential famine mortality, tax pressure and recovery without household-level simulation.

### Accumulating social pressures

Track pressures that rise and decay over time:

- Hunger and malnutrition
- Housing crowding
- Disease burden
- Work exhaustion and unemployment
- Inequality and tax burden
- Physical insecurity and war losses
- Cultural displacement
- Institutional confidence

Persistent combinations should produce consequences. A shortage can increase hunger, reduce health and productivity, raise migration pressure and weaken legitimacy. Recovery should likewise take time.

## Culture, knowledge and belief

### Cultural prevalence

Represent practices, languages, identities, beliefs and skills as population shares at each settlement. Shares change through births, migration, trade contact, institutional teaching, political pressure and the demonstrated utility of a practice. This is a reaction-and-diffusion model: local adoption supplies the reaction, while routes and migrating populations supply diffusion.

This permits mixed settlements, minorities, diasporas, gradual assimilation, revival and regional divergence. Conquest should change administration before it changes culture.

### Cultural production

Consider sparse records for culturally important works and forms:

- Songs, poems, stories and chronicles
- Musical forms, instruments and dances
- Visual and craft styles
- Ceremonies and festivals
- Schools of thought

Ordinary cultural activity remains aggregate. A unique record is created only when a threshold event makes a work historically important. Works can spread through prevalence fields, institutions and travel.

### Knowledge and information flow

Distinguish authoritative events from what populations know about them. Settlement- or institution-level knowledge can use compact states such as unknown, rumored, witnessed and documented.

Travelers, migrants, shipping, trade and written works propagate knowledge. This can create delayed warnings, incomplete maps, competing accounts, reputations and politically useful misinformation without simulating conversations.

Research should have prerequisites, teachers, material requirements and locations. Libraries and archives hold finite copies or access shares; destruction or dispersal should have consequences.

### Religion and organized belief

Extend household faiths into population shares and institutions with:

- Congregations, priesthoods and religious orders
- Temples, shrines, holy sites and festivals
- Conversion, syncretism, schism and persecution
- Religious offices, resources and authority
- Pilgrimage and diaspora links

Belief should remain distinct from ancestry, culture and political membership.

## Organizations and institutions

Introduce a general organization model capable of representing:

- Merchant associations and companies
- Craft guilds
- Military and mercenary companies
- Religious orders
- Scholarly schools
- Diaspora and mutual-aid associations
- Criminal networks
- Political movements

An organization can have membership shares, leaders or offices, wealth, property, skills, home sites, geographic reach, relationships and goals. Organizations may split, merge, migrate, gain influence or disappear. They provide durable collective actors without requiring every member to be individually represented.

Institutions should maintain measurable capacities, such as administration, legitimacy, enforcement, relief, scholarship, commercial trust and religious authority. Capacity consumes labor and material support and decays when neglected.

## Material culture and artifacts

Most goods should remain aggregate inventories. Create unique objects only for exceptional outputs, inherited symbols, discoveries or politically important property.

A unique object may record:

- Material and object type
- Creator, workshop and date
- Cultural style or associated knowledge
- Owner and physical location
- Heirloom, religious, institutional or political claims
- Transfers by gift, inheritance, trade, theft or conquest
- Reputation and referenced historical events

This permits artifacts to cause disputes, diplomacy, pilgrimage, theft or recovery without expanding all manufactured goods into individual items.

## Persistent and differentiated sites

Between an abstract settlement and a fully playable local map, add a persistent site-asset layer:

- Districts and housing
- Farms, workshops and markets
- Roads, bridges and harbor facilities
- Administrative and religious buildings
- Guildhalls, schools and archives
- Walls, forts and other defenses
- Monuments, cemeteries and ruins

Each asset needs only type, capacity, condition, owner, material requirements and construction history. Assets affect production, storage, defense, institutional capacity and disaster recovery. Regional placement and detailed geometry can remain deferred.

Additional site categories could include ports, mining settlements, fortified passes, monasteries, trade posts, seasonal camps, ruined settlements and expedition stations.

## Conflict, diplomacy and intrigue

### More expressive conflict

Extend aggregate warfare with:

- Multiple armies and fronts
- Fortifications, sieges and blockades
- Command structures and military organizations
- Captives, ransom and negotiated release
- Loot and artifact seizure
- Occupation policy and resistance
- Rebellions, civil wars and negotiated settlements
- Persistent battlefields, damaged infrastructure and ruins

Armies remain conserved population and equipment stocks. Travel, recruitment, supply, attrition and casualties must reconcile with settlement inventories.

### Diplomacy

Build beyond bilateral trust and non-aggression agreements toward alliances, tribute, guarantees, access rights, trade concessions, competing claims and negotiated peace. Relationships should be affected by observed deliveries, broken commitments, casualties, shared institutions and incomplete information.

### Intrigue and crime

Potential aggregate or organization-level actions include theft, smuggling, sabotage, corruption, embezzlement, abduction, assassination, coups and framing. These should require actors, capabilities, motives, opportunity and risk. Evidence and knowledge propagate separately from the authoritative event.

This system should begin with organizations and offices rather than an unrestricted simulation of individual conspirators.

## Health and disease

Fixed epidemiological compartments are well suited to aggregate simulation:

```text
susceptible -> exposed -> infectious -> recovered
                              `-> dead
```

Transmission and outcomes can depend on density, nutrition, climate, water quality, immunity, trade, migration and medical knowledge. Disease moves through explicit routes and travelers. Multiple diseases can be catalog entries with fixed traits rather than individually simulated pathogens.

## Migration and collective actors

Calculate migration demand by cohort from hunger, employment, danger, flooding, cultural affinity, destination capacity and route cost. Accepted migrants enter explicit in-transit stocks carrying origin, livelihood composition, cultural shares, health and possessions.

Named collective actors can emerge from materially important groups:

- A displaced farming community
- The dockworkers of a port
- A merchant association
- A military cohort
- A congregation
- An island diaspora

Collectives may split, merge, relocate and become attached to institutions or events. They provide continuity and specificity without individual simulation.

## Ecology and biogeography additions relevant to history

Several ecological extensions would create historical consequences:

- Multiple competing producers per vegetation layer
- Flexible animal-guild diets instead of one fixed prey compartment
- Regional ecotypes and dispersal barriers
- Extinction, invasion and ecological succession
- Domestication and selective adaptation
- Shipping-mediated invasive species
- Hunting and fishing pressure on guild biomass

Full genetic evolution is not necessary. Trait-bearing regional populations and occasional CPU-created lineage records would be sufficient to create endemic species, range shifts and extinction history.

## Geology additions relevant to history

The most useful geological extensions are those that create persistent places, hazards and resource histories:

- Stratigraphic columns with layer thickness and provenance
- Coherent mineral provinces and deposit types instead of isolated probabilities
- Fault, volcano, earthquake and landslide events
- Persistent quarry and mine depletion
- Springs, aquifers and groundwater accessibility
- Caves, karst and other subsurface site opportunities
- Depositional changes that affect rivers, deltas and harbors

Excavation-ready voxels and a mantle-scale plate solver are much larger projects. Persistent regional strata and resource bodies would provide more immediate historical value.

## CPU and future GPU boundary

CPU execution is appropriate at the current settlement scale. GPU migration should follow profiling rather than the mere addition of more systems. It becomes attractive with thousands of settlements, many cohort compartments, large route graphs, dense regional population fields or large parallel scenario suites.

Good candidates for eventual GPU execution are:

- Carrying-capacity and site-suitability evaluation
- Cohort demographics and health transitions
- Production, consumption and demand calculations
- Wealth, culture and faction-share transitions
- Disease propagation
- Bulk route scoring and supplier search
- Migration and trade proposals

The CPU should continue to own:

- Sparse entity and organization graphs
- Route and settlement topology changes
- Treaties, wars and unique-object records
- Proposal arbitration and resource commitment
- Historical event storage and causal indexes
- Names and prose presentation

Once a dense subsystem moves to the GPU, its state should remain resident for a complete monthly batch. Read back compact settlement summaries, accepted proposals, threshold crossings, conservation totals and candidate events rather than full state buffers.

## Suggested implementation order

### First: richer aggregate population

1. Age-by-livelihood cohorts
2. Wealth and food-security histograms
3. Persistent social pressures
4. Conserved cohort migration
5. Cultural and faction prevalence vectors

This foundation can already produce inequality, labor transitions, diaspora, assimilation, factional change, recovery and collapse.

### Second: durable collective structure

1. General organizations
2. Institutional capacities
3. Differentiated site assets
4. Knowledge and information propagation
5. Organized religion

### Third: sparse historical identity

1. Important cultural works
2. Unique artifacts and claims
3. Organization-level intrigue
4. Richer diplomacy and conflict consequences
5. Persistent ruins, battlefields and resource sites

### Later or optional

- Regional ecotypes and speciation
- Excavation-ready geology
- Detailed local site geometry
- Individual agents beyond exceptional historical figures

## Example causal history

The goal is for histories to arise from connected state changes:

```text
multi-year drought
  -> poor harvest and rising grain prices
  -> impoverishment of land-poor households
  -> migration appeal carried to a port
  -> mutual-aid association funds relief shipment
  -> storm damage closes the harbor
  -> prolonged hunger weakens council legitimacy
  -> merchant faction demands emergency trade concessions
  -> agreement restores imports but transfers political influence
  -> displaced farmers establish a diaspora community abroad
```

Every arrow should correspond to an inspectable stock, transition, decision or event reference. That causal material history is the intended advantage of these systems over disconnected procedural lore.
