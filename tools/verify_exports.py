#!/usr/bin/env python3
"""Verify downloaded evidence against inventory; report gaps without pretending success."""
import hashlib
import json
from pathlib import Path
import re
import zipfile
from preserve import ROOT, REPOS, items, save

secret = re.compile(rb'(?:gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{40,}|-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----|AKIA[0-9A-Z]{16})')
manifest=[]
summary=[]
findings=[]
for repo in REPOS:
    folder=ROOT/'records'/repo
    artifacts=items(json.loads((folder/'artifacts.json').read_text()),'artifacts')
    available=[a for a in artifacts if not a['expired']]
    for a in available:
        path=folder/'artifact-bytes'/f'{a["id"]}.zip'
        assert path.is_file(),f'Available artifact missing: {path}'
        digest=hashlib.file_digest(path.open('rb'),'sha256').hexdigest()
        assert not a.get('digest') or a['digest']=='sha256:'+digest, path
    prs=items(json.loads((folder/'pulls.json').read_text()))
    for pr in prs:
        for name in ['detail.json','reviews.json','commits.json','timeline.json']:
            assert (folder/'pulls'/str(pr['number'])/name).is_file(),(repo,pr['number'],name)
    runs=items(json.loads((folder/'runs.json').read_text()),'workflow_runs')
    for run in runs:
        d=folder/'runs'/str(run['id'])
        assert (d/'jobs.json').is_file(),d
        for attempt in range(1,run.get('run_attempt',1)+1):
            p=d/f'attempt-{attempt}.zip'
            assert p.is_file() or p.with_suffix('.zip.unavailable.json').is_file(),p
    for path in sorted(folder.rglob('*')):
        if not path.is_file():continue
        assert not path.name.endswith('.tmp'), path
        data=path.read_bytes()
        if path.suffix=='.zip':
            with zipfile.ZipFile(path) as z:
                assert z.testzip() is None,path
                for member in z.infolist():
                    contents=z.read(member)
                    if secret.search(contents):findings.append({'path':str(path.relative_to(ROOT)),'member':member.filename})
        elif secret.search(data):findings.append({'path':str(path.relative_to(ROOT))})
        manifest.append({'path':str(path.relative_to(ROOT)),'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()})
    unavailable=[str(p.relative_to(ROOT)) for p in folder.rglob('*.unavailable.json')]
    summary.append({'repository':repo,'prs':len(prs),'workflow_runs':len(runs),'available_artifacts':len(available),'expired_artifacts':len(artifacts)-len(available),'unavailable_logs':unavailable})
assert sum(r['available_artifacts'] for r in summary)==1766
save(ROOT/'evidence-manifest.json',manifest)
save(ROOT/'evidence-summary.json',summary)
save(ROOT/'secret-scan-findings.json',findings)
print(json.dumps({'files':len(manifest),'bytes':sum(x['bytes'] for x in manifest),'artifacts':1766,'unavailable_logs':sum(len(x['unavailable_logs']) for x in summary),'secret_pattern_findings':len(findings)},indent=2))
if findings:raise SystemExit('Review secret pattern findings privately before publishing records.')
