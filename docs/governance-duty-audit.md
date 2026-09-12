# Governance and institutional participation audit

Inspected at `a35d491`. This is a code-boundary audit, not a claim that named
administrative work is implemented. It addresses the personal-duty review in the
integration worklist while retaining the implementation gaps below.

## Current authority and accounting

| Duty | Current cost and timing | Named participation gap |
|---|---|---|
| Local office coordination | `office_capacity` in `src/offices.rs` derives capacity from a valid holder and knowledge breadth. Vacancies retain 0.5 capacity. Governance observes it in Respond. | No personal commitment or completed-work receipt. A holder can be eligible while fully committed elsewhere. |
| Administrative funding and tax policy | `governance_month_observed` in `src/governance.rs` transfers existing council money to town finance, tracks funding, and changes loyalty/tax effects. | Payment does not establish an administrator's attendance; there is no corresponding personal-time reservation. Do not mistake money transferred to town finance for wages already delivered to a particular person. |
| Institutional upkeep and building work | `maintain_institutions` in `src/institution_capacity.rs` consumes the cultural labor budget, institution money and finite building inputs; staff must include present adult members. Readiness and physical condition affect later operation. | Staffing is a count, not the set of people doing the work. There is no maintenance-specific participant assignment per institution. |
| Institutional succession | `src/institution_succession.rs` uses present local representatives for a ballot and spends cultural work to convene it. | The ballot electorate is not a reserved work team. Political preference does not itself require labor, but organizing the election needs a specific accountable participant. |
| Institutional teaching | Source-specific cultural study requests retain teacher/institution IDs; reservation includes the teacher, and execution measures actual learning. | The wider cultural bundle still shares one team and grant rather than assigning each action its own workers. |
| Office campaigning | Existing cultural actions consume cultural work and money and produce political history. | Campaign participation does not cover subsequent routine office duties. |

`reserve_cultural_work_capped` in `src/culture.rs` builds one team from the
selected cultural actor, successor, institutional teacher and local recovery
workers. `settle_participation` charges the bundle's actual total once. This
bounds named cultural work overall, but does not guarantee that every upkeep
recipient supplied a member. Adding all institutional members to this same team
would make absent members block unrelated actions and is not an adequate repair.

## Integration boundaries to preserve

- Keep the five monthly phases. Opening policy/preparation remains separate from
  the later governance response. Do not retroactively apply a new officeholder's
  time to production already completed this month.
- Reserve routine administrative work before production if it is to compete with
  production. Both a personal grant and an existing town labor allowance must be
  debited; person capacity alone is not a second labor supply.
- Record which office and holder were reserved. Validate them at execution;
  succession or departure after reservation must not silently transfer work to
  the replacement. Completed work can inform the same month's later governance
  response; a new appointment can wait until the next reservation window.
- Keep existing money transfers single-owned. The administrator's assigned work
  must not create a second copy of funding, wages or tax proceeds.
- For institutions, split upkeep into institution-specific requests inside the
  existing cultural allocation. Reserve suitable members per request, retaining
  ordinary teacher/learner assignments. Charge the shared cultural allowance once,
  not once for the existing bundle and again for each new action.
- Keep aggregate mode and old-archive interpretation explicit. Old tenure or
  membership records are not evidence of historical hours worked.

## Next implementation and acceptance boundaries

Start with dated office service requests and grants, using an explicit policy for
competition with other labor. Capacity should read delivered service rather than
merely an office title, while preserving the separately declared unnamed-staff
baseline until that baseline is converted too. Compare aggregate requested work,
named attendance and actual administrative output separately. Follow with
institution-specific upkeep assignments rather than expanding the generic team.

Required checks include a fully committed holder, absent/dead holder, insufficient
shared labor, changed jurisdiction/holder after reservation, partial completion,
no duplicate funding or labor debit, and monthly/batch/checkpoint consistency.
For institutions also test two organizations sharing one member, an unrelated
absent teacher, and a funded building with no available maintenance member.
Scarcity comparisons must measure displacement of care, teaching and production;
reserving government work first is a policy choice, not automatically fair.

The audit identifies actual gaps; it does not close named administration,
institutional action assignments, multi-site branches or multiple officer roles.
