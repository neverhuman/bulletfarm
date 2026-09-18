#!/usr/bin/env python3
"""Import original objects and make browsable snapshots without another worktree."""
import json
import os
from pathlib import Path
import subprocess
from preserve import ROOT, REPOS, command, save

repo=Path('/home/ubuntu/bulletfarm')
def git(*args,**kw):return command(['git','-C',str(repo),*args],**kw)
all_refs=[]
for name in REPOS:
    refs=json.loads((ROOT/'records'/name/'all-refs.json').read_text())
    git('fetch',str(ROOT/'mirrors'/f'{name}.git'),f'refs/*:refs/tags/archive/{name}/*',stderr=subprocess.DEVNULL)
    for ref in refs:
        actual=git('rev-parse','refs/tags/'+ref['archive_tag']).decode().strip()
        assert actual==ref['oid'], (name,ref,actual)
        all_refs.append({'repository':f'neverhuman/{name}',**ref})
save(ROOT/'archive-ref-index.json',all_refs)
env=dict(os.environ,GIT_INDEX_FILE=str(ROOT/'archive.index'))
git('read-tree','--empty',env=env)
for name in REPOS:
    tag=f'refs/tags/archive/{name}/heads/main'
    git('read-tree',f'--prefix=snapshots/{name}/',tag,env=env)

def add(path,data):
    if not isinstance(data,bytes):data=data.encode()
    oid=git('hash-object','-w','--stdin',input=data).decode().strip()
    git('update-index','--add','--cacheinfo',f'100644,{oid},{path}',env=env)

add('README.md', '''# Historical BulletFarm archive

This branch preserves superseded implementations and source evidence. Its instructions,
plans, grants and reviews are historical records; they do not govern current development
or constitute current approvals. Development occurs only on `neverhuman/bulletfarm/main`.

`snapshots/<repository>/` contains each source main tree byte-for-byte at the inventory.
`ref-index.json` maps every inventoried ref to an immutable `archive/<repository>/...`
tag with its original Git object ID and authorship. `records/` records source inventories
and GitHub exports; more evidence is appended as capture completes. PR heads are under
`archive/<repository>/pull/<number>/head`, with their review evidence in
`records/<repository>/pulls/<number>/`. Local reflog and branch refs retain original objects.

No local Git configuration, credential stores or live coordination database is published.
An independent private backup outside the development checkout retains local state.
GitHub records are historical evidence, not newly imported native GitHub reviews.
Retirement remains blocked until every available export and restoration gate passes.
''')
add('ref-index.json',(ROOT/'archive-ref-index.json').read_bytes())
for name in REPOS:
    for filename in ['source-refs.json','all-refs.json','git-fsck.txt']:
        add(f'records/{name}/{filename}',(ROOT/'records'/name/filename).read_bytes())
tree=git('write-tree',env=env).decode().strip()
parent=git('rev-parse','refs/tags/archive/bulletfarm/heads/main').decode().strip()
commit=git('commit-tree',tree,'-p',parent,input=b'Preserve six original repository histories and browsable main snapshots\n').decode().strip()
git('update-ref','refs/heads/archive/history',commit)
for name in REPOS:
    assert git('rev-parse',f'{commit}:snapshots/{name}').strip()==git('rev-parse',f'refs/tags/archive/{name}/heads/main^{{tree}}').strip()
save(ROOT/'source-archive-verification.json',{'commit':commit,'refs':len(all_refs),'snapshots':6,'original_object_ids_verified':True})
print(json.dumps({'commit':commit,'refs':len(all_refs),'snapshots_verified':6}))
