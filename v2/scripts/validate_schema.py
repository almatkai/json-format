#!/usr/bin/env python3
from pathlib import Path
from copy import deepcopy
import json
from jsonschema import Draft202012Validator
root=Path(__file__).resolve().parents[1]
schema=json.loads((root/'fixtures/host/schemas/extension-v3.schema.json').read_text())
m=json.loads((root/'extension.json').read_text());v=Draft202012Validator(schema);v.validate(m)
count=0
for key,bad in [('schemaVersion',2),('runtime','javascript'),('execution','builtin'),('protocolVersion',2),('capabilities',['core.any']),('icon',None),('id','raycast.json-format'),('artifacts',{'x86_64-pc-windows-gnu':{}})]:
 b=deepcopy(m);b[key]=bad;assert list(v.iter_errors(b)),key;count+=1
for bad in [None,{}, {'url':'http://bad','sha256':'0'*64,'executable':'../escape','executableSha256':'0'*64}]:
 b=deepcopy(m);b['artifacts']={'aarch64-apple-darwin':bad};assert list(v.iter_errors(b));count+=1
print(f'Exact committed schema 3: source valid, empty dev artifacts allowed; {count} negative cases rejected')
