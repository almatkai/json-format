#!/usr/bin/env python3
"""No Node/host/sibling repository needed: drive the production executable."""
import json,subprocess,uuid,tempfile,os,sys,queue,threading
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
class Pipe:
 def __init__(self,binary,base):
  self.p=subprocess.Popen([str(Path(binary).resolve())],cwd=base,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,bufsize=0)
  self.frames=queue.Queue(maxsize=8)
  def pump():
   while True:
    line=self.p.stdout.readline(16*1024*1024+2)
    self.frames.put(line)
    if not line:break
  self.reader=threading.Thread(target=pump,daemon=True);self.reader.start()
  self.base=Path(base);self.hello=self.recv();assert self.hello['id']=='com.tezbar.native.json-format'
 def send(self,value):self.p.stdin.write(json.dumps(value,ensure_ascii=True,separators=(',',':')).encode()+b'\n');self.p.stdin.flush()
 def recv(self):
  line=self.frames.get(timeout=8);assert line and line.endswith(b'\n')and len(line)<=16*1024*1024+1,'invalid frame';assert b'\r'not in line;return json.loads(line)
 def request(self,method,params):
  key='h:'+str(uuid.uuid4());self.send(dict(type='request',id=key,method=method,params=params));return key
 def response(self,key):
  v=self.recv();assert v['type']=='response'and v['id']==key,v;return v
 def call(self,method,params):return self.response(self.request(method,params))
 def reply(self,rpc,result=None,error=None):
  self.send(dict(type='response',id=rpc['id'],**({'error':{'code':'fixture','message':error}}if error is not None else {'result':result})))
 def init(self):
  v=self.call('initialize',dict(protocol=1,extensionId='com.tezbar.native.json-format',platform='macos',packageDir=str(self.base),dataDir=str(self.base/'data'),tempDir=str(self.base/'temp'),grantedCapabilities=['selection.read','clipboard.write'],reason='isolated-test',migrationPaths={}))
  assert v['result']['ready']
 def finish(self):
  self.call('shutdown',{});self.p.stdin.close();assert self.p.wait(timeout=8)==0;assert not self.p.stderr.read();self.reader.join(timeout=2);assert not self.reader.is_alive()
def run(binary,output=None):
 (ROOT/'.work').mkdir(exist_ok=True)
 with tempfile.TemporaryDirectory(prefix='smoke-',dir=ROOT/'.work')as base:
  p=Pipe(binary,base);p.init()
  key=p.request('session.open',dict(sessionId='selected',commandName='formatSelectionAndShow',preferences={'indent':'2'}))
  rpc=p.recv();assert rpc['method']=='host.selection.read'
  evil='```\n![evil](https://evil.test/icon.png)\n[evil](https://evil.test)\n<button>evil</button>\n```'
  p.reply(rpc,json.dumps({'text':evil}));view=p.response(key)['result'];assert view['root']['type']=='Detail'
  key=p.request('session.call',dict(sessionId='selected',operation='extension:invoke-action',payload={'actionId':view['actions'][0]['id']}))
  rpc=p.recv();assert rpc['method']=='host.clipboard.write';assert 'text'in rpc['params']
  p.reply(rpc,error=evil);error=p.response(key)['result'];assert error['title']=='Delivery not confirmed'
  assert 'exact prepared bytes'in error['root']['props']['markdown']
  if output:Path(output).write_text(json.dumps({'detail':view,'recovery':error},indent=2)+'\n')
  p.finish()
 print('Isolated production JSONL smoke passed, no GUI OS effects used')
if __name__=='__main__':run(sys.argv[1],sys.argv[2]if len(sys.argv)>2 else None)
