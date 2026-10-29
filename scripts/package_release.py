#!/usr/bin/env python3
"""Create source-free, digest-pinned GitHub Release deployment assets."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import tarfile

ROOT = Path(__file__).resolve().parents[1]
ARCHIVE = 'dynamic-url-proxy-deploy.tar.gz'


def package(repository: str, version: str, image: str, output: Path) -> None:
    if not re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+', repository):
        raise ValueError('repository must be OWNER/REPO')
    if not re.fullmatch(r'[A-Za-z0-9_][A-Za-z0-9_.-]{0,127}', version):
        raise ValueError('version must be a valid Docker image tag')
    expected = rf'ghcr\.io/{re.escape(repository.lower())}@sha256:[a-f0-9]{{64}}'
    if not re.fullmatch(expected, image):
        raise ValueError('image must pin this repository to a SHA-256 digest')
    if output.exists() and any(output.iterdir()):
        raise ValueError('output directory must be empty; existing files are never removed')
    output.mkdir(parents=True, exist_ok=True)
    files = {
        'docker-compose.yml': ROOT / 'deploy/docker-compose.yml',
        'Caddyfile': ROOT / 'config/caddy/Caddyfile',
        'config.toml': ROOT / 'config/config.docker.toml',
        'config.redis.toml': ROOT / 'config/config.redis.toml',
        'README.md': ROOT / 'deploy/README.md',
    }
    for target, source in files.items():
        content = source.read_text(encoding='utf-8')
        (output / target).write_text(content, encoding='utf-8', newline='\n')
    environment = (ROOT / 'deploy/.env.example').read_text(encoding='utf-8')
    (output / '.env.example').write_text(environment.replace('__RELEASE_IMAGE__', image), encoding='utf-8', newline='\n')
    metadata = {'repository': repository, 'version': version, 'image': image, 'platform': 'linux/amd64'}
    (output / 'release.json').write_text(json.dumps(metadata, indent=2) + '\n', encoding='utf-8')
    members = sorted(output.iterdir())
    # Deliberately omit .env, source, staging data and any caller-supplied files.
    with tarfile.open(output / ARCHIVE, 'w:gz') as archive:
        for member in members:
            info = archive.gettarinfo(str(member), arcname=member.name)
            info.uid = info.gid = 0
            info.uname = info.gname = ''
            info.mode = 0o644
            with member.open('rb') as data:
                archive.addfile(info, data)
    checksums = []
    for member in sorted(output.iterdir()):
        checksums.append(f'{hashlib.sha256(member.read_bytes()).hexdigest()}  {member.name}\n')
    (output / 'SHA256SUMS').write_text(''.join(checksums), encoding='utf-8', newline='\n')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repository', required=True)
    parser.add_argument('--version', required=True)
    parser.add_argument('--image', required=True)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    package(args.repository, args.version, args.image, args.output)


if __name__ == '__main__':
    main()
