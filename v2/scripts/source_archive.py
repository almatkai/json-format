#!/usr/bin/env python3
"""Snapshot indexed source for an independent build; never includes caches."""
from pathlib import Path
import json,hashlib,zipfile,sys
root=Path(__file__).resolve().parents[2];items=json.loads((root/'v2/INDEX.json').read_text())['files'];out=Path(sys.argv[1]).resolve();out.parent.mkdir(parents=True,exist_ok=True)
with zipfile.ZipFile(out,'w',compression=zipfile.ZIP_STORED)as z:
 for row in items+[{'path':'v2/INDEX.json'}]:
  p=root/row['path'];data=p.read_bytes()
  if'sha256'in row:assert hashlib.sha256(data).hexdigest()==row['sha256']
  info=zipfile.ZipInfo(row['path'],(1980,1,1,0,0,0));info.create_system=3;info.external_attr=0o100644<<16;z.writestr(info,data)
print(hashlib.sha256(out.read_bytes()).hexdigest(),out.name)
