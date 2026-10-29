#!/usr/bin/env python3
"""Verify one TOML file with real startup overrides and an owned Redis instance."""
import argparse
import json
import os
from pathlib import Path
import secrets
import signal
import socket
import subprocess
import tempfile
import time
import urllib.request


def port():
    with socket.socket() as sock:
        sock.bind(('127.0.0.1', 0))
        return sock.getsockname()[1]


def stop(process):
    if process.poll() is None:
        process.send_signal(signal.SIGTERM)
        try:
            process.wait(timeout=15)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()
            raise RuntimeError('Fixture failed graceful shutdown')


def main(binary):
    binary = str(Path(binary).resolve())
    with tempfile.TemporaryDirectory(prefix='proxy-config-test-') as temporary:
        directory = Path(temporary)
        admin, proxy, redis_port = port(), port(), port()
        config = directory / 'config.toml'
        config.write_text(f'''admin_listen="127.0.0.1:{admin}"
proxy_listen="127.0.0.1:{proxy}"
database_url="sqlite://{directory / 'test.db'}"
[auth]
api_token_enabled=true
[cache]
mode="memory"
[cache.redis]
enabled=false
key_prefix="config_test:"
sync_interval_seconds=1
''')
        original = config.read_bytes()
        redis = subprocess.Popen(['redis-server', '--port', str(redis_port), '--bind', '127.0.0.1',
                                  '--save', '', '--appendonly', 'no', '--dir', str(directory)],
                                 stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        try:
            for _ in range(100):
                try:
                    with socket.create_connection(('127.0.0.1', redis_port), timeout=.2):
                        break
                except OSError:
                    if redis.poll() is not None:
                        raise RuntimeError('Redis fixture exited')
                    time.sleep(.1)
            else:
                raise RuntimeError('Redis fixture timed out')
            env = os.environ.copy()
            env.pop('DATABASE_URL', None)
            env.pop('CACHE_MODE', None)
            env.update(PROXY_CONFIG=str(config), ADMIN_USERNAME='config-test-admin',
                       ADMIN_PASSWORD=secrets.token_urlsafe(24), ADMIN_API_TOKEN=secrets.token_hex(32))
            base = f'http://127.0.0.1:{admin}'

            def request(path, data=None):
                headers = {'Authorization': 'Bearer ' + env['ADMIN_API_TOKEN'], 'Content-Type': 'application/json'}
                with urllib.request.urlopen(urllib.request.Request(base + path, headers=headers, data=data), timeout=3) as response:
                    return json.load(response)

            for mode in [None, 'memory_redis', 'memory']:
                if mode is None:
                    env.pop('CACHE_MODE', None)
                else:
                    env['CACHE_MODE'] = mode
                # Memory mode must also work when Redis cannot be reached.
                env['REDIS_URL'] = f'redis://127.0.0.1:{redis_port if mode == "memory_redis" else 1}/0'
                with (directory / 'server.log').open('w+') as log:
                    process = subprocess.Popen([binary], env=env, stdout=log, stderr=subprocess.STDOUT)
                    try:
                        for _ in range(150):
                            if process.poll() is not None:
                                raise RuntimeError('Proxy fixture exited')
                            try:
                                request('/health/ready')
                                break
                            except OSError:
                                time.sleep(.1)
                        else:
                            raise RuntimeError('Proxy readiness timed out')
                        state = request('/api/v1/system/cache/reconcile', b'{}')
                        assert state['mode'] == (mode or 'memory'), state
                        assert state['connected'], state
                        assert config.read_bytes() == original, 'The shared TOML file must not change'
                        print(f'Same config.toml startup passed: CACHE_MODE={mode or "unset"}', flush=True)
                    except Exception:
                        log.flush()
                        log.seek(0)
                        print(log.read())
                        raise
                    finally:
                        stop(process)
            for mode in ['', 'redis']:
                env['CACHE_MODE'] = mode
                result = subprocess.run([binary], env=env, capture_output=True, text=True, timeout=10)
                assert result.returncode != 0 and 'CACHE_MODE must be memory or memory_redis' in result.stderr
            print('Invalid/empty cache mode startup rejection passed', flush=True)
            if os.name == 'posix':
                env.pop('CACHE_MODE', None)
                invalid = {os.fsencode(key): os.fsencode(value) for key, value in env.items()}
                invalid[b'CACHE_MODE'] = b'\xff'
                result = subprocess.run([binary], env=invalid, capture_output=True, text=True, timeout=10)
                assert result.returncode != 0 and 'CACHE_MODE must be valid UTF-8' in result.stderr
                print('Non-UTF-8 cache mode startup rejection passed', flush=True)
        finally:
            stop(redis)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', default=os.environ.get('PROXY_SERVER_BIN', 'target/debug/proxy-server'))
    main(parser.parse_args().binary)
