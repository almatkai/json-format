#!/usr/bin/env python3
"""Separate host prerequisite: preserve verified process resource directory.
Writes patch + exact before/after icon function harness; never touches live host.
"""
from pathlib import Path
import difflib,re,sys
root=Path(__file__).resolve().parents[1]
p='src-tauri/src/native_core/extensions/registry.rs';before=(root/'fixtures/host'/p).read_text();after=before
# Keep .path as the managed preferences/discovery root. Only resources use the
# directory already verified by process_install::resolve.
after=after.replace('    path: PathBuf,\n    manifest:', '    path: PathBuf,\n    resource_dir: Option<PathBuf>,\n    manifest:',1)
after=after.replace('            let mut manifest = if native {','            let mut resource_dir = None;\n            let mut manifest = if native {',1)
after=after.replace('                    Ok(package) => package.manifest,','                    Ok(package) => {\n                        resource_dir = Some(package.dir);\n                        package.manifest\n                    },',1)
after=after.replace('''                path,
                manifest,''','''                path,
                resource_dir,
                manifest,''',1)
# The second literal constructs preference-only metadata, not a process package.
start=after.index('    let extension = Installed {')
pos=after.index('        manifest:',start);after=after[:pos]+'        resource_dir: None,\n'+after[pos:]
needle='''    let mut candidates = vec![
        extension.path.join(relative),
        extension.path.join("assets").join(relative),
    ];'''
assert after.count(needle)==2
after=after.replace(needle,'''    let resource_root = extension.resource_dir.as_deref().unwrap_or(&extension.path);
    let mut candidates = vec![
        resource_root.join(relative),
        resource_root.join("assets").join(relative),
    ];''')
after+='''
#[cfg(test)]
mod process_icon_resource_tests {
    use super::*;
    #[test]
    fn verified_process_icons_use_version_resources_without_moving_preferences() {
        let temp = tempfile::tempdir().unwrap();
        let managed = temp.path().join("packages/com.tezbar.native.example");
        let version = managed.join("versions/1.0.0-fixture");
        std::fs::create_dir_all(version.join("resources")).unwrap();
        let icon = version.join("resources/command-icon.png");
        std::fs::write(&icon, b"icon fixture").unwrap();
        let extension = Installed {
            id: "com.tezbar.native.example".into(), slug: "example".into(),
            path: managed.clone(), resource_dir: Some(version), data_dir: temp.path().into(),
            manifest: json!({"icon":"resources/command-icon.png"}),
        };
        assert_eq!(icon_path(&extension).as_deref(), icon.to_str());
        assert_eq!(command_icon_path(&extension, &json!({"icon":"resources/command-icon.png"})).as_deref(), icon.to_str());
        assert_eq!(extension.path, managed);
    }
}
'''
patch=''.join(difflib.unified_diff(before.splitlines(True),after.splitlines(True),fromfile='a/'+p,tofile='b/'+p))
def fn(s,name):
 m=re.search(r'(?m)^fn '+name+r'(?:<|\()',s);assert m;return s[m.start():s.index('\n}',m.start())+2]
def struct(s):
 start=s.index('pub(super) struct Installed {');return s[start:s.index('\n}',start)+2]
code='// @generated from frozen host registry, Apache-2.0; wrappers are test-owned.\n'
for name,source in [('before',before),('after',after)]:
 code+='#[allow(dead_code,clippy::all)]\npub mod '+name+' {\nuse serde_json::Value;use std::path::{Path,PathBuf,Component};\n'+struct(source)+'\n'+'\n'.join(fn(source,f)for f in ['fallback','icon_path','command_icon_path'])+'\n'
 code+='pub fn icons(path:PathBuf,'+('resource'if name=='after'else'_resource')+':PathBuf,manifest:Value,command:Value)->(Option<String>,Option<String>,PathBuf){\n let extension=Installed{id:"com.tezbar.native.json-format".into(),slug:"json-format".into(),path,'+('resource_dir:Some(resource),'if name=='after'else'')+'manifest,data_dir:PathBuf::new()};\n (icon_path(&extension),command_icon_path(&extension,&command),extension.path)\n}\n}\n'
for path,content in [(root/'integration/registry-icon.patch',patch),(root/'fixtures/icon-contract.rs',code)]:
 if '--check'in sys.argv:assert path.read_text()==content,path
 else:path.write_text(content)
print('Separate process registry icon patch and actual before/after contract extraction verified')
