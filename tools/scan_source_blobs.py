#!/usr/bin/env python3
import json
import re
import subprocess
from preserve import ROOT, save

r='/home/ubuntu/bulletfarm'
refs=subprocess.check_output(['git','-C',r,'for-each-ref','--format=%(refname)','refs/tags/archive'])
objects=subprocess.check_output(['git','-C',r,'rev-list','--objects','--stdin'],input=refs).decode().splitlines()
paths={l.split(' ',1)[0]:l.split(' ',1)[1] if ' ' in l else '' for l in objects}
proc=subprocess.Popen(['git','-C',r,'cat-file','--batch'],stdin=subprocess.PIPE,stdout=subprocess.PIPE)
secret=re.compile(rb'(?:gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{40,}|-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----|AKIA[0-9A-Z]{16})')
canaries={'2c6829f9daf90669841457c1df1e9a5a6ec4138f','5d75502f16281cc7b61483aa37c511ef894fe7fa','ff0d6ca539d56198dd28877580bea585c13da263','5e5e95936ce9b1c16322394a0e73eebd656c7fee'}
findings=[];known=[];blobs=0
for oid,path in paths.items():
    proc.stdin.write((oid+'\n').encode());proc.stdin.flush()
    header=proc.stdout.readline().decode().split();data=proc.stdout.read(int(header[2]));proc.stdout.read(1)
    if header[1]=='blob':
        blobs+=1
        if secret.search(data):
            (known if oid in canaries else findings).append({'oid':oid,'path':path})
proc.stdin.close();proc.wait()
report={'blobs_scanned':blobs,'findings':findings,'reviewed_test_canaries':known,'review':'Allowlisted four exact historical redaction-test blobs; no private configuration or real credential is allowlisted.'}
save(ROOT/'source-secret-scan-final.json',report)
print(json.dumps({'blobs_scanned':blobs,'unreviewed_findings':len(findings),'historical_test_canaries':len(known)}))
if findings:raise SystemExit('Review findings privately before publication')
