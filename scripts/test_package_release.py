"""Security and integrity checks for the source-free release bundle."""
import hashlib
import json
from pathlib import Path
import tarfile
import tempfile
import unittest

from package_release import ARCHIVE, package

IMAGE = 'ghcr.io/example/link-proxy@sha256:' + 'a' * 64


class ReleasePackageTests(unittest.TestCase):
    def test_bundle_is_pinned_complete_and_contains_no_sources_or_real_environment(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            package('Example/Link-Proxy', 'v1.0.0', IMAGE, output)
            env = (output / '.env.example').read_text()
            self.assertIn(f'PROXY_IMAGE={IMAGE}', env)
            self.assertIn('ADMIN_PASSWORD=\n', env)
            self.assertIn('COMPOSE_PROFILES=postgres,caddy', env)
            self.assertNotIn('__RELEASE_IMAGE__', env)
            self.assertEqual(json.loads((output / 'release.json').read_text())['version'], 'v1.0.0')
            self.assertNotIn('build:', (output / 'docker-compose.yml').read_text())
            with tarfile.open(output / ARCHIVE) as archive:
                names = {member.name for member in archive.getmembers()}
                self.assertEqual(names, {'.env.example', 'docker-compose.yml', 'Caddyfile',
                                         'config.toml', 'config.redis.toml', 'README.md', 'release.json'})
                self.assertTrue(all(member.isfile() and member.mode == 0o644 for member in archive.getmembers()))
            for line in (output / 'SHA256SUMS').read_text().splitlines():
                digest, name = line.split('  ', 1)
                self.assertEqual(hashlib.sha256((output / name).read_bytes()).hexdigest(), digest)

    def test_existing_output_cannot_be_overwritten_or_packaged_as_secrets(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            (output / '.env').write_text('secret-never-package')
            with self.assertRaisesRegex(ValueError, 'must be empty'):
                package('Example/Link-Proxy', 'v1.0.0', IMAGE, output)
            self.assertEqual((output / '.env').read_text(), 'secret-never-package')
            self.assertEqual(len(list(output.iterdir())), 1)

    def test_unpinned_or_foreign_images_and_unsafe_metadata_are_rejected(self):
        for repository, version, image in [
            ('Example/Link-Proxy', 'v1.0.0', 'ghcr.io/example/link-proxy:latest'),
            ('Example/Link-Proxy', 'v1.0.0', IMAGE.replace('example/', 'attacker/')),
            ('Example/Link-Proxy', 'v1/../../escape', IMAGE),
            ('Example/Link-Proxy', 'v1\nINJECT=1', IMAGE),
            ('../../escape', 'v1.0.0', IMAGE),
        ]:
            with self.subTest(repository=repository, version=version, image=image):
                with tempfile.TemporaryDirectory() as directory:
                    output = Path(directory) / 'output'
                    with self.assertRaises(ValueError):
                        package(repository, version, image, output)
                    self.assertFalse(output.exists())


if __name__ == '__main__':
    unittest.main()
