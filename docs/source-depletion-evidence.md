# Source depletion as historical evidence

The continuity review confirms that regional surveys already carry the canonical
accessible-source stocks, active/retired regional mine controls, surviving site
inventories, objects and processing residues. They do not generate a second ore
reserve or a second ruin inventory.

A remaining useful evidence gap is the depletion boundary itself. A source can
stop supplying ore or clay without a durable source-specific historical marker.
The next increment records the first observed withdrawal that takes a previously
usable pool below one gram. The marker is evidence, not a new inventory or an
instruction to erase the residual stock. Regional source snapshots can retain the
same event reference after abandonment or a new settlement claim.

This deliberately does not claim transformed-goods ancestry. Mixed stores,
smelting, alloying, recycling, trade and ordinary household goods still lose the
identity of their individual source deposits. A future lot/mixture model must
transfer bounded ancestry fractions alongside every relevant physical transfer,
mark old inventory origins unknown, and avoid reconstructing ancestry from the
current local mineral. A new provenance field alone would not solve that gap.

The marker is implemented as `Source.depletion_events[ore, clay]`, with empty
legacy defaults. It records only a new crossing caused by actual withdrawal; an
already depleted imported source receives no invented witnessed event. The site
reference anchors the source geographically and does not credit that town with all
extraction. The lowest site ID gives a stable anchor among shared claimants.

All three canonical-resource fixtures pass. They cover competing claims, unused
allowance return, unchanged conservation, a single ore depletion event while clay
remains available, serialization, no repeated event, old-field import, and a new
settlement claim that cannot replenish the exhausted source or erase its evidence.
Regional snapshots already serialize the canonical `Source` records, so this field
travels with the same source instead of creating an archaeological inventory.
Broader transformed-material ancestry remains unfinished.
