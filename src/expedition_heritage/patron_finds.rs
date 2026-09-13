//! Patron-associated finds are interpretations, not proof of the patron's presence.
pub(super) struct Item {
    pub id: &'static str,
    pub patron: &'static str,
    pub kind: &'static str,
    pub material: &'static str,
    pub kg: f32,
    pub description: &'static str,
}
pub(super) static ITEMS: &[Item] = &[
Item { id: "human_guide/staff", patron: "human_guide", kind: "strange wooden staff", material: "wood", kg: 1.2, description: "A crooked wooden staff has a polished handhold and shallow marks resembling the guide's remembered gestures." },
Item { id: "human_guide/flute", patron: "human_guide", kind: "flute of unknown material", material: "tools", kg: 0.18, description: "A small flute is made from a smooth, unfamiliar substance that cannot yet be identified. Its finger spacing recalls the guide's hands." },
Item { id: "human_guide/fabric", patron: "human_guide", kind: "guide-like fabric remnant", material: "cloth", kg: 0.08, description: "A scrap of fabric resembles the clothing preserved in accounts of the guide, including an unusual folded edging." },
Item { id: "reed_crowned/crown", patron: "reed_crowned", kind: "braided reed crown", material: "wood", kg: 0.12, description: "A ring of woven reeds echoes the navigator's sensing crown." },
Item { id: "reed_crowned/track", patron: "reed_crowned", kind: "many-legged track tablet", material: "pottery", kg: 0.3, description: "A fired tablet records a procession of many-legged tracks at a shoreline." },
Item { id: "reed_crowned/whistle", patron: "reed_crowned", kind: "reed signaling whistle", material: "wood", kg: 0.04, description: "A split reed whistle produces several thin notes, resembling remembered shore signals." },
Item { id: "glassback/plate", patron: "glassback", kind: "translucent plate fragment", material: "tools", kg: 0.1, description: "A translucent plate bears branching seams reminiscent of the wayfinder's back; it may be shed material or an imitation." },
Item { id: "glassback/mosaic", patron: "glassback", kind: "glassback mosaic fragment", material: "pottery", kg: 0.25, description: "A mosaic depicts a plated swimmer among alternating bands of dark and clear water." },
Item { id: "glassback/lens", patron: "glassback", kind: "polished lake lens", material: "tools", kg: 0.08, description: "A cloudy polished lens scatters light into the patterns described in founding accounts." },
Item { id: "lantern_elk/antler", patron: "lantern_elk", kind: "antler-shaped pendant", material: "wood", kg: 0.06, description: "A carved pendant follows the unusual branching of the lantern elk's antlers." },
Item { id: "lantern_elk/lamp", patron: "lantern_elk", kind: "antler-handled lamp", material: "pottery", kg: 0.25, description: "A soot-marked lamp has a handle shaped like the patron's antlers; no light remains in it." },
Item { id: "lantern_elk/bark", patron: "lantern_elk", kind: "forest procession bark strip", material: "wood", kg: 0.05, description: "Marks on a preserved bark strip show small walkers following a tall branching figure." },
Item { id: "basalt_tortoise/shell", patron: "basalt_tortoise", kind: "shell-pattern rubbing", material: "cloth", kg: 0.08, description: "A pigment rubbing reproduces grooves resembling the tortoise's weathered shell." },
Item { id: "basalt_tortoise/tablet", patron: "basalt_tortoise", kind: "tortoise relief tablet", material: "pottery", kg: 0.3, description: "A relief shows a broad tortoise carrying bundles across broken ground." },
Item { id: "basalt_tortoise/weight", patron: "basalt_tortoise", kind: "shell-shaped survey weight", material: "metal", kg: 0.2, description: "A heavy notched weight has the patron's shell silhouette, perhaps an ordinary surveying aid." },
Item { id: "storm_heron/feather", patron: "storm_heron", kind: "long flight-feather remnant", material: "leather", kg: 0.02, description: "A preserved feather remnant resembles the long flight feathers described by founding witnesses." },
Item { id: "storm_heron/vane", patron: "storm_heron", kind: "heron-shaped wind vane", material: "metal", kg: 0.2, description: "A bent vane depicts a long-winged bird turning toward approaching weather." },
Item { id: "storm_heron/call", patron: "storm_heron", kind: "storm-call pipe", material: "tools", kg: 0.08, description: "A narrow pipe produces a rasping call resembling a traditional imitation of the patron." },
Item { id: "root_marten/pouch", patron: "root_marten", kind: "root-woven seed pouch", material: "cloth", kg: 0.07, description: "An empty root-fiber pouch bears a small burrowing-animal motif; its seed pockets are worn smooth." },
Item { id: "root_marten/comb", patron: "root_marten", kind: "layered-fur grooming comb", material: "wood", kg: 0.08, description: "A comb has unusually deep alternating teeth suitable for layered fur." },
Item { id: "root_marten/tunnel", patron: "root_marten", kind: "burrow-marked tablet", material: "pottery", kg: 0.22, description: "A tablet depicts branching burrows and bundles of seeds beside a small animal." },
Item { id: "silver_manta/ribbon", patron: "silver_manta", kind: "silver-patterned river ribbon", material: "cloth", kg: 0.05, description: "A ribbon preserves a ray-like pattern and loops that could represent feeding tendrils." },
Item { id: "silver_manta/bowl", patron: "silver_manta", kind: "manta-shaped shallow bowl", material: "pottery", kg: 0.25, description: "A shallow bowl spreads into broad ray-shaped wings, with river marks around its rim." },
Item { id: "silver_manta/chime", patron: "silver_manta", kind: "river-pattern chime", material: "metal", kg: 0.12, description: "A thin chime is etched with curved bands resembling the patron's signaling patterns." },
Item { id: "ash_bear/claw", patron: "ash_bear", kind: "claw-shaped digging handle", material: "wood", kg: 0.18, description: "A broken digging handle is carved like the broad claws attributed to the ash bear." },
Item { id: "ash_bear/print", patron: "ash_bear", kind: "large paw impression", material: "pottery", kg: 0.3, description: "A fired impression preserves an enormous paw shape; scale alone cannot identify its maker." },
Item { id: "ash_bear/wrap", patron: "ash_bear", kind: "ash-stained carrying wrap", material: "cloth", kg: 0.12, description: "A coarse carrying wrap bears a bear-shaped repair patch and old ash stains." },
Item { id: "many_hands/tool", patron: "many_hands", kind: "many-gripped craft tool", material: "tools", kg: 0.2, description: "A small craft tool has several grip points, suggesting coordinated use by many hands." },
Item { id: "many_hands/gloves", patron: "many_hands", kind: "paired miniature glove remnants", material: "leather", kg: 0.06, description: "Fragments of several narrow gloves are joined by one surviving cord." },
Item { id: "many_hands/knots", patron: "many_hands", kind: "eight-strand instruction braid", material: "cloth", kg: 0.08, description: "An eight-strand braid repeats a sequence of knots that might encode a manual practice." },
Item { id: "salt_listener/cup", patron: "salt_listener", kind: "shell-shaped listening cup", material: "pottery", kg: 0.18, description: "A cup directs sound toward a narrow opening like a listening instrument." },
Item { id: "salt_listener/tokens", patron: "salt_listener", kind: "salt-worn dialogue tokens", material: "pottery", kg: 0.1, description: "Paired tokens carry alternating marks that interpreters read as turns in a conversation." },
Item { id: "salt_listener/mantle", patron: "salt_listener", kind: "coastal interpreter mantle scrap", material: "cloth", kg: 0.09, description: "A salt-stiffened mantle scrap has edging shaped around a broad shell-like torso." },
Item { id: "deep_weaver/loom", patron: "deep_weaver", kind: "small subterranean loom shuttle", material: "wood", kg: 0.12, description: "A narrow shuttle retains traces of unfamiliar fiber caught in its eye." },
Item { id: "deep_weaver/veil", patron: "deep_weaver", kind: "deep-fiber veil fragment", material: "cloth", kg: 0.06, description: "A finely layered veil resembles the living-fiber clothing of founding accounts, but the recovered fibers are inert." },
Item { id: "deep_weaver/signs", patron: "deep_weaver", kind: "woven hand-sign sampler", material: "cloth", kg: 0.08, description: "A sampler repeats hand-like motifs which may depict signs rather than decoration." },
Item { id: "dawn_keeper/fan", patron: "dawn_keeper", kind: "throat-fan ornament", material: "cloth", kg: 0.07, description: "A pleated ornament resembles the dawn keeper's resonant throat fan." },
Item { id: "dawn_keeper/pipe", patron: "dawn_keeper", kind: "dawn-call double pipe", material: "tools", kg: 0.16, description: "A double pipe produces two overlapping notes comparable to ceremonial dawn calls." },
Item { id: "dawn_keeper/feathers", patron: "dawn_keeper", kind: "feather-patterned sash", material: "cloth", kg: 0.1, description: "A faded sash bears a procession of feathered bipeds beneath a rising disc." },
];
pub(super) fn get(id: &str) -> Option<&'static Item> {
    ITEMS.iter().find(|item| item.id == id)
}
pub(super) fn choose(patron: &str, draw: u32) -> Option<&'static Item> {
    let count = ITEMS.iter().filter(|i| i.patron == patron).count();
    if count == 0 {
        return None;
    }
    ITEMS
        .iter()
        .filter(|i| i.patron == patron)
        .nth(draw as usize % count)
}
