#!/usr/bin/env python3
"""Disposable V1 reproduction; frozen lock and source fixtures are unchanged."""
from pathlib import Path
import subprocess,shutil,hashlib,json,os
root=Path(__file__).resolve().parents[1];dest=root/'.work/v1';dest.mkdir(parents=True,exist_ok=True)
for p in (root/'fixtures/own').rglob('*'):
 if p.is_file():d=dest/p.relative_to(root/'fixtures/own');d.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,d)
lock=hashlib.sha256((dest/'package-lock.json').read_bytes()).hexdigest();npm='npm.cmd'if os.name=='nt'else'npm'
for args in [['ci','--ignore-scripts','--no-audit','--no-fund'],['test'],['run','typecheck']]:subprocess.run([npm]+args,cwd=dest,check=True)
assert hashlib.sha256((dest/'package-lock.json').read_bytes()).hexdigest()==lock
subprocess.run(['node',str(root/'scripts/oracle.cjs')],cwd=root,check=True,stdout=subprocess.DEVNULL)
x=json.loads((root/'fixtures/oracle.json').read_text());print(f'V1 18 tests/typecheck passed; exact lock unchanged; {len(x["vectors"])} actual-utils/helper vectors reproduced with js-beautify {x["version"]}')
