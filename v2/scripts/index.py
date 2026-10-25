#!/usr/bin/env python3
"""Inventory all Git-visible review deliverables, including untracked fixtures.
Build/disposable outputs (.work, target, artifacts, node_modules) are not delivered.
Parent must regenerate after final edits/staging; INDEX excludes only itself.
"""
from pathlib import Path
import json,hashlib,subprocess,sys
root=Path(__file__).resolve().parents[2];dest=root/'v2/INDEX.json'
paths=subprocess.check_output(['git','ls-files','--cached','--others','--exclude-standard','-z'],cwd=root).decode().split('\0')
items=[]
for name in sorted(set(paths)):
 if not name or(name!='README.md'and not name.startswith('v2/'))or name=='v2/INDEX.json':continue
 p=root/name;assert p.is_file()and not p.is_symlink(),name;b=p.read_bytes();items.append({'path':name,'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()})
# Ensure required fixtures are Git-visible; a broad dist/node_modules ignore must
# never accidentally exclude a fixture or a reviewable generated patch.
visible={i['path']for i in items}
for folder in ['fixtures','src','tests','scripts','resources','integration','ci']:
 for p in (root/'v2'/folder).rglob('*'):
  if p.is_file()and not any(x in ['node_modules','__pycache__']for x in p.parts):assert p.relative_to(root).as_posix()in visible,p
value={'scope':'README.md plus v2 source/review deliverables; excludes INDEX itself and disposable build outputs','files':items}
if '--check'in sys.argv:assert json.loads(dest.read_text())==value,'INDEX stale: regenerate after final edits/staging'
else:dest.write_text(json.dumps(value,indent=2)+'\n')
print(f'INDEX: {len(items)} Git-visible deliverables, {sum(i["bytes"]for i in items)} bytes verified')
