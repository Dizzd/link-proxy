#!/usr/bin/env python3
"""Run Chromium against an isolated real server, including password/session changes."""
import argparse
import json
import os
import pathlib
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


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', default='target/debug/proxy-server')
    parser.add_argument('--database-url')
    parser.add_argument('--windows', action='store_true', help='Use Windows Node/Chromium from WSL')
    parser.add_argument('--external', action='store_true', help='Wait for browser_test.ps1 to write a result alongside settings.json')
    args = parser.parse_args()
    binary = str(pathlib.Path(args.binary).resolve())
    admin, proxy = port(), port()
    with tempfile.TemporaryDirectory(prefix='browser-', dir='docs/verification') as directory:
        directory = pathlib.Path(directory).resolve()
        config = directory / 'config.toml'
        url = args.database_url or 'sqlite://' + str(directory / 'browser.db')
        config.write_text(f'admin_listen="127.0.0.1:{admin}"\nproxy_listen="127.0.0.1:{proxy}"\ndatabase_url={json.dumps(url)}\n')
        env = os.environ.copy()
        env.pop('DATABASE_URL', None)
        env.update(ADMIN_USERNAME='browser-admin', ADMIN_PASSWORD=secrets.token_urlsafe(24), PROXY_CONFIG=str(config))
        settings = {key: env[key] for key in ['ADMIN_USERNAME', 'ADMIN_PASSWORD']}
        settings.update(PLAYWRIGHT_BASE_URL=f'http://127.0.0.1:{admin}', PROXY_BROWSER_URL=f'http://127.0.0.1:{proxy}', EXPECT_DATABASE_BACKEND='postgresql' if args.database_url else 'sqlite')
        settings_path = directory / 'settings.json'
        settings_path.write_text(json.dumps(settings))
        settings_path.chmod(0o600)
        with open(directory / 'server.log', 'w+') as log:
            process = subprocess.Popen([binary], env=env, stdout=log, stderr=subprocess.STDOUT)
            try:
                for _ in range(200):
                    if process.poll() is not None:
                        raise RuntimeError('Server exited during startup')
                    try:
                        with urllib.request.urlopen(settings['PLAYWRIGHT_BASE_URL'] + '/health/ready', timeout=1):
                            break
                    except OSError:
                        time.sleep(.1)
                else:
                    raise RuntimeError('Server readiness timed out')
                if args.external:
                    print(f'Browser ready: {settings_path}', flush=True)
                    result = settings_path.with_suffix('.result')
                    for _ in range(180):
                        if result.exists():
                            if result.read_text().strip() != '0':
                                raise RuntimeError('External browser test failed')
                            break
                        time.sleep(1)
                    else:
                        raise RuntimeError('External browser runner timed out')
                elif args.windows:
                    script = subprocess.check_output(['wslpath', '-w', str(pathlib.Path('scripts/browser_test.ps1').resolve())], text=True).strip()
                    windows_settings = subprocess.check_output(['wslpath', '-w', str(settings_path)], text=True).strip()
                    subprocess.run(['powershell.exe', '-NoProfile', '-File', script, '-SettingsPath', windows_settings], check=True)
                else:
                    subprocess.run(['node', 'node_modules/@playwright/test/cli.js', 'test'], env={**env, **settings}, check=True)
            except Exception:
                log.flush()
                log.seek(0)
                print(log.read())
                raise
            finally:
                if process.poll() is None:
                    process.send_signal(signal.SIGTERM)
                    try:
                        process.wait(timeout=15)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait()


if __name__ == '__main__':
    main()
