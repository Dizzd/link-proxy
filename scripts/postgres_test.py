#!/usr/bin/env python3
"""Test against an owned, disposable PostgreSQL cluster; never touch an existing DB."""
import argparse
import glob
import os
import pathlib
import shutil
import socket
import subprocess
import tempfile


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', default='target/debug/proxy-server')
    parser.add_argument('--browser-external', action='store_true', help='Also run browser_test.py using an external Windows runner')
    args = parser.parse_args()
    candidates = sorted(glob.glob('/usr/lib/postgresql/*/bin/initdb'))
    initdb = shutil.which('initdb') or (candidates[-1] if candidates else None)
    if not initdb:
        raise SystemExit('Install PostgreSQL server tools (initdb and pg_ctl) first')
    pg_ctl = str(pathlib.Path(initdb).with_name('pg_ctl'))
    with tempfile.TemporaryDirectory(prefix='dynamic-proxy-pg-') as directory:
        data = str(pathlib.Path(directory) / 'data')
        with socket.socket() as sock:
            sock.bind(('127.0.0.1', 0))
            port = sock.getsockname()[1]
        subprocess.run([initdb, '-D', data, '-A', 'trust', '-U', 'proxy_test', '--no-locale'], check=True, stdout=subprocess.DEVNULL)
        subprocess.run([pg_ctl, '-D', data, '-l', directory + '/postgres.log', '-o', f'-h 127.0.0.1 -p {port} -k {directory}', '-w', 'start'], check=True)
        env = os.environ.copy()
        url = f'postgres://proxy_test@127.0.0.1:{port}/postgres'
        env['TEST_POSTGRES_URL'] = url
        try:
            subprocess.run(['cargo', 'test', '-p', 'proxy-application', '--test', 'postgres', '--', '--ignored', '--test-threads=1'], env=env, check=True)
            # A separate DB isolates HTTP fixtures from application tests.
            subprocess.run([str(pathlib.Path(initdb).with_name('createdb')), '-h', '127.0.0.1', '-p', str(port), '-U', 'proxy_test', 'http_tests'], check=True)
            subprocess.run(['python3', 'scripts/e2e.py', '--binary', args.binary, '--database-url', url.replace('/postgres', '/http_tests')], env=env, check=True)
            if args.browser_external:
                subprocess.run([str(pathlib.Path(initdb).with_name('createdb')), '-h', '127.0.0.1', '-p', str(port), '-U', 'proxy_test', 'browser_tests'], check=True)
                subprocess.run(['python3', 'scripts/browser_test.py', '--binary', args.binary, '--database-url', url.replace('/postgres', '/browser_tests'), '--external'], env=env, check=True)
        finally:
            subprocess.run([pg_ctl, '-D', data, '-m', 'fast', '-w', 'stop'], check=True)


if __name__ == '__main__':
    main()
