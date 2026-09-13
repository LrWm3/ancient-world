# Merchant crew productivity

Previously completed merchant work affected hiring priority but not service output.
Named crew assignments now preserve a service-rate bonus from the worker's completed
port-service experience at reservation:

```
bonus = 0.50 * completed_worker_months / (12 + completed_worker_months)
service_equivalent = actual_crew_time * (1 + bonus)
```

Twelve completed worker-months produce a 25% bonus; experience approaches a 50%
ceiling. These are game settings, not measured shipping productivity. New experience
accrues only from actual completed time and cannot improve its own current assignment.

## Time, service and cargo remain different quantities

Paid/committed/used worker-months remain unchanged in personal ledgers, payroll,
external labor reservations and aggregate/individual resolution receipts. Fleet
`work()` still returns actual time. `capacity()` incorporates the frozen bonus,
then applies the existing 250 kg per-vessel ceiling. Hull material and port assets
still impose their existing limits. Capacity does not create cargo inventory.

This benefits partially staffed fleets. A full complement still reaches the same
hull ceiling: this pass does not reduce hiring targets or replace crew demand
forecasts with productivity-adjusted labor requirements. Wages remain tied to time,
not to the service multiplier.

New dispatch capacity uses the settled crews. At next month's Open, the preceding
paid interval still determines cargo progress. Both endpoints retain their capacity
constraints, and voyage clocks cannot advance more than one interval per month.
A crew member lost between reservation and settlement retains prepaid wages under
the existing rule, but contributes neither time nor bonus capacity.

The assignment's `productivity_bonus` persists with receipts. Legacy assignments
default to zero, preserving their already prepaid baseline. Hiring, absence,
settlement, checkpointing and disabling named participation use the same records.

## Remaining occupation coverage

Workshops and merchant crews now have distinct bounded productivity effects.
Agriculture, forestry, mining and construction attendance still need their own
productive-work conversion and verification. Their GPU labor accounting must not
mistake effective work for actual worker-months. This change does not convert them.

## Verification (2026-09-12)

Seven vessel tests pass, including GPU fixtures. Matched named-crew cases on seeds
17, 81 and 256 use identical scarce time and wages; 12 months of prior experience
produces a 1.25 capacity ratio with unchanged actual work and conserved money.
Saved restoration and repeated settlement preserve the bonus and receipts.
Controls cover no funds, unavailable workers, absence after hiring, a fully staffed
hull ceiling, and legacy receipts without the bonus field. Existing cargo endpoint
and completed-interval tests pass.

Ordinary library suite: 150 passed, 130 ignored. All-target Clippy with warnings
denied passes. These are controlled fixtures, not long-history market calibration;
we have not established the effect on century-scale trade, population or inequality.
