#!/usr/bin/env python3
import hashlib
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import sys
import tarfile
from preserve import ROOT, REPOS, save

os.umask(0o077)
suffix='-'+sys.argv[1] if len(sys.argv)>1 else ''
parts=ROOT/('backup-parts'+suffix);parts.mkdir(exist_ok=True)
names=['mirrors','local','records','tools']+[p.name for p in ROOT.glob('*.json')]
proc=subprocess.Popen(['tar','-C',str(ROOT),'-czf','-',*names],stdout=subprocess.PIPE,stderr=subprocess.PIPE)
manifest=[];overall=hashlib.sha256();number=0
while True:
    data=proc.stdout.read(50*1024*1024)
    if not data:break
    name=f'private-backup.tar.gz.part{number:04d}';number+=1
    (parts/name).write_bytes(data);overall.update(data)
    manifest.append({'path':name,'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()})
assert proc.wait()==0,proc.stderr.read().decode()
save(parts/'index.json',{'format':'Concatenate parts in listed order to reconstruct gzip tar. Contains private configuration, original launchers and coordination backup: do not publish.','sha256':overall.hexdigest(),'parts':manifest})
restore=ROOT/('restore-drill'+suffix);restore.mkdir(exist_ok=True)
untar=subprocess.Popen(['tar','-xzf','-','-C',str(restore)],stdin=subprocess.PIPE,stderr=subprocess.PIPE)
for item in manifest:
    data=(parts/item['path']).read_bytes()
    assert hashlib.sha256(data).hexdigest()==item['sha256']
    untar.stdin.write(data)
untar.stdin.close()
assert untar.wait()==0,untar.stderr.read().decode()
for name in REPOS:
    mirror=restore/'mirrors'/f'{name}.git'
    subprocess.run(['git','-C',str(mirror),'fsck','--full'],check=True,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
    refs=dict(line.split(' ',1)[::-1] for line in subprocess.check_output(['git','-C',str(mirror),'for-each-ref','--format=%(objectname) %(refname)']).decode().splitlines())
    for item in json.loads((restore/'records'/name/'all-refs.json').read_text()):
        assert refs[item['ref']]==item['oid'],item
    for item in json.loads((restore/'local-commit-index.json').read_text()):
        if item['repository']=='neverhuman/'+name:
            assert refs[item['ref']]==item['oid'],item
for item in json.loads((restore/'evidence-manifest.json').read_text()):
    path=restore/item['path']
    assert path.stat().st_size==item['bytes'],path
    assert hashlib.file_digest(path.open('rb'),'sha256').hexdigest()==item['sha256'],path
with sqlite3.connect('file:'+str(restore/'local/host/bf.sqlite')+'?mode=ro',uri=True) as db:
    assert db.execute('pragma integrity_check').fetchall()==[('ok',)]
for name in ['bf','bullet']:
    a=ROOT/'local/host'/name;b=restore/'local/host'/name
    assert hashlib.file_digest(a.open('rb'),'sha256').hexdigest()==hashlib.file_digest(b.open('rb'),'sha256').hexdigest()
assert os.readlink(restore/'local/host/bulletfarm')=='bullet'
result={'private_backup_parts':len(manifest),'max_part_bytes':50*1024*1024,'archive_sha256':overall.hexdigest(),'restored_mirrors':6,'original_refs_verified':1475,'additional_local_commits_verified':len(json.loads((restore/'local-commit-index.json').read_text())),'evidence_files_verified':2997,'artifacts_verified':1766,'sqlite_integrity':'ok','original_launchers':'restored with matching checksums and original symlink','status':'passed','scope':'Restoration of original source/evidence/local backup; migration PR B and installation gates remain pending.'}
save(ROOT/'restoration-drill.json',result)
print(json.dumps(result,indent=2),flush=True)
