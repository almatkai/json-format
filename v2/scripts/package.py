#!/usr/bin/env python3
"""Deterministic schema-3 ZIP. Does not publish, install or modify source pins."""
from pathlib import Path
import argparse,hashlib,json,struct,zipfile,re
ROOT=Path(__file__).resolve().parents[1]
sha=lambda b:hashlib.sha256(b).hexdigest()
def package(binary,target,out,url=None):
 binary=Path(binary).resolve();data=binary.read_bytes();exe='bin/json-format'+('.exe'if target.endswith('msvc')else'')
 if target=='aarch64-apple-darwin':assert data[:4]==b'\xcf\xfa\xed\xfe'and struct.unpack_from('<I',data,4)[0]==0x100000c,'Need real ARM64 Mach-O executable'
 elif target=='x86_64-pc-windows-msvc':
  assert data[:2]==b'MZ';offset=struct.unpack_from('<I',data,0x3c)[0];assert data[offset:offset+4]==b'PE\0\0'and struct.unpack_from('<H',data,offset+4)[0]==0x8664,'Need real x64 PE executable'
 else:raise ValueError('Unsupported publication target')
 members={exe:data,'LICENSE':(ROOT/'LICENSE').read_bytes(),'NOTICE':(ROOT/'NOTICE').read_bytes()}
 for p in sorted((ROOT/'resources').rglob('*')):
  assert not p.is_symlink(),'Package symlinks forbidden'
  if p.is_file():members[p.relative_to(ROOT).as_posix()]=p.read_bytes()
 manifest=json.loads((ROOT/'extension.json').read_text());manifest['files']={k:sha(v)for k,v in sorted(members.items())}
 members['extension.json']=(json.dumps(manifest,ensure_ascii=False,indent=2,sort_keys=True)+'\n').encode()
 out=Path(out);out.mkdir(parents=True,exist_ok=True);name=f'json-format-{manifest["version"]}-{target}.zip';archive=out/name
 with zipfile.ZipFile(archive,'w',compression=zipfile.ZIP_STORED)as z:
  for key,value in sorted(members.items()):
   info=zipfile.ZipInfo(key,(1980,1,1,0,0,0));info.create_system=3;info.external_attr=(0o100755 if key==exe else 0o100644)<<16;z.writestr(info,value)
 digest={'target':target,'archive':name,'sha256':sha(archive.read_bytes()),'executable':exe,'executableSha256':sha(data),'files':manifest['files']}
 (out/(name+'.digests.json')).write_text(json.dumps(digest,indent=2,sort_keys=True)+'\n')
 if url:
  assert re.fullmatch(r'https://[^@]+',url),'HTTPS artifact URL required'
  reference=json.loads((ROOT/'extension.json').read_text());reference['artifacts']={target:{**{k:digest[k]for k in ['sha256','executable','executableSha256']},'url':url}}
  (out/(name+'.reference.json')).write_text(json.dumps(reference,indent=2,sort_keys=True)+'\n')
 print(json.dumps({k:v for k,v in digest.items()if k!='files'}));return archive,digest
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--binary',required=True);p.add_argument('--target',required=True);p.add_argument('--out',default=str(ROOT/'artifacts'));p.add_argument('--url');a=p.parse_args();package(a.binary,a.target,a.out,a.url)
