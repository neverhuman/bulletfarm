#!/usr/bin/env python3
"""Resumable, read-only source/forge capture. No credentials in the public records."""
import concurrent.futures as futures
import hashlib
import json
import os
from pathlib import Path
import shutil
import sqlite3
import subprocess
import sys
import tarfile
import time

ROOT = Path(__file__).resolve().parents[1]
REPOS = ['bulletfarm', 'bf', 'bullet-farm', 'bullet-kernel', 'bullet-git', 'bullet-portal']
os.umask(0o077)

def command(args, **kw):
    return subprocess.check_output(args, **kw)

def save(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_suffix(path.suffix + '.tmp')
    tmp.write_bytes(data if isinstance(data, bytes) else (json.dumps(data, indent=2) + '\n').encode())
    tmp.replace(path)

def api(repo, endpoint, dest, paged=False, optional=False):
    path = ROOT / 'records' / repo / dest
    if path.exists():
        return json.loads(path.read_text())
    args = ['gh', 'api', f'repos/neverhuman/{repo}{endpoint}']
    if paged:
        args += ['--paginate', '--slurp']
    for attempt in range(4):
        p = subprocess.run(args, capture_output=True)
        if p.returncode == 0:
            data = json.loads(p.stdout)
            save(path, data)
            return data
        if optional and ('HTTP 404' in p.stderr.decode() or 'HTTP 410' in p.stderr.decode()):
            data = {'unavailable': p.stderr.decode(), 'endpoint': endpoint}
            save(path, data)
            return data
        if attempt < 3:
            time.sleep(2 ** attempt)
    raise RuntimeError(f'{repo} {endpoint}: {p.stderr.decode()}')

def items(pages, key=None):
    return [x for p in pages for x in (p[key] if key else p)]

def source(repo):
    mirror = ROOT / 'mirrors' / (repo + '.git')
    if not mirror.exists():
        command(['git', 'clone', '--mirror', f'https://github.com/neverhuman/{repo}.git', str(mirror)], stderr=subprocess.STDOUT)
    # Include GitHub PR heads and merge refs; clone --mirror also captures advertised refs.
    command(['git', '-C', str(mirror), 'fetch', 'origin', '+refs/heads/*:refs/heads/*', '+refs/tags/*:refs/tags/*', '+refs/pull/*:refs/pull/*'], stderr=subprocess.STDOUT)
    remote = command(['git', '-C', str(mirror), 'for-each-ref', '--format=%(objectname) %(refname)']).decode()
    save(ROOT/'records'/repo/'source-refs.json', [{'oid': l.split()[0], 'ref': l.split()[1]} for l in remote.splitlines()])
    local = Path('/home/ubuntu/bulletfarm') if repo == 'bulletfarm' else Path('/home/ubuntu/bullet')/repo
    dest = ROOT/'local'/repo
    dest.mkdir(parents=True, exist_ok=True)
    if not (dest/'capture-complete.json').exists():
        for name, args in [('status.txt',['status','--porcelain=v1','--untracked-files=all']), ('refs.txt',['show-ref']), ('reflogs.txt',['reflog','show','--all','--format=%H %gd %gs']), ('diff.patch',['diff','--binary','HEAD']), ('index.patch',['diff','--binary','--cached']), ('ignored.txt',['ls-files','--others','--ignored','--exclude-standard'])]:
            save(dest/name, command(['git','-C',str(local),*args]))
        # Back up all Git objects (including local unreachable/reflog history), refs, index, and private config.
        git_copy = dest/'git'
        git_copy.mkdir(exist_ok=True)
        for name in ['objects','refs','logs','packed-refs','HEAD','index','config','shallow','info']:
            src = local/'.git'/name
            if src.is_dir():
                shutil.copytree(src,git_copy/name,dirs_exist_ok=True)
            elif src.exists():
                shutil.copy2(src,git_copy/name)
        names = command(['git','-C',str(local),'ls-files','-z','--cached','--others','--exclude-standard']).decode().split('\0')
        with tarfile.open(dest/'working-tree.tar.gz','w:gz') as tf:
            for name in sorted(set(names)-{''}):
                if os.path.lexists(local/name):
                    tf.add(local/name,arcname=name,recursive=False)
        save(dest/'capture-complete.json',{'at':time.time(),'excluded':'Ignored generated files listed in ignored.txt; private Git config stays private.'})
    command(['git','-C',str(mirror),'fetch',str(local),'+refs/*:refs/local/*'],stderr=subprocess.STDOUT)
    command(['git','-C',str(mirror),'fetch',str(local),'HEAD:refs/local/HEAD'],stderr=subprocess.STDOUT)
    # Reflog commits may no longer have named refs. Preserve each under an immutable object-ID name.
    for line in (dest/'reflogs.txt').read_text().splitlines():
        oid=line.split()[0]
        exists=subprocess.run(['git','-C',str(mirror),'cat-file','-e',oid],capture_output=True).returncode==0
        if not exists:
            command(['git','-C',str(mirror),'fetch',str(local),f'{oid}:refs/local/reflog/{oid}'],stderr=subprocess.STDOUT)
        else:
            command(['git','-C',str(mirror),'update-ref',f'refs/local/reflog/{oid}',oid])
    command(['git','-C',str(mirror),'lfs','fetch','--all'],stderr=subprocess.STDOUT)
    save(ROOT/'records'/repo/'git-fsck.txt', command(['git','-C',str(mirror),'fsck','--full'],stderr=subprocess.STDOUT))
    refs=command(['git','-C',str(mirror),'for-each-ref','--format=%(objectname) %(refname)']).decode()
    save(ROOT/'records'/repo/'all-refs.json',[{'oid':l.split()[0],'ref':l.split()[1], 'archive_tag':f'archive/{repo}/{l.split()[1].removeprefix("refs/")}'} for l in refs.splitlines()])
    print(f'source {repo}: {len(refs.splitlines())} refs',flush=True)

def inventory(repo):
    api(repo,'','repository.json')
    for ep,name,key in [('/pulls?state=all&per_page=100','pulls.json',None),('/issues?state=all&per_page=100','issues.json',None),('/issues/comments?per_page=100','issue-comments.json',None),('/pulls/comments?per_page=100','review-comments.json',None),('/issues/events?per_page=100','issue-events.json',None),('/actions/artifacts?per_page=100','artifacts.json','artifacts'),('/actions/runs?per_page=100','runs.json','workflow_runs'),('/releases?per_page=100','releases.json',None)]:
        data=api(repo,ep,name,paged=True)
        print(f'inventory {repo} {name}: {len(items(data,key))}',flush=True)
    api(repo,'/branches/main/protection','branch-protection.json',optional=True)

def download(repo,endpoint,path,optional=False):
    path = ROOT/'records'/repo/path
    if path.exists():
        return
    if path.with_suffix(path.suffix+'.unavailable.json').exists():
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp=path.with_suffix(path.suffix+'.tmp')
    for attempt in range(4):
        with tmp.open('wb') as stream:
            p=subprocess.run(['gh','api',f'repos/neverhuman/{repo}{endpoint}'],stdout=stream,stderr=subprocess.PIPE)
        if p.returncode==0:
            # gh follows signed redirects; only publish downloaded bytes, never redirect credentials.
            tmp.replace(path)
            return
        tmp.unlink(missing_ok=True)
        if optional and any(x in p.stderr.decode() for x in ['HTTP 404','HTTP 410']):
            save(path.with_suffix(path.suffix+'.unavailable.json'),{'endpoint':endpoint,'reason':p.stderr.decode()})
            return
        if attempt<3:time.sleep(2**attempt)
    raise RuntimeError(f'{repo} {endpoint}: {p.stderr.decode()}')

def export_pr(repo,pr):
    n=pr['number']; base=f'pulls/{n}'
    detail=api(repo,f'/pulls/{n}',f'{base}/detail.json')
    api(repo,f'/pulls/{n}/reviews?per_page=100',f'{base}/reviews.json',paged=True)
    api(repo,f'/pulls/{n}/commits?per_page=100',f'{base}/commits.json',paged=True)
    api(repo,f'/issues/{n}/timeline?per_page=100',f'{base}/timeline.json',paged=True)
    sha=detail['head']['sha']
    api(repo,f'/commits/{sha}/check-runs?per_page=100',f'checks/{sha}.json',paged=True,optional=True)
    api(repo,f'/commits/{sha}/status?per_page=100',f'statuses/{sha}.json',paged=True,optional=True)

def export_run(repo,run):
    rid=run['id'];base=f'runs/{rid}'
    api(repo,f'/actions/runs/{rid}/jobs?filter=all&per_page=100',f'{base}/jobs.json',paged=True)
    for attempt in range(1,run.get('run_attempt',1)+1):
        download(repo,f'/actions/runs/{rid}/attempts/{attempt}/logs',f'{base}/attempt-{attempt}.zip',optional=True)

def export_artifact(repo,a):
    if a['expired']:
        save(ROOT/'records'/repo/'artifact-bytes'/f'{a["id"]}.expired.json',{'id':a['id'],'expired_at_inventory':True})
        return
    download(repo,f'/actions/artifacts/{a["id"]}/zip',f'artifact-bytes/{a["id"]}.zip')
    path=ROOT/'records'/repo/'artifact-bytes'/f'{a["id"]}.zip'
    digest=hashlib.file_digest(path.open('rb'),'sha256').hexdigest()
    if a.get('digest') and a['digest'] != 'sha256:'+digest:
        raise RuntimeError(f'artifact digest mismatch {repo}/{a["id"]}')

def exports():
    jobs=[]
    for repo in REPOS:
        folder=ROOT/'records'/repo
        jobs += [(export_pr,repo,p) for p in items(json.loads((folder/'pulls.json').read_text()))]
        jobs += [(export_run,repo,r) for r in items(json.loads((folder/'runs.json').read_text()),'workflow_runs')]
        jobs += [(export_artifact,repo,a) for a in items(json.loads((folder/'artifacts.json').read_text()),'artifacts')]
    failures=[]
    with futures.ThreadPoolExecutor(max_workers=4) as pool:
        pending={pool.submit(fn,repo,item):(fn.__name__,repo,item.get('id')) for fn,repo,item in jobs}
        for count,f in enumerate(futures.as_completed(pending),1):
            try:f.result()
            except Exception as e:
                failures.append({'job':pending[f],'error':str(e)})
                print(f'ERROR {pending[f]}: {e}',flush=True)
            if count%50==0:print(f'export {count}/{len(jobs)} failures={len(failures)}',flush=True)
    save(ROOT/'export-failures.json',failures)
    if failures:raise RuntimeError(f'{len(failures)} export failures block retirement')

def private_state():
    dest=ROOT/'local'/'host';dest.mkdir(exist_ok=True)
    for name in ['bf','bullet','bulletfarm']:
        p=Path('/home/ubuntu/.local/bin')/name
        shutil.copy2(p,dest/name,follow_symlinks=False)
    with sqlite3.connect('file:/home/ubuntu/.bf/bf.sqlite?mode=ro',uri=True) as src, sqlite3.connect(dest/'bf.sqlite') as dst:
        src.backup(dst)
    for p in [Path('/home/ubuntu/.bf/AGENT_CHAT.md'),Path('/home/ubuntu/bullet/AGENT_CHAT.md.archive')]:
        shutil.copy2(p,dest/p.name)
    shutil.copytree('/home/ubuntu/bullet/AGENT_CHAT.archive',dest/'AGENT_CHAT.archive',dirs_exist_ok=True)
    for n in ['AGENTS.md','README.md','repos.manifest.toml']:
        shutil.copy2(Path('/home/ubuntu/bullet')/n,dest/n)

if __name__=='__main__':
    mode=sys.argv[1]
    if mode=='initial':
        private_state()
        with futures.ThreadPoolExecutor(max_workers=4) as pool:
            for f in [pool.submit(source,r) for r in REPOS]:f.result()
        with futures.ThreadPoolExecutor(max_workers=4) as pool:
            for f in [pool.submit(inventory,r) for r in REPOS]:f.result()
    elif mode=='exports':exports()
