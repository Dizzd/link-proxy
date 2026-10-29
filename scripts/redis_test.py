#!/usr/bin/env python3
"""Run Redis integration tests against an owned disposable server, including shutdown."""
import os,socket,subprocess,tempfile,time
with socket.socket() as sock:
 sock.bind(('127.0.0.1',0));port=sock.getsockname()[1]
with tempfile.TemporaryDirectory(prefix='proxy-redis-test-') as directory:
 process=subprocess.Popen(['redis-server','--port',str(port),'--bind','127.0.0.1','--save','','--appendonly','no','--dir',directory,'--loglevel','warning'],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
 try:
  for _ in range(100):
   try:
    with socket.create_connection(('127.0.0.1',port),timeout=.2):break
   except OSError:time.sleep(.05)
  env=os.environ.copy();env['REDIS_TEST_URL']=f'redis://127.0.0.1:{port}/0'
  subprocess.run(['cargo','test','-p','proxy-application','--features','redis-cache','--test','cache','real_redis','--','--ignored','--nocapture','--test-threads=1'],env=env,check=True)
 finally:
  if process.poll() is None:process.terminate();process.wait(timeout=5)
