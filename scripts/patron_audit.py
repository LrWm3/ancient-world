#!/usr/bin/env python3
"""Audit saved patron histories independently of their runtime validation."""
import argparse
import collections
import json
import struct
from pathlib import Path


def audit(path):
    with path.open('rb') as stream:
        magic = stream.read(8)
        assert magic == b'ANCIENT2', f'{path}: unsupported archive'
        length = struct.unpack('<Q', stream.read(8))[0]
        assert length <= 256 * 1024 * 1024
        world = json.loads(stream.read(length))
    h = world['civilizations']
    c = h['culture']
    events = h['events']
    counts = collections.Counter(e['kind'] for e in events)
    for i, event in enumerate(events):
        assert event['id'] == i
        assert all(cause < i for cause in event.get('causes', [])), (path, 'event cycle', i)
    if not c['legacy_baseline']:
        assert counts['patron_arrival'] == len(c['patrons'])
        assert len({p['civilization'] for p in c['patrons']}) == len(c['patrons'])
    for patron in c['patrons']:
        assert events[patron['arrival_event']]['kind'] == 'patron_arrival'
        assert patron['witnesses']
        if h['month'] >= patron['departure_month']:
            assert patron['departed'] == patron['departure_month']
        assert abs(patron['imported_kg'] - patron['consumed_kg'] - patron['provisions_kg'] - patron['returned_kg']) < 1e-6
    for account in c['accounts']:
        assert account['author'] is not None or account['institution'] is not None
        assert account['facts']
        if account['author'] is not None:
            author = h['people'][account['author']]
            assert author['died'] is None or author['died'] >= account['month'], (path, 'account composed after author death', account['id'])
        assert all(events[event]['month'] <= account['month'] for event in account['facts'])
    for artifact in c['artifacts']:
        assert artifact['events']
        assert all(a < b for a, b in zip(artifact['events'], artifact['events'][1:]))
        if artifact.get('lost', False):
            assert artifact['custodian'] is None or artifact['destroyed']
    for agent in c['agents']:
        for topic, event in agent.get('knowledge_sources', {}).items():
            assert int(topic) in agent['knowledge']
            assert ['person', agent['person']] in events[event]['subjects'], (path, 'learning without learner reference', agent['person'], event)
    assert counts['pilgrimage_departed'] == counts['pilgrimage_returned']
    adults = [a for a in c['agents'] if h['people'][a['person']]['died'] is None and h['month'] - h['people'][a['person']]['born'] >= 180]
    return dict(path=str(path), year=h['month'] // 12, patrons=len(c['patrons']), traditions=len(c['traditions']), tradition_capacity_reached=len(c['traditions']) >= 256, living_adults=len(adults), living_knowledge=sum(len(a['knowledge']) for a in adults), pilgrimages=counts['pilgrimage_returned'], curated=counts['specimen_curated'], campaigns=counts['office_campaign'], manuscripts=sum(a['kind'] == 'inscribed manuscript' and not a['destroyed'] and not a.get('lost', False) for a in c['artifacts']), people=len(h['people']), events=len(events))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('worlds', nargs='+', type=Path)
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    text = json.dumps({'passed': True, 'worlds': [audit(path) for path in args.worlds]}, indent=2)
    if args.output:
        args.output.write_text(text + '\n')
    print(text)
