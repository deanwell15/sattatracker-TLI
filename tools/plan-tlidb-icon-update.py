#!/usr/bin/env python3
"""Developer-only TLIDB update planner. Reads a local TITrack-style reference catalog.
Produces a reviewable missing-icon download plan; never contacts TLIDB/CDN.
Actual downloading remains disabled until access/cache/distribution terms are confirmed.
"""
import argparse
import hashlib
import json
from pathlib import Path
from urllib.parse import urlsplit

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--catalog', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
root = Path(__file__).resolve().parents[1]
known = set(json.loads((root/'src/assets/original-icons.json').read_text())['ids'])
raw = args.catalog.read_bytes()
items = json.loads(raw).get('items', [])
missing = {}
for item in items:
    key = str(item['id'])
    if not key.isascii() or not key.isdigit() or len(key)>12:
        raise ValueError('Invalid reference ID')
    if key in known:
        continue
    url = item.get('img')
    if url:
        parsed = urlsplit(url)
        if parsed.scheme != 'https' or parsed.hostname != 'cdn.tlidb.com' or parsed.username or parsed.password or parsed.port not in (None,443):
            raise ValueError('Unexpected reference image URL')
    entry = {'id': key, 'candidate_url': url}
    if key in missing and missing[key] != entry:
        raise ValueError('Conflicting reference image mapping: '+key)
    missing[key] = entry
plan = {'schema': 1, 'reference_sha256': hashlib.sha256(raw).hexdigest(),
        'season_id': 1601, 'season_note': 'Reference review target; not inferred from every catalog record.',
        'download_enabled': False,
        'required_before_download': ['Confirmed TLIDB automated access method and limits', 'Permission for offline cache and public asset redistribution', 'Recorded source attribution/license terms'],
        'missing': sorted(missing.values(), key=lambda item:int(item['id']))}
args.output.parent.mkdir(parents=True, exist_ok=True)
args.output.write_text(json.dumps(plan,indent=2)+'\n',encoding='utf-8')
print(f'Prepared {len(missing)} candidates. No network requests or resource changes.')
