#!/usr/bin/env python3
"""Compare exported histories allowing only added loan events and shifted references."""
import argparse
import copy
import json
import re
from pathlib import Path

# Explicit reference paths: never normalize arbitrary integer-valued simulation data.
EVENT_REFERENCES = {
    'culture.institutions.#.capacity.mandate.events.#',
    'culture.accounts.#.facts.#',
    'culture.work_plans.#.identities.people.#.#.#.#.source',
    'culture.agents.#.knowledge_sources.#',
    'culture.agents.#.last_campaign',
    'culture.agents.#.studies.#.source',
    'culture.artifacts.#.events.#',
    'offices.seats.#.last_event',
    'offices.seats.#.tenures.#.appointment',
    'governance.petitions.#.outcome',
    'governance.petitions.#.cause',
    'governance.treaties.#.cause',
    'society.councils.#.pending_distribution.cause',
}


def compare(before, after):
    """Fail closed on unexpected changes, including new unrecognized reference paths."""
    old = copy.deepcopy(before)
    new = copy.deepcopy(after)
    kept = [e for e in new['events'] if not e['kind'].startswith('loan_')]
    assert len(kept) == len(old['events']), 'non-loan event count changed'
    mapping = {}
    for earlier, later in zip(old['events'], kept):
        mapping[later['id']] = earlier['id']
    for earlier, later in zip(old['events'], kept):
        later['id'] = mapping[later['id']]
        later['causes'] = [mapping[c] for c in later['causes']]
        if later['kind'].startswith('civic_petition_'):
            later['detail'] = re.sub(
                r'\bpetition (\d+)\b',
                lambda match: f"petition {mapping[int(match[1])]}",
                later['detail'],
            )
        assert earlier == later, f"event content changed at {earlier['id']}"
    new['events'] = kept

    def normalize(value, path=()):
        key = '.'.join(path)
        if key in EVENT_REFERENCES:
            return mapping[value] if value is not None else None
        if isinstance(value, dict):
            return {k: normalize(v, path + ('#' if k.isdigit() else k,)) for k, v in value.items()}
        if isinstance(value, list):
            return [normalize(v, path + ('#',)) for v in value]
        return value

    # A cursor is a count of processed entries, not an event identity.
    cursor = new['governance']['event_cursor']
    new['governance']['event_cursor'] = sum(
        e['id'] < cursor and not e['kind'].startswith('loan_')
        for e in after['events']
    )
    old['credit'].pop('last_events', None)
    new['credit'].pop('last_events', None)
    normalized = normalize(new)
    changed = [k for k in old.keys() | normalized.keys()
               if old.get(k) != normalized.get(k)]
    assert not changed, f'unexpected changed sections: {changed}'
    return len(after['events']) - len(before['events'])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('before', type=Path)
    parser.add_argument('after', type=Path)
    args = parser.parse_args()
    added = compare(json.loads(args.before.read_text()), json.loads(args.after.read_text()))
    print(f'Equivalent apart from {added} added loan events, their index and shifted references.')


if __name__ == '__main__':
    main()
