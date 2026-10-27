// Real frozen Surface/Form/Detail/Markdown code, with presentation, unrelated
// grid/list/color-wheel and IPC hooks stubbed. React 18 matches committed host.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const cp=require('node:child_process'),esbuild=require('esbuild');
const root=path.resolve(__dirname,'..'),work=path.join(root,'.work/renderer');fs.mkdirSync(work,{recursive:true});
const {JSDOM}=require('jsdom');const dom=new JSDOM('<!doctype html><html><body></body></html>');
global.window=dom.window;global.document=window.document;global.DOMParser=window.DOMParser;global.Element=window.Element;global.HTMLTextAreaElement=window.HTMLTextAreaElement;
window.tezbar={setSuppressBlurHide:async()=>{},openExternalUrl:async()=>{},clipboardWriteText:async()=>({ok:true})};
const React=require('react'),{create,act}=require('react-test-renderer');global.IS_REACT_ACT_ENVIRONMENT=true;
const source=path.join(root,'fixtures/host/src/renderer/src/raycast-api');
cp.execFileSync(process.env.PYTHON||'python3',[path.join(root,'scripts/smoke.py'),path.join(root,'target/debug/tezbar-json-format'),path.join(work,'production.json')]);
const production=JSON.parse(fs.readFileSync(path.join(work,'production.json')));
// Apply the reviewable patch only to this disposable committed-source copy.
const mirror=path.join(work,'tree/src/renderer/src/raycast-api');fs.mkdirSync(mirror,{recursive:true});fs.copyFileSync(path.join(source,'index.tsx'),path.join(mirror,'index.tsx'));
cp.execFileSync('git',['apply','--directory','v2/.work/renderer/tree',path.join(__dirname,'renderer.patch')],{cwd:path.dirname(root)});
const patched=fs.readFileSync(path.join(mirror,'index.tsx'),'utf8');assert(patched.includes('++draftRevision.current'));
function stubs(){return {name:'isolated-host',setup(b){
 b.onResolve({filter:/^react(\/.*)?$/},a=>({path:require.resolve(a.path),external:true}));
 b.onResolve({filter:/^\.\/form-runtime$/},()=>({path:path.join(source,'form-runtime.tsx')}));
 b.onResolve({filter:/^\.\/detail-runtime$/},()=>({path:path.join(source,'detail-runtime.tsx')}));
 b.onResolve({filter:/ui\/Markdown$/},()=>({path:path.join(root,'fixtures/host/src/renderer/ui/Markdown.tsx')}));
 b.onResolve({filter:/ui\/primitives$/},()=>({path:'ui',namespace:'stub'}));
 b.onResolve({filter:/colorConverter$/},()=>({path:'color',namespace:'stub'}));
 b.onResolve({filter:/^\.\/(action-runtime|action-runtime-registry|grid-runtime|list-runtime|hooks)$/},a=>({path:a.path,namespace:'stub'}));
 b.onLoad({filter:/.*/,namespace:'stub'},a=>{
  const code=a.path==='ui'?`import React from 'react'; export const Hint=({children})=>React.createElement('span',{},children),HintBar=Hint,Kbd=Hint,ViewHeader=Hint; export const cx=(...v)=>v.join(' ');`:
   a.path==='color'?`export const buildColorFormatRows=()=>[];`:
   a.path==='./action-runtime'?`import React from 'react';export const Action=()=>null,ActionPanel=()=>null;export function ActionPanelOverlay(p){return React.createElement('overlay',p)}`:
   a.path==='./action-runtime-registry'?`import React from 'react';export const ActionRegistryContext=React.createContext({});`:
   a.path==='./hooks'?`import React from 'react';export const NavigationContext=React.createContext({});export const setRuntimeContext=()=>{};export const Clipboard={},environment={},getPreferenceValues=()=>({}),open=()=>{},showHUD=()=>{},showToast=()=>{},useCachedPromise=()=>{},useFetch=()=>{},useNavigation=()=>{},usePromise=()=>{};`:
   `export const GridRuntime=()=>null,ListRuntime=()=>null;`;
  return {contents:code,loader:'js'};
 });
}}}
async function build(code,name){const out=path.join(work,name+'.cjs');await esbuild.build({stdin:{contents:code,resolveDir:source,sourcefile:'index.tsx',loader:'tsx'},outfile:out,bundle:true,platform:'node',format:'cjs',jsx:'automatic',nodePaths:[path.join(__dirname,'node_modules')],plugins:[stubs()],external:['react','react/jsx-runtime']});return require(out);}
async function check(surface,expectPatched){
 const calls=[];let tree;const actions=[{id:'format',title:'Format',kind:'submit-form'},{id:'view',title:'View Result',kind:'submit-form'}];
 const form={type:'Form',children:[{type:'Form.TextArea',props:{id:'input',title:'Input',value:''}}]};
 const props={sessionId:'s',title:'Format JSON',extensionId:'com.tezbar.native.json-format',commandName:'index',root:form,actions,onBack:()=>calls.push(['back']),onSearchTextChanged:()=>{},onLoadMore:()=>{},onInvokeAction:(...v)=>calls.push(v)};
 await act(async()=>{tree=create(React.createElement(surface.ExtensionRuntimeSurface,props));});
 await act(async()=>{tree.root.findByType('textarea').props.onChange({target:{value:'newest draft'}});});
 await act(async()=>{tree.root.findByType('form').props.onSubmit({preventDefault(){}})});
 assert.equal(calls[0][0],'format');assert.equal(calls[0][1].input,'newest draft');
 await act(async()=>tree.root.findAllByType('button').find(b=>b.props['aria-label']==='Open actions').props.onClick());
 await act(async()=>tree.root.findByType('overlay').props.onExecute(actions[1]));
 assert.equal(calls[1][0],'view');if(expectPatched)assert.equal(calls[1][1].input,'newest draft');else assert.equal(calls[1][1],undefined);
 // Persistent fields omit value after initial render. Queued local drafts survive
 // stale responses, while descriptions may advance independently.
 const persistent={type:'Form',children:[{type:'Form.TextArea',props:{id:'input',title:'Input',value:'stored',actionId:'change'}}]};
 await act(async()=>tree.update(React.createElement(surface.ExtensionRuntimeSurface,{...props,title:'Value',root:persistent})));
 await act(async()=>tree.root.findByType('textarea').props.onChange({target:{value:'first'}}));
 await act(async()=>tree.root.findByType('textarea').props.onChange({target:{value:'second'}}));
 if(expectPatched){assert.equal(calls.at(-2)[1].__tezbarDraftRevision,1);assert.equal(calls.at(-1)[1].__tezbarDraftRevision,2);}
 const stale={type:'Form',children:[{type:'Form.TextArea',props:{id:'input',title:'Input',actionId:'change'}},{type:'Form.Description',props:{text:'"first"'}}]};
 await act(async()=>tree.update(React.createElement(surface.ExtensionRuntimeSurface,{...props,title:'Value',root:stale})));
 assert.equal(tree.root.findByType('textarea').props.value,'second');
 // Markdown comes from adapter's indented-code policy, not from this harness.
 const detail={...production.detail.root,props:{...production.detail.root.props,backActionId:'return-to-form'}};
 await act(async()=>tree.update(React.createElement(surface.ExtensionRuntimeSurface,{...props,root:detail,actions:[{id:'copy',title:'Copy',kind:'custom'}]})));
 const html=tree.root.findAll(n=>n.props.dangerouslySetInnerHTML)[0].props.dangerouslySetInnerHTML.__html;
 assert(!/<(?:a |img |button[ >])/i.test(html),html);assert(html.includes('&lt;button&gt;evil&lt;/button&gt;'));
 const event={type:'click',nativeEvent:{}};
 await act(async()=>tree.root.findAllByType('button').find(b=>b.children.includes('Enter')).props.onClick(event));
 if(expectPatched)assert.equal(calls.at(-1)[0],'copy');else assert.equal(calls.at(-1)[0],event);
 await act(async()=>tree.root.findAllByType('button').find(b=>b.children.includes('Back')).props.onClick());assert.equal(calls.at(-1)[0],'return-to-form');
 await act(async()=>tree.update(React.createElement(surface.ExtensionRuntimeSurface,{...props,root:production.recovery.root,actions:production.recovery.actions})));
 const errHtml=tree.root.findAll(n=>n.props.dangerouslySetInnerHTML)[0].props.dangerouslySetInnerHTML.__html;assert(!/<(?:a |img |button[ >])/i.test(errHtml),errHtml);
 await act(async()=>tree.unmount());
}
(async()=>{await check(await build(fs.readFileSync(path.join(source,'index.tsx'),'utf8'),'before'),false);await check(await build(patched,'after'),true);console.log('Actual frozen renderer: baseline failures reproduced; patch Form/Detail/draft/Back/Markdown contracts passed (documented stubs).')})().catch(e=>{console.error(e);process.exit(1)});
