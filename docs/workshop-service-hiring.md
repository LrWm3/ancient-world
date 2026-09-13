# Workshop hiring after essential-service commitments

The workshop hiring ceiling previously took the minimum of last month's whole
craft allocation and the current shared-service allowance. The latter subtracts
public commitments, but the former did not. When crafting was the smaller pool,
firms could therefore prepay attendance that public work would consume first.
Water operation was absent from this hiring bound as well, although GPU execution
operates installed waterworks before construction and recipes.

Hiring now subtracts current public commitments from the previous craft ceiling,
takes the minimum with the remaining workforce-based service allowance, then
subtracts installed water-service demand and floors the result at zero. All
workshop families share that remaining ceiling through the existing allocator.
Actual orders, input feasibility, lease capacity, cash and named attendance still
apply. This changes the request before payroll; wages already earned are not
refunded and no money is created.

Water operation retains its existing execution priority. Its rate is now a shared
Rust/WGSL parameter in the production module, with the same value (0.001
worker-months per served resident). The hiring forecast conservatively assumes
installed service up to current population can operate. Weather, storage wear or
water shortage can subsequently reduce actual service work. The unused allowance
is not reassigned retrospectively. The previous craft allocation is still a
ceiling rather than an exact current GPU forecast; this does not promise full
employment or eliminate every cause of paid but unproductive work.

## Controlled checks

A paired hiring test creates a funded operator with metal stocks and installed
workshop equipment. Of one prior craft worker-month, public services reserve 0.6.
Without waterworks the actual paid shift is 0.4. With capacity for 100 residents it
is 0.3. Both match the analytical remainder and preserve the money ledger.
The hardware fixture passes (40.46 s including cold setup).

Existing automatic service procurement/checkpoint continuation passes (1.72 s),
as does prepaid capacity/GPU execution/checkpoint equivalence (2.30 s). These are
focused checks, not a claim that the full history test suite has been rerun.

## Matched 50-year comparison

Seeds 1024, 256 and 409 use frozen 32/32 founding checkpoints and the identical
native argument arrays from `output/building-reservation-screen`, changing only
the binary and export destination. Credit/issuance are off; service procurement
at 0.25, demand/contract staffing, inheritance, named office service and abandoned
stock recovery are on. All three new runs complete. Source-only results are
summarized here; raw files and executable are in ignored
`output/service-hiring-screen`. Build and strict all-target Clippy pass, as do the
two existing CPU labor tests. Compilation overlapped the first run; no isolated
performance claim is made.

| Seed | Population before → after | Paid work before → after | Completed operator work before → after | Operating margin before → after | Terminal need-weighted hunger before → after |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1024 | 164.042 → 164.045 | 6.863 → 6.750 | 6.737 → 6.737 | 43.789 → 47.266 | 0.0599 → 0.0487 |
| 256 | 319.626 → 322.662 | 12.029 → 3.787 | 8.262 → 3.782 | -55.605 → 17.143 | 0.0513 → 0.0481 |
| 409 | 329.123 → 317.718 | 214.886 → 48.258 | 118.153 → 48.034 | -2688.858 → 228.843 | 0.0648 → 0.0746 |

Attendance completion exceeds 99.5% in every new run. All cumulative operator
operating margins are positive without raising fees, issuing money or refunding
already paid wages. Maximum absolute relative cash residual is 1.91e-7.

This supports the diagnosis that overlapping service/hiring allowances were a
major cause of paid idle work. It does not prove that every individual firm is
profitable or that workshop demand and circulation are now satisfactory. Work
volume falls sharply in two seeds; seed 409 loses population and ends with worse
food access. The controlled fixture isolates the hiring mechanism; the diverged
50-year outcomes cannot attribute every later change to it. Next compare useful
work demand, current versus previous craft allocation, service expenditure and
household access rather than restoring the overhiring that concealed these limits.
