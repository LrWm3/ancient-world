# Institutional relocation

An existing institution can request a funded move through
`Generator::relocate_institution(id, destination)`. The destination must have two
present members and an occupied settlement, connected by an open direct road.
The move retains identity, membership, knowledge, ownership claims and the same
treasury account. The transport fee transfers real institutional funds to the
origin's town account. This first version uses an abstract paid transport service,
not named porters or a simulation of packing.

Portable, accessible institutional artifacts enter transit; foundations stay
behind. Services stop during transit. Arrival is processed at Open after the due
month and requires the route and destination to remain available. A disrupted
journey waits with its property intact. Arrival resets local readiness, meeting
space and mandate: the new school must establish and fund local services rather
than carrying a building or a completed administration across the map.

The request does not relocate members: it follows members who have already moved.
Branch services and automatic relocation policy remain separate future additions.
The retained old property can support later archaeological or ownership work.
Records and in-transit artifact custody persist in ordinary history archives.

Verification: `cargo test --lib funded_move_preserves_identity_property_and_checkpoint -- --ignored`.
The fixture checks finite funding, duplicate departure rejection, delayed custody,
identity and serialized continuation. It is not a balance claim about natural
institutional survival.
