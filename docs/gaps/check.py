#!/usr/bin/env python3
"""Validate docs/gaps/register.json: every BF3-001..030 criterion and every AT/HF/CF scenario exactly once,
unique row_id per row, counts equal to the rows. Run: python3 docs/gaps/check.py"""
import json,collections,sys,os
p=os.path.join(os.path.dirname(__file__),'register.json'); reg=json.load(open(p)); rows=reg['rows']
expected=set(f'BF3-{n:03d}-AC0{k}' for n in range(1,31) for k in (1,2,3))|set(f'AT-{n:03d}' for n in range(1,53))|set(f'HF{n:02d}' for n in range(1,21))|set(f'CF0{n}' for n in range(1,9))
ids=collections.Counter(r['id'] for r in rows if r['id'] in expected)
missing=sorted(expected-set(ids)); dup=sorted(k for k,v in ids.items() if v>1)
rid=collections.Counter(r.get('row_id') for r in rows); baddup=sorted(k for k,v in rid.items() if v>1 or not k)
counts=collections.Counter(r['status'] for r in rows)
ok = not missing and not dup and not baddup and dict(counts)==dict(reg['counts'])
print(f"rows={len(rows)} canonical={sum(ids.values())}/170 missing={missing} duplicates={dup} bad_row_ids={baddup} counts_match={dict(counts)==dict(reg['counts'])}")
sys.exit(0 if ok else 1)
