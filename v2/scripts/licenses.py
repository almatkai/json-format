#!/usr/bin/env python3
"""Bundle actual license/notice files for the locked production dependency graph."""
from pathlib import Path
import json,subprocess,hashlib
root=Path(__file__).resolve().parents[1]
packages={};seen=set();items=[]
for target in ['aarch64-apple-darwin','x86_64-pc-windows-msvc']:
 d=json.loads(subprocess.check_output(['cargo','metadata','--locked','--offline','--format-version','1','--filter-platform',target],cwd=root))
 packages.update({p['id']:p for p in d['packages']});nodes={n['id']:n for n in d['resolve']['nodes']};visited=set()
 def visit(key):
  if key in visited:return
  visited.add(key);seen.add(key)
  for dep in nodes[key]['deps']:
   if any(k['kind'] is None for k in dep['dep_kinds']):visit(dep['pkg'])
 visit(d['resolve']['root'])
for key in sorted(seen):
 p=packages[key]
 if p['name']=='tezbar-json-format':continue
 src=Path(p['manifest_path']).parent
 files=[q for q in src.iterdir()if q.is_file()and q.name.upper().startswith(('LICENSE','COPYING','NOTICE'))]
 assert files,(p['name'],'missing real license file')
 entry={k:p[k]for k in ['name','version','license','source']};entry['files']={}
 for f in sorted(files):
  dest=root/'resources/licenses/cargo'/f'{p["name"]}-{p["version"]}'/f.name;dest.parent.mkdir(parents=True,exist_ok=True);data=f.read_bytes();dest.write_bytes(data);entry['files'][dest.relative_to(root).as_posix()]=hashlib.sha256(data).hexdigest()
 items.append(entry)
(root/'resources/licenses/cargo-index.json').write_text(json.dumps(items,indent=2,sort_keys=True)+'\n');print(f'Bundled original license/notice files for {len(items)} locked production dependencies')
