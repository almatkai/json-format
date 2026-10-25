#!/usr/bin/env python3
"""Inventory the immutable input snapshots; never read live host working files."""
from pathlib import Path
import hashlib,json,sys,base64
root=Path(__file__).resolve().parents[1];refs=json.loads((root/'fixtures/refs.json').read_text());files={}
for group in ['own','host','sdk','upstream','beautifier']:
 for p in sorted((root/'fixtures'/group).rglob('*')):
  if p.is_file():
   b=p.read_bytes();files[p.relative_to(root).as_posix()]={'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest(),'gitBlobSha1':hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()}
d={'refs':refs,'files':files};target=root/'fixtures/provenance.json'
if '--check'in sys.argv:assert json.loads(target.read_text())==d,'Frozen reference bytes changed'
else:target.write_text(json.dumps(d,indent=2,sort_keys=True)+'\n')
icon=(root/'resources/command-icon.png').read_bytes();assert len(icon)==237176 and hashlib.sha256(icon).hexdigest()=='cfee8098d945a0541c391777deeae1fc6e3ddee07a8516f1f730b2103e4e8f13'
p=json.loads((root/'fixtures/beautifier/provenance.json').read_text());archive=(root/'fixtures/beautifier/js-beautify-1.15.1.tgz').read_bytes();assert p['integrity']=='sha512-'+base64.b64encode(hashlib.sha512(archive).digest()).decode()
toml=(root/'Cargo.toml').read_text();assert refs['sdk']in toml and 'path ='not in toml and 'tauri'not in toml.lower()
lock=(root/'Cargo.lock').read_text();assert 'git+https://github.com/almatkai/tezbar-extension-sdk-rust.git?rev='+refs['sdk']+'#'+refs['sdk']in lock
print(f'Pinned own/host/SDK/upstream/beautifier inputs: {len(files)} files verified; icon and git dependency verified')
