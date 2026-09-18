#!/usr/bin/env python3
import json
import os
from pathlib import Path
from preserve import ROOT, REPOS, command, items, save

repo='/home/ubuntu/bulletfarm'
def git(*args,**kw):return command(['git','-C',repo,*args],**kw)
assert json.loads((ROOT/'export-failures.json').read_text())==[]
assert json.loads((ROOT/'secret-scan-findings.json').read_text())==[]
env=dict(os.environ,GIT_INDEX_FILE=str(ROOT/'archive-evidence.index'))
git('read-tree','archive/history',env=env)

def add(path,data):
    if isinstance(data,str):data=data.encode()
    oid=git('hash-object','-w','--stdin',input=data).decode().strip()
    git('update-index','--add','--cacheinfo',f'100644,{oid},{path}',env=env)

index=[]
for name in REPOS:
    for pr in items(json.loads((ROOT/'records'/name/'pulls.json').read_text())):
        n=pr['number']; detail=json.loads((ROOT/'records'/name/'pulls'/str(n)/'detail.json').read_text())
        tag=f'archive/{name}/pull/{n}/head'
        assert git('rev-parse',f'refs/tags/{tag}').decode().strip()==detail['head']['sha'],tag
        index.append({'repository':f'neverhuman/{name}','number':n,'original_url':pr['html_url'],'title':pr['title'],'head_sha':detail['head']['sha'],'source_tag':tag,'records':f'records/{name}/pulls/{n}/','discussion':f'records/{name}/issue-comments.json','inline_comments':f'records/{name}/review-comments.json'})
save(ROOT/'pr-index.json',index)
for record in json.loads((ROOT/'evidence-manifest.json').read_text()):
    path=ROOT/record['path']
    assert path.stat().st_size<=50*1024*1024,path
    add(record['path'],path.read_bytes())
for file in ['evidence-manifest.json','evidence-summary.json','pr-index.json']:
    add(file,(ROOT/file).read_bytes())
for file in ['source-archive-verification.json','remote-ref-verification.json','source-secret-scan.json','secret-scan-findings.json']:
    add('verification/'+file,(ROOT/file).read_bytes())
for file in ['preserve.py','verify_exports.py','archive_sources.py','publish_evidence.py']:
    add('tools/'+file,(ROOT/'tools'/file).read_bytes())
add('README.md','''# Historical BulletFarm archive

This branch preserves superseded implementations and historical evidence. Its instructions,
plans, grants and reviews do not govern current development or constitute current approvals.
Development occurs only on `neverhuman/bulletfarm/main` in `/home/ubuntu/bulletfarm`.

## Find original code, PRs and checks

- `snapshots/<repository>/`: six byte-identical source main trees at inventory.
- `ref-index.json`: all 1,475 original remote/local/reflog refs, original object IDs and
  their immutable `archive/<repository>/...` tags. Git authorship is unchanged.
- `pr-index.json`: all 68 historical PR numbers, original head SHAs, source tags and exports.
- `records/<repository>/pulls/<number>/`: descriptions, reviews, commits and timelines.
  Repository-level issue-comments and review-comments files preserve discussions/replies.
- `records/<repository>/checks/` and `statuses/`: historical PR-head check results.
- `records/<repository>/runs/`: job/check results and all available attempt logs (ZIP).
- `records/<repository>/artifact-bytes/`: all 1,766 available CI artifact ZIPs.
- `evidence-manifest.json`: SHA-256 and byte count for every exported evidence file.
  `evidence-summary.json` records repository counts and unavailable material explicitly.

The capture includes 368 workflow runs. No inventoried artifact was expired, no available
artifact failed capture, and no requested workflow attempt log was unavailable. Wikis were
unavailable (recorded per repository); all six repositories had Discussions disabled and no
releases at inventory. These statements describe the captured inventory, not future changes.

The pending `bf` PR 12 head is `b9bd7227ab9ce3dde8959dd0082a3848ed778673` at
`archive/bf/pull/12/head`. Both exact-head review exchanges, discussion, and subsequent
supersession/closure evidence are retained. Historical reviews are not newly imported native
GitHub approvals. The successor lives in the active repository.

No private local Git configuration, credential store, coordination database or launcher is
published. Private mirrors, Git objects, working state, ignored evidence, coordination history
and launchers are backed up outside the development checkout. Build/dependency caches are
excluded from private working-state archives; their paths are inventoried. Original Git source
objects are preserved unchanged, including deliberate secret-redaction test canaries. Exported
record/ZIP contents passed the documented credential-pattern scan with no findings.

Logs/artifacts are compressed; no exported file exceeds 50 MiB. Larger private backup archives
are split into ordered parts with SHA-256 manifests. A fresh GitHub mirror verifies all original
ref object IDs and passes Git integrity checks. See `verification/` for evidence and `tools/`
for the capture/verification implementation. Tools are historical diagnostics, not live authority.

This archive does not by itself authorize deletion. Both migration PRs, independent reviews,
installation/data/PR-discovery checks, independent-backup restoration, final source recheck
and operator-assisted deletion authentication must pass the active migration plan's gates.
''')
tree=git('write-tree',env=env).decode().strip()
parent=git('rev-parse','archive/history').decode().strip()
commit=git('commit-tree',tree,'-p',parent,input=b'Archive 68 PRs, 368 workflow runs and all 1766 CI artifacts with checksums\n').decode().strip()
git('update-ref','refs/heads/archive/history',commit,parent)
save(ROOT/'published-evidence.json',{'commit':commit,'files':len(json.loads((ROOT/'evidence-manifest.json').read_text())),'artifacts':1766,'prs':68,'runs':368})
print(json.dumps({'archive_commit':commit,'artifacts':1766,'prs':68,'runs':368}))
