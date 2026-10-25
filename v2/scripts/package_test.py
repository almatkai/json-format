#!/usr/bin/env python3
"""Extracted executable acceptance with sanitized environment and no Node/host."""
from pathlib import Path
import json,sys,zipfile,tempfile,shutil,subprocess,hashlib,os
root=Path(__file__).resolve().parents[1];archive=Path(sys.argv[1]).resolve();digest=json.loads(archive.with_name(archive.name+'.digests.json').read_text())
assert hashlib.sha256(archive.read_bytes()).hexdigest()==digest['sha256']
(root/'.work').mkdir(exist_ok=True)
with tempfile.TemporaryDirectory(prefix='package-',dir=root/'.work')as base:
 base=Path(base);dest=base/'package';dest.mkdir()
 with zipfile.ZipFile(archive)as z:
  assert all(not n.startswith('/')and'..'not in Path(n).parts and'\\'not in n for n in z.namelist())
  z.extractall(dest)
 manifest=json.loads((dest/'extension.json').read_text());members={p.relative_to(dest).as_posix()for p in dest.rglob('*')if p.is_file()}
 assert members==set(manifest['files'])|{'extension.json'}
 for n,h in manifest['files'].items():assert hashlib.sha256((dest/n).read_bytes()).hexdigest()==h,n
 assert manifest['files']==digest['files'];exe=dest/digest['executable'];exe.chmod(0o755)
 assert (dest/manifest['icon']).is_file();icon=(dest/manifest['icon']).read_bytes();assert len(icon)==237176 and hashlib.sha256(icon).hexdigest()=='cfee8098d945a0541c391777deeae1fc6e3ddee07a8516f1f730b2103e4e8f13'
 # Smoke script reads only stdlib + executable. Copy it into the isolated tree;
 # no original source fixture, host, sibling repo or Node is on its runtime path.
 harness=base/'scripts';harness.mkdir();shutil.copyfile(root/'scripts/smoke.py',harness/'smoke.py')
 env={k:v for k,v in os.environ.items()if k in ['SystemRoot','WINDIR','TEMP','TMP','USERPROFILE','HOME']};env['PATH']=os.defpath
 if len(sys.argv)>2:
  test_env=dict(env,JSON_FORMAT_PACKAGE_DIR=str(dest));subprocess.run([str(Path(sys.argv[2]).resolve()),'--ignored','--exact','extracted_package_against_actual_host_validator'],cwd=dest,env=test_env,check=True)
 subprocess.run([sys.executable,str(harness/'smoke.py'),str(exe)],cwd=dest,env=env,check=True)
 print('Extracted ZIP: complete inventory/digests, offline original PNG and real production executable smoke passed')
