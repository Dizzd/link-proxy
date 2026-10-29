#!/usr/bin/env python3
"""Exercise the exact release image with PostgreSQL, Redis and HTTP through Caddy."""
import http.cookiejar
import json
import os
from pathlib import Path
import secrets
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request
import uuid

from package_release import package


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


def main(image: str):
    with tempfile.TemporaryDirectory(prefix='proxy-image-smoke-') as temporary:
        root = Path(temporary)
        package('smoke/test', 'smoke', 'ghcr.io/smoke/test@sha256:' + '0' * 64, root)
        env = {key: value for key, value in os.environ.items() if not key.startswith('COMPOSE_')}
        env.update(PROXY_IMAGE=image, COMPOSE_PROJECT_NAME='proxy-smoke-' + uuid.uuid4().hex[:12],
                   COMPOSE_PROFILES='postgres,caddy,redis', ADMIN_USERNAME='smoke-admin',
                   ADMIN_PASSWORD=secrets.token_urlsafe(24), POSTGRES_USER='proxy', POSTGRES_DB='proxy',
                   POSTGRES_PASSWORD=secrets.token_hex(32), CACHE_MODE='memory_redis',
                   PUBLIC_HOST=':80', PROXY_BIND_IP='127.0.0.1', ADMIN_PORT='0', PROXY_PORT='0', HTTP_PORT='0', HTTPS_PORT='0')
        env['DATABASE_URL'] = f"postgres://proxy:{env['POSTGRES_PASSWORD']}@postgres:5432/proxy"
        # Do not load the example or any caller's real .env file.
        empty = root / 'smoke.env'
        empty.touch(mode=0o600)
        command = ['docker', 'compose', '--env-file', str(empty), '-f', str(root / 'docker-compose.yml')]

        def compose(*args, **kwargs):
            return subprocess.run(command + list(args), env=env, check=True, **kwargs)

        def address(service, port):
            result = compose('port', service, str(port), capture_output=True, text=True)
            return 'http://127.0.0.1:' + result.stdout.strip().rsplit(':', 1)[1]

        try:
            compose('up', '-d', '--wait', '--wait-timeout', '180')
            admin, public = address('proxy', 9000), address('caddy', 80)
            jar = http.cookiejar.CookieJar()
            client = urllib.request.build_opener(urllib.request.HTTPCookieProcessor(jar))
            csrf = ''

            def request(path, data=None):
                headers = {'Origin': admin, 'Content-Type': 'application/json', 'X-CSRF-Token': csrf}
                body = None if data is None else json.dumps(data).encode()
                with client.open(urllib.request.Request(admin + path, data=body, headers=headers), timeout=10) as response:
                    return json.load(response)

            with client.open(admin + '/', timeout=10) as response:
                assert b'id="root"' in response.read(), 'built React UI missing'
            login = request('/api/v1/auth/login', {'username': env['ADMIN_USERNAME'], 'password': env['ADMIN_PASSWORD']})
            csrf = login['csrf_token']
            assert request('/api/v1/auth/session')['user']['username'] == env['ADMIN_USERNAME']
            assert request('/api/v1/system/database/status')['backend'] == 'postgresql'
            request('/api/v1/mappings', {'mapping_key': 'image-smoke',
                'source_matcher': {'hosts': ['smoke.example'], 'path_type': 'exact', 'path_value': '/smoke'},
                'default_action': {'type': 'redirect', 'status': 302, 'target_url': 'https://destination.example/'}})
            request('/api/v1/config/publish', {'expected_revision': 0})
            for _ in range(100):
                cache = request('/api/v1/system/cache/status')
                if cache.get('connected') and cache.get('mode') == 'memory_redis':
                    break
                time.sleep(.1)
            else:
                raise AssertionError('Redis synchronization did not connect')
            transport = urllib.request.build_opener(NoRedirect())
            for attempt in range(50):
                try:
                    transport.open(urllib.request.Request(public + '/smoke', headers={'Host': 'smoke.example'}), timeout=5)
                    raise AssertionError('Caddy route did not redirect')
                except urllib.error.HTTPError as response:
                    assert response.code == 302 and response.headers['Location'] == 'https://destination.example/'
                    break
                except urllib.error.URLError:
                    if attempt == 49:
                        raise
                    time.sleep(.1)
            print('Release image smoke passed: UI, password session, PostgreSQL, Redis, publish, Caddy -> Pingora redirect')
        except Exception:
            compose('logs', '--no-color', '--tail', '100')
            raise
        finally:
            # Only this randomly named fixture project and its disposable volumes are removed.
            compose('down', '--volumes', '--remove-orphans')


if __name__ == '__main__':
    main(sys.argv[1])
