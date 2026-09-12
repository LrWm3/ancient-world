# Governance and diplomacy

See [civic petitions](civic-petitions.md) for the added institutional representation and remembered-outcome loop.

This extension makes territorial control depend on administration and local consent, and gives trading neighbors a reason to agree to peace. Enable **Governance and diplomacy** after politics in the civilization sidebar, or activate it on an existing political archive:

```sh
mise exec rust@1.89.0 -- cargo run -- --headless \
  --load output/civilization-politics.world --epochs 0 --governance --history-years 100 \
  --save output/civilization-governance.world --history-export output/civilization-governance.history.json
mise exec rust@1.89.0 -- cargo run -- --load output/civilization-governance.world
```

For a new civilization history, combine `--civilizations 8 --society --politics --governance`. Omit activation flags when resuming a world that already has those systems. Activation records a baseline without replaying earlier events or altering population, food or money.

## Territorial administration

Each site has local autonomy, loyalty, unrest, consecutive unpaid months and an administrative crisis duration. A conquest resets local loyalty and introduces unrest. Subsequent policy and economic conditions determine whether control becomes durable.

Monthly administrative wages cost 0.01 abstract currency per resident in the cultural homeland, and 0.04 under foreign administration. Autonomy reduces the required payment by up to half. Councils pay from existing treasury balances, and residents receive exactly that payment in the site's private money. These payments do not create currency; cumulative wages are diagnostic totals, not another stockpile. Underfunding is exposed as consecutive unpaid months rather than accumulating a transferable debt instrument.

Current shortages, remembered household deprivation, crowding/disruption, high taxation and unfunded administration lower loyalty and increase unrest. The [local-pressure specification](governance-pressure.md) gives the equations and controlled comparisons. Paid services and autonomy improve the trajectory. Annual tax receipts decrease by up to 75% with autonomy, so maintaining nominal control through devolved government sacrifices revenue.

A foreign-administered settlement with low loyalty and high unrest can enter a governance crisis. After twelve consecutive crisis months, it ceases recognizing the foreign administrator and restores its existing cultural polity. Its people and private stocks remain in place; no rebel population, money or army appears. This represents administrative secession, not a simulated armed uprising. Granting at least 75% autonomy recognizes local self-rule and prevents this particular independence crisis while retaining nominal affiliation. Domestic revolutions, new breakaway identities, garrisons and coercive suppression are not implemented.

Controller changes immediately affect claims and future taxation. Armies whose objectives change owner follow the existing withdrawal rule. Crisis, autonomy and secession records appear in the history browser with modeled causal references.

## Diplomacy

Each civilization pair has a trust score and a count of successful cross-administration trade deliveries since activation. Delivered cargo raises trust; declarations of war reduce it. This is a first diplomatic-memory model, without cultural affinity, ambassadors or personality-specific relationships.

Neighboring administrations with sufficient trust and repeated trade can automatically conclude a ten-year non-aggression agreement during an annual review. An open surveyed route is required. Agreements cannot overlap or be signed during an active war. Existing ten-year postwar truces remain separate constraints.

The inspector also offers an explicit scenario agreement between two parties through `Generator::sign_nonaggression(a, b, months)`. This represents mutual agreement, rather than a demand with an implemented negotiation dialogue. Terms must be between one and fifty years. `Generator::set_autonomy(site, fraction)` applies the local policy scenario.

Non-aggression agreements block both automatic and scenario war declarations until their expiry month. Expiry is recorded once and opens the possibility of war again; peace is not permanent. Reliable trading partners may renew at a later annual review. The later [negotiated peace extension](negotiated-peace.md) supports active-war settlement, actual installments and breach of payment obligations. Autonomous bargaining, alliance intervention and general tribute treaties remain absent. Existing cargo continues to follow its original transit contract.

## Inspector, persistence and verification

The **Governance and diplomacy** inspector shows loyalty, unrest, unpaid months, cumulative wages, an autonomy control, trust, trade contacts and treaty expiry dates. All records and the diplomatic event cursor persist in an optional version-one governance extension inside the existing version-two world archive. Older political worlds retain their prior behavior until activation.

Validation checks political controller agreement, finite bounded legitimacy values, canonical diplomatic pairs, non-overlapping treaty intervals and historical references. Scenario operations validate a cloned history before committing. Save/resume and different advancement batch sizes must match exactly on the same GPU backend and software version.

Tests cover treaty enforcement and expiry, transactional failure of a prohibited declaration, deterministic continuation, invalid administrative state, and a controlled occupation that secedes under failed governance while self-rule retains nominal control. Existing population, food, C/N/P, water, money and goods conservation checks continue to apply. These are coupled game rules and controlled comparisons; they are not a claim of calibrated historical probabilities.


## Saved-world continuation

A 100-year continuation of the year-200 political sample reached year 300 with 38 settlements and 6,290 residents. Maximum managed-budget relative error was 2.85e-5, below the existing 0.001 threshold. Councils transferred 2,623.48 currency units in administrative wages. This particular continuation's 4,025 cargo arrivals were all within the same administration, so they correctly generated no international trust or treaties. A separate controlled cross-border cargo fixture tests automatic agreement after repeated deliveries, including its causal reference and inventory accounting. The older political archive loads without the extension and can be explicitly upgraded; same-backend checkpoints preserve the new clocks and diplomatic event cursor.
