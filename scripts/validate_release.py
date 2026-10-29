#!/usr/bin/env python3
"""Validate release Compose profiles with the real Compose parser, without a daemon."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile


def validate(directory: Path, command: list[str], source_compose: Path | None = None) -> None:
    directory = directory.resolve()
    example = (directory / '.env.example').read_text()
    dotenv = example.replace('ADMIN_PASSWORD=\n', 'ADMIN_PASSWORD=validation-only-password\n')
    dotenv = dotenv.replace('POSTGRES_PASSWORD=\n', 'POSTGRES_PASSWORD=' + 'a' * 64 + '\n')
    keys = {line.split('=', 1)[0] for line in example.splitlines() if line and not line.startswith('#') and '=' in line}
    image = json.loads((directory / 'release.json').read_text())['image']
    variants = [('', {'proxy'}), ('caddy', {'proxy', 'caddy'}),
                ('postgres,caddy', {'proxy', 'postgres', 'caddy'}),
                ('postgres,caddy,redis', {'proxy', 'postgres', 'caddy', 'redis'})]
    with tempfile.NamedTemporaryFile(mode='w', suffix='.env') as fixture:
        fixture.write(dotenv)
        fixture.flush()
        for profiles, expected in variants:
            env = {key: value for key, value in os.environ.items() if not key.startswith('COMPOSE_') and key not in keys}
            env.update(COMPOSE_PROFILES=profiles, COMPOSE_PROJECT_NAME='release-validation')
            if 'postgres' not in expected:
                env['DATABASE_URL'] = 'sqlite:///data/proxy.db'
            if 'redis' in expected:
                env['PROXY_CONFIG_FILE'] = 'config.redis.toml'
            compose_file = source_compose.resolve() if source_compose else directory / 'docker-compose.yml'
            compose = command + ['--env-file', fixture.name, '-f', str(compose_file)]
            config = json.loads(subprocess.check_output(compose + ['config', '--format', 'json'], env=env, text=True))
            services = config['services']
            assert set(services) == expected, (profiles, set(services))
            if source_compose:
                assert services['proxy']['build']['dockerfile'] == 'docker/Dockerfile'
            else:
                assert services['proxy']['image'] == image
                assert 'build' not in services['proxy']
            if 'postgres' in expected:
                assert services['proxy']['environment']['DATABASE_URL'] == 'postgres://proxy:' + 'a' * 64 + '@postgres:5432/proxy'
            for published in services['proxy']['ports']:
                assert published['host_ip'] == '127.0.0.1'
            for service in services.values():
                for volume in service.get('volumes', []):
                    if volume['type'] == 'bind':
                        assert Path(volume['source']).is_file(), volume['source']
            for private in ['postgres', 'redis']:
                if private in services:
                    assert not services[private].get('ports')
            print(f'Compose profile validation passed: {profiles or "SQLite only"}', flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('directory', type=Path)
    parser.add_argument('--compose-binary', help='Optional standalone docker-compose executable')
    parser.add_argument('--source-compose', type=Path, help='Also usable to validate the source-build Compose file')
    args = parser.parse_args()
    validate(args.directory, [args.compose_binary] if args.compose_binary else ['docker', 'compose'], args.source_compose)
