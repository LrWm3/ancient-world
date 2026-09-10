#!/usr/bin/env python3
"""Audit settlement lifecycle and post-abandonment notices in saved worlds."""
import argparse
from collections import Counter
import json
from pathlib import Path
import struct

LOCAL_ACTIVITY = {'persistent_inundation', 'inundation_recovered', 'flood', 'flood_receded', 'flood_recovery', 'flood_adaptation',
                  'road_flood_closed', 'road_flood_recovered', 'port_weather_closed',
                  'port_weather_recovered', 'sea_weather_closed', 'sea_weather_recovered', 'harvest', 'food_crisis', 'regional_drought', 'weather_recovery',
                  'inheritance', 'succession', 'marriage', 'faith_adopted',
                  'religious_interpretation', 'religious_syncretism', 'religious_schism'}


def audit(path):
    with path.open('rb') as stream:
        if stream.read(8) != b'ANCIENT2':
            raise ValueError(f'{path}: unsupported archive')
        size = struct.unpack('<Q', stream.read(8))[0]
        h = json.loads(stream.read(size))['civilizations']
    abandoned = set()
    ghosts = []
    for event in h['events']:
        site = event['site']
        if event['kind'] == 'abandoned':
            abandoned.add(site)
        elif event['kind'] == 'settlement_reoccupied':
            abandoned.discard(site)
        elif site in abandoned and event['kind'] in LOCAL_ACTIVITY:
            ghosts.append(event)
    return {
        'world': str(path), 'seed': h['seed'], 'years': h['month'] / 12,
        'active_population': sum(s['stocks']['stock'][0] for s in h['sites'] if not s['abandoned']),
        'sizes': dict(Counter('Ruins' if s['abandoned'] else s.get('lifecycle', {}).get('size', 'Legacy') for s in h['sites'])),
        'reclassifications': sum(e['kind'] == 'settlement_reclassified' for e in h['events']),
        'abandonments': sum(e['kind'] == 'abandoned' for e in h['events']),
        'reoccupations': sum(e['kind'] == 'settlement_reoccupied' for e in h['events']),
        'ghost_activity_count': len(ghosts), 'ghost_activity_examples': ghosts[:10],
        'ruins_population': sum(s['stocks']['stock'][0] for s in h['sites'] if s['abandoned']),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('worlds', nargs='+', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    rows = [audit(path) for path in args.worlds]
    args.output.write_text(json.dumps(rows, indent=2) + '\n')
    print(json.dumps(rows, indent=2))
    return int(any(row['ghost_activity_count'] or row['ruins_population'] for row in rows))


if __name__ == '__main__':
    raise SystemExit(main())
