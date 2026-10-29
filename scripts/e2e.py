#!/usr/bin/env python3
"""Launch isolated real services, run Rust HTTP tests, then verify process recovery."""
import argparse, http.server, json, os, pathlib, signal, socket, secrets, subprocess, tempfile, threading, time, urllib.request

def port():
 with socket.socket() as sock:
  sock.bind(('127.0.0.1',0));return sock.getsockname()[1]
class Echo(http.server.BaseHTTPRequestHandler):
 def do_GET(self):
  size=int(self.headers.get('Content-Length','0'));body=self.rfile.read(size).decode()
  data=json.dumps({'path':self.path,'body':body,'host':self.headers.get('Host'),'forwarded_for':self.headers.get('X-Forwarded-For')}).encode()
  self.send_response(200);self.send_header('Content-Type','application/json');self.send_header('Content-Length',str(len(data)));self.end_headers();self.wfile.write(data)
 do_POST=do_GET
 def log_message(self,*args):pass

def main():
 parser=argparse.ArgumentParser();parser.add_argument('--binary',default=os.environ.get('PROXY_SERVER_BIN','target/debug/proxy-server'));parser.add_argument('--database-url');args=parser.parse_args()
 binary=str(pathlib.Path(args.binary).resolve());admin,proxy,upstream=port(),port(),port()
 echo=http.server.ThreadingHTTPServer(('127.0.0.1',upstream),Echo);threading.Thread(target=echo.serve_forever,daemon=True).start()
 with tempfile.TemporaryDirectory(prefix='dynamic-url-proxy-') as directory:
  config=pathlib.Path(directory)/'config.toml';db=pathlib.Path(directory)/'proxy.db'
  config.write_text(f'''admin_listen="127.0.0.1:{admin}"
proxy_listen="127.0.0.1:{proxy}"
database_url="{args.database_url or "sqlite://"+str(db)}"
[auth]
api_token_enabled=true
[security]
upstream_cidr_allowlist=["127.0.0.1/32"]
[cache.redis]
sync_interval_seconds=1
''')
  env=os.environ.copy();env.pop('DATABASE_URL',None);env.pop('CACHE_MODE',None);env.update(ADMIN_USERNAME='e2e-admin',ADMIN_PASSWORD=secrets.token_urlsafe(24),PROXY_CONFIG=str(config),ADMIN_API_TOKEN='isolated-e2e-token-24-bytes-minimum',PROXY_E2E_ADMIN=f'http://127.0.0.1:{admin}',PROXY_E2E_PROXY=f'http://127.0.0.1:{proxy}',PROXY_E2E_UPSTREAM_PORT=str(upstream))
  def read(path):
   request=urllib.request.Request(f'http://127.0.0.1:{admin}{path}',headers={'Authorization':'Bearer '+env['ADMIN_API_TOKEN']})
   with urllib.request.urlopen(request,timeout=2) as response:return json.load(response)
  log=open(pathlib.Path(directory)/'server.log','w+')
  def start():
   process=subprocess.Popen([binary],env=env,stdout=log,stderr=subprocess.STDOUT)
   for _ in range(200):
    if process.poll() is not None:log.seek(0);raise RuntimeError(log.read())
    try:read('/health/ready');return process
    except (OSError,ValueError):time.sleep(.1)
   process.kill();raise RuntimeError('server readiness timed out')
  def stop(process):
   process.send_signal(signal.SIGTERM)
   try:process.wait(timeout=15)
   except subprocess.TimeoutExpired:process.kill();process.wait();raise RuntimeError('server failed graceful shutdown deadline')
  def check_session():
   import http.cookiejar
   jar=http.cookiejar.CookieJar();client=urllib.request.build_opener(urllib.request.HTTPCookieProcessor(jar))
   base=f'http://127.0.0.1:{admin}'
   request=urllib.request.Request(base+'/api/v1/auth/login',data=json.dumps({'username':env['ADMIN_USERNAME'],'password':env['ADMIN_PASSWORD']}).encode(),headers={'Content-Type':'application/json','Origin':base})
   with client.open(request) as response:session=json.load(response)
   assert session['user']['username']==env['ADMIN_USERNAME'] and len(jar)==1
   with client.open(base+'/api/v1/auth/session') as response:assert json.load(response)['user']==session['user']
   request=urllib.request.Request(base+'/api/v1/auth/logout',data=b'{}',headers={'Content-Type':'application/json','X-CSRF-Token':session['csrf_token'],'Origin':base})
   with client.open(request) as response:assert response.status==204
   assert len(jar)==0
  process=start()
  try:
   subprocess.run(['cargo','test','-p','proxy-e2e','--','--ignored','--test-threads=1'],env=env,check=True)
   before=read('/api/v1/config/active')['revision'];check_session();stop(process);process=start();check_session();after=read('/api/v1/config/active')['revision'];assert before==after
   stop(process);env['ADMIN_PASSWORD']=secrets.token_urlsafe(24)
   subprocess.run([binary,'--reset-admin-password'],env=env,stdout=log,stderr=subprocess.STDOUT,check=True)
   process=start();check_session()
   assert read('/api/v1/config/active')['revision']==after
   print(json.dumps({'process_recovery':'passed','password_reset_cli':'passed','committed_revision':after,'real_pingora_http_tests':'passed'}))
  except Exception:
   log.flush();log.seek(0);print(log.read());raise
  finally:
   if process.poll() is None:stop(process)
   log.close();echo.shutdown()
if __name__=='__main__':main()
