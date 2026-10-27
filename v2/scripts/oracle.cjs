// Executes frozen own TS/TSX and npm-locked js-beautify. Raycast/React are explicit
// headless stubs; they capture callbacks/effects, never assert native desktop QA.
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const { createRequire } = require('node:module');
const assert = require('node:assert/strict');
const root = path.resolve(__dirname, '..');
const dep = createRequire(path.join(root, '.work/v1/package.json'));
const ts = dep('typescript');
const beautify = dep('js-beautify');
const lock = JSON.parse(fs.readFileSync(path.join(root,'fixtures/own/package-lock.json')));
const version = lock.packages['node_modules/js-beautify'].version;
assert.equal(dep('js-beautify/package.json').version, version);
let prefs={indent:'tab',autopaste:false,maxInitialLines:'10000'}, effects=[], input='', text='', showAll=false;
let localText=undefined, pushed=[], pending=[];
const jsx=(type,props)=>({type:typeof type==='string'?type:type?.name||'Fragment',props:props||{}});
const component=(name)=>Object.assign(()=>{}, {displayName:name});
const api={
 Clipboard:{readText:async()=>text,copy:async t=>effects.push(['copy',t]),paste:async t=>effects.push(['paste',t])},
 getSelectedText:async()=>{effects.push(['selection']);return text;},
 closeMainWindow:async()=>effects.push(['close']),showHUD:async(...a)=>effects.push(['hud',...a]),
 showToast:async(...a)=>effects.push(['toast',...a]),
 getPreferenceValues:()=>prefs,environment:{launchType:'UserInitiated'},LaunchType:{Background:'Background'},
 Toast:{Style:{Failure:'failure',Success:'success'}}, Icon:{Clipboard:'clipboard',Eye:'eye'},
 useNavigation:()=>({push:t=>pushed.push(t)}),
 Action:Object.assign(component('Action'),{SubmitForm:'Action.SubmitForm',Paste:'Action.Paste',CopyToClipboard:'Action.CopyToClipboard'}),
 ActionPanel:Object.assign(component('ActionPanel'),{Section:'ActionPanel.Section'}),
 Form:Object.assign(component('Form'),{TextArea:'Form.TextArea',Description:'Form.Description'}),Detail:'Detail'
};
const react={useMemo:f=>f(),useState:()=>[showAll,x=>{showAll=x}]};
const utils={
 useForm:opts=>({values:{input},itemProps:{input:{id:'input',value:input}},handleSubmit:opts.onSubmit}),
 useLocalStorage:()=>({value:localText,setValue:v=>{localText=v},isLoading:false}),
 usePromise:f=>{pending.push(f());return {data:undefined,isLoading:true}}
};
const cache=new Map();
function load(file){
 file=path.resolve(root,'fixtures/own/src',file);
 if(!path.extname(file))file+=fs.existsSync(file+'.tsx')?'.tsx':'.ts';
 if(cache.has(file))return cache.get(file).exports;
 const mod={exports:{}};cache.set(file,mod);
 const src=ts.transpileModule(fs.readFileSync(file,'utf8'),{compilerOptions:{module:ts.ModuleKind.CommonJS,jsx:ts.JsxEmit.ReactJSX,esModuleInterop:true,target:ts.ScriptTarget.ES2022}}).outputText;
 const req=id=>id==='@raycast/api'?api:id==='@raycast/utils'?utils:id==='react'?react:id==='react/jsx-runtime'?{jsx,jsxs:jsx,Fragment:'Fragment'}:id.startsWith('.')?load(path.resolve(path.dirname(file),id)):dep(id);
 vm.runInNewContext('(function(require,module,exports){'+src+'\n})',{console:{error:()=>{}}},{filename:file})(req,mod,mod.exports);
 return mod.exports;
}
const actual=load('utils.ts'),pure=load('lib/json.ts');
function clean(v){return v===undefined?null:v}
async function invoke(f){effects=[];try{return {value:clean(await f()),effects}}catch(e){return {value:null,error:e.name+': '+e.message,effects}}}
const cases=[
 '', ' ', '{}','[]','null','true','false','0','-0','1e+02','1E-7','1e400','9007199254740993',
 '{"a":1,"b":[2,3]}','{"b":1,"a":2,"b":3,"10":1,"2":2,"01":3}',
 '{"__proto__":{"a":1},"x":2}','{"x":-0,"n":1.00,"e":1E+3,"v":9007199254740993}',
 '[1,2,3]','[1,\n2,\n3]','[\n1,\n2\n]','[[1,2],[3,4]]','[{"a":1},{"b":2}]',
 '{\n\n"a":1,\n\n"b":2\n}', '{"a":[],"b":{},"c":[true,null,"x"]}',
 '"{\\"a\\":1}"','"[1,2]"','"hello"','""','" \\n "','"\\ud800"',
 '{"str":"a \\u0041 \\/ \\b \\n \\t \\uD83D\\uDE00 \\ud800 \\udc00"}',
 '{\\"a\\":1}','{\\"a\\":\\"hi\\"}','{a:1,b:\'hi\',}',
 '{a:[1,2,],b:{z:true,},}',"{'2':'two','1':'one',z:-0,a:1e2}",
 '{a:1,a:2,__proto__:{ignored:true},z:undefined,b:NaN,c:Infinity}',
 '[1,,3,undefined,NaN,Infinity,-Infinity]', '+2','0x10','-0x10','0b11','0o17','.5','1.',
 '{a:"paren()"}','{a:`hello`}','{"a":1};',' [1,2] ','[1,2];','null;',
 'no json {{{','{"a":}', '"bad', '{a:unknownVariable}', '(function(){})()', 'tag`x`','1+2','[...[]]'
];
// Deterministic syntax/whitespace corpus, evaluated by the genuine beautifier.
const atoms=['0','-0','1.2300','1e-7','1e21','9007199254740993','true','null','"x"','{}','[]','{"a":1}','[1,2]'];
for(const a of atoms)for(const b of ['2','{}','[3]'])for(const ws of ['','\n'])cases.push('['+ws+a+','+ws+b+ws+']');
for(const src of ['[1,2\n]','[{},{}]','[{},{"a":1}]','[1,{"a":2},3]','{"a":\n1}','{"a":\n{"b":2}}','{"a":[[1],[2]]}','{"a":[{"b":2},{"c":3}]}','[1,\n\n2]','[,]', '[,,]', '[1,,]', '{"a":1,}', '{__proto__:null,x:1}', '{__proto__:1,x:2}', "{['a']:1}",'{a:1,__proto__:{},__proto__:null}','1_000','1e-324','5e-324','-1e400','{"s":"\\u2028\\u2029"}','{a:"\\x41",b:"\\u{1f600}"}','`hi`'])cases.push(src);
async function main(){
 const vectors=[];
 for(const indent of ['tab','2','4','8'])for(const src of cases){
  prefs={...prefs,indent};const a=await invoke(()=>actual.formatJS(src));const h=await invoke(()=>pure.formatJSLogic(src,indent));
  vectors.push({kind:'format',input:src,indent,actual:a,helper:h});
 }
 for(const src of ['[]','[1,{"a":2},"hi"]','[{"a":1},{"b":2}]','[{"a":1},{"b":2},1,"a",true,null]',
  '["a","b",{"a":1,"b":2},[1,2]]','[{"z":1,"2":2,"1":1,"z":3},-0,1e400,9007199254740993]',
  '["\\ud800","\\ud83d\\ude00"]',' [1,2] ','[1,2];','[1,]','null','{}']){
  vectors.push({kind:'lines',input:src,actual:await invoke(()=>actual.formatToJSONLines(src)),helper:await invoke(()=>pure.formatToJSONLinesPure(src))});
 }
 for(const t of ['', 'hello','a "quote"','a\nb\r\t\0','😀','\ud800','```\n![x](https://x)\n```'])vectors.push({kind:'value',input:t,actual:{value:JSON.stringify(t)}});
 for(const v of vectors){if(v.input && /[\ud800-\udfff]/u.test(v.input) && !v.input.isWellFormed()){v.inputUtf16=Array.from({length:v.input.length},(_,i)=>v.input.charCodeAt(i));delete v.input;}}
 fs.writeFileSync(path.join(root,'fixtures/oracle.json'),JSON.stringify({version,vectors},null,2)+'\n');
 // Actual TSX command callback paths; dependencies are the documented stubs above.
 const actions=[];
 for(const autopaste of [false,true]){
  prefs={indent:'2',autopaste,maxInitialLines:'2'};text='{"a":1}';
  for(const name of ['formatClipboard','formatSelection'])actions.push({name,autopaste,result:await invoke(()=>load(name+'.tsx').default())});
  for(const name of ['index','formatJsonLines'])for(const actionIndex of [0,1]){
   input=name==='index'?'{"a":1}':'[{"a":1},{"b":2}]';pushed=[];const tree=load(name+'.tsx').default();const panel=tree.props.actions;
   const acts=panel.props.children;const a=acts[actionIndex];const result=await invoke(()=>a.props.onSubmit());
   actions.push({name,autopaste,actionIndex,result,pushed:pushed.map(t=>load('formattedJsonDetail.tsx').FormattedJsonDetail(t.props))});
  }
 }
 const valueComp=load('formatToJsonValue.tsx').default;localText='persisted\n"text"';const vt=valueComp();
 assert.equal(vt.props.children[1].props.text,JSON.stringify(localText));vt.props.children[0].props.onChange('next');assert.equal(valueComp().props.children[1].props.text,'"next"');
 const treeComp=load('jsonTreeView.tsx').JsonTreeView;const views=[];
 for(const maxInitialLines of ['2','0','bogus','-1','-100','  +2suffix','1e2']){
  prefs={...prefs,maxInitialLines};showAll=false;const first=treeComp({json:'1\n2\n3\n4'});showAll=true;const all=treeComp({json:'1\n2\n3\n4'});
  views.push({maxInitialLines,first,all});
 }
 load('formatSelectionAndShow.tsx').default();await Promise.all(pending);
 fs.writeFileSync(path.join(root,'fixtures/action-oracle.json'),JSON.stringify({actions,views,storedText:'next',stubs:['Raycast host services','React hooks','useForm','useLocalStorage','usePromise']},null,2)+'\n');
 console.log(`${vectors.length} frozen vectors; js-beautify ${version}; ${actions.length} command paths`);
 console.log(JSON.stringify(vectors.filter(v=>v.kind==='lines'),null,2));
}
main().catch(e=>{console.error(e);process.exit(1)});
