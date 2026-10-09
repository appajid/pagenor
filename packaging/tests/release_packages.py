#!/usr/bin/env python3
"""Local package contracts with synthetic binaries; no host installation or Rust build."""
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile
import tomllib
import unittest

ROOT = Path(os.environ.get('PAGENOR_PACKAGE_TEST_ROOT', Path(__file__).resolve().parents[2]))
VERSION = tomllib.loads((ROOT / 'Cargo.toml').read_text())['workspace']['package']['version']
APP_ID = 'org.pagenor.app'


class ReleasePackages(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='pagenor-packaging-')
        self.addCleanup(self.temp.cleanup)
        self.path = Path(self.temp.name)
        self.shims = self.path / 'tools'
        self.shims.mkdir()
        self.tool('uname', '#!/bin/sh\ncase "$1" in -s) echo "${PAGENOR_TEST_OS:-Linux}";; *) echo "${PAGENOR_TEST_ARCH:-x86_64}";; esac\n')
        self.tool('strip', '#!/bin/sh\nexit 0\n')
        self.tool('install', '''#!/usr/bin/env python3
import os, shutil, sys
from pathlib import Path
args=sys.argv[1:]; mode=0o644
while args and args[0].startswith('-'):
    a=args.pop(0)
    if a=='-m': mode=int(args.pop(0),8)
    elif a.startswith('-Dm'): mode=int(a[3:],8)
    else: raise SystemExit('unsupported test install option')
dest=Path(args[-1])
for source in args[:-1]:
    target=dest/Path(source).name if dest.is_dir() or len(args)>2 else dest
    target.parent.mkdir(parents=True,exist_ok=True)
    shutil.copyfile(source,target); os.chmod(target,mode)
''')
        self.env = dict(os.environ, PATH=str(self.shims) + os.pathsep + os.environ['PATH'],
                        CARGO_TARGET_DIR=str(self.path / 'target'), DIST=str(self.path / 'dist'),
                        PAGENOR_VERSION=VERSION, PAGENOR_BUILD_DATE='2026-10-10')
        self.env.pop('GITHUB_REPOSITORY', None)
        self.env.pop('CRAFT_FONTS_DIR', None)

    def tool(self, name, contents):
        p = self.shims / name
        p.write_text(contents)
        p.chmod(0o755)
        return p

    def run_script(self, script, *args, success=True, env=None):
        result = subprocess.run(['bash', str(ROOT / script), *map(str, args)],
                                cwd=self.path, env=env or self.env, capture_output=True, text=True)
        if success:
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        else:
            self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        return result

    def release_tar(self, arch='x86_64', machine=None, extras=()):
        base = f'pagenor-{VERSION}-linux-{arch}'
        machine = machine or {'x86_64': b'\x3e\x00', 'aarch64': b'\xb7\x00'}[arch]
        header = b'\x7fELF\x02\x01' + b'\x00' * 12 + machine + b'synthetic fixture'
        resources = {
            'bin/pagenor': header, 'bin/pagenor-cli': header,
            f'share/applications/{APP_ID}.desktop': (ROOT / f'packaging/linux/{APP_ID}.desktop').read_bytes(),
            f'share/mime/packages/{APP_ID}.xml': (ROOT / f'packaging/linux/{APP_ID}.mime.xml').read_bytes(),
            f'share/metainfo/{APP_ID}.metainfo.xml': (ROOT / f'packaging/linux/{APP_ID}.metainfo.xml.in').read_bytes(),
            f'share/icons/hicolor/256x256/apps/{APP_ID}.png': b'synthetic icon',
            f'share/icons/hicolor/scalable/apps/{APP_ID}.svg': b'<svg/>',
            'share/doc/pagenor/LICENSE-MIT': (ROOT / 'LICENSE-MIT').read_bytes(),
            'share/doc/pagenor/LICENSE-APACHE': (ROOT / 'LICENSE-APACHE').read_bytes(),
        }
        out = self.path / 'release.tar.gz'
        with tarfile.open(out, 'w:gz') as tar:
            for name, payload in resources.items():
                info = tarfile.TarInfo(base + '/' + name)
                info.mode = 0o755 if name.startswith('bin/') else 0o644
                info.size = len(payload)
                tar.addfile(info, io.BytesIO(payload))
            for info, payload in extras:
                tar.addfile(info, io.BytesIO(payload) if payload is not None else None)
        return out

    def test_freebsd_dry_run_contains_owner_icons_licences_and_version(self):
        env = dict(self.env, PAGENOR_TEST_ARCH='amd64', PAGENOR_TEST_OS='FreeBSD')
        self.run_script('packaging/freebsd/package.sh', '--dry-run', env=env)
        out = Path(env['CARGO_TARGET_DIR']) / 'freebsd-package' / f'pagenor-{VERSION}-freebsd-x86_64.tar.gz'
        with tarfile.open(out) as tar:
            names = set(tar.getnames())
            base = f'pagenor-{VERSION}-freebsd-x86_64/'
            for name in ['bin/pagenor', 'bin/pagenor-cli', 'share/doc/pagenor/LICENSE-MIT',
                         'share/doc/pagenor/LICENSE-APACHE', 'share/doc/pagenor/NOTICE',
                         'share/doc/pagenor/LICENSE-Pagenor-brand.txt', 'share/doc/pagenor/LICENSE-Pagenor-app-icon.txt',
                         'share/doc/pagenor/OFL-Inter.txt', 'share/doc/pagenor/OFL-SourceSerif4.txt',
                         f'share/applications/{APP_ID}.desktop', f'share/icons/hicolor/256x256/apps/{APP_ID}.png']:
                self.assertIn(base + name, names)
            meta = tar.extractfile(base + f'share/metainfo/{APP_ID}.metainfo.xml').read().decode()
            self.assertIn(VERSION, meta)
        self.assertFalse(list(Path(env['DIST']).glob('*.tar.gz')), 'dry-run stubs must not enter release artifacts')

    def test_flatpak_stage_accepts_both_architectures_without_host_installation(self):
        for arch in ['x86_64', 'aarch64']:
            env = dict(self.env, PAGENOR_TEST_ARCH=arch)
            self.run_script('packaging/linux/flatpak-bundle.sh', '--stage-only', self.release_tar(arch), env=env)
            staged = Path(env['CARGO_TARGET_DIR']) / 'flatpak-bundle/stage'
            self.assertTrue((staged / 'bin/pagenor-cli').stat().st_mode & 0o111)
            self.assertIn('*.pgnr', (staged / f'share/mime/packages/{APP_ID}.xml').read_text())

    def test_flatpak_rejects_wrong_architecture_before_extracting(self):
        self.run_script('packaging/linux/flatpak-bundle.sh', '--stage-only', self.release_tar(machine=b'\xb7\x00'), success=False)
        self.assertFalse((Path(self.env['CARGO_TARGET_DIR']) / 'flatpak-bundle/stage/bin/pagenor').exists())

    def test_flatpak_rejects_wrong_release_root(self):
        self.run_script('packaging/linux/flatpak-bundle.sh', '--stage-only', self.release_tar('aarch64'), success=False)
        self.assertFalse((Path(self.env['CARGO_TARGET_DIR']) / 'flatpak-bundle/stage/bin/pagenor').exists())

    def test_flatpak_rejects_traversal_and_links_before_writing_payloads(self):
        base = f'pagenor-{VERSION}-linux-x86_64'
        traversal = tarfile.TarInfo(base + '/../outside')
        traversal.size = 4
        link = tarfile.TarInfo(base + '/share/redirect')
        link.type = tarfile.SYMTYPE
        link.linkname = '/tmp'
        for info, payload in [(traversal, b'bad!'), (link, None)]:
            self.run_script('packaging/linux/flatpak-bundle.sh', '--stage-only', self.release_tar(extras=[(info, payload)]), success=False)
            self.assertFalse((Path(self.env['CARGO_TARGET_DIR']) / 'flatpak-bundle/stage/bin/pagenor').exists())
            self.assertFalse((self.path / 'outside').exists())

    def test_flatpak_manifest_permissions_and_cli_match_the_host(self):
        import yaml
        manifests = [yaml.safe_load((ROOT / f'packaging/linux/flatpak/{APP_ID}{suffix}.yml').read_text()) for suffix in ['', '.bundle']]
        for key in ['id', 'command', 'runtime', 'runtime-version', 'sdk', 'finish-args']:
            self.assertEqual(manifests[0][key], manifests[1][key], key)
        self.assertIn('--socket=x11', manifests[0]['finish-args'])
        self.assertNotIn('--socket=fallback-x11', manifests[0]['finish-args'])
        self.assertIn('--filesystem=xdg-documents', manifests[0]['finish-args'])
        self.assertNotIn('--share=network', manifests[0]['finish-args'])
        for manifest in manifests:
            self.assertTrue(any('/app/bin/pagenor-cli' in command for command in manifest['modules'][0]['build-commands']))

    def test_release_workflow_routes_architectures_and_gates_publication(self):
        import yaml
        jobs = yaml.safe_load((ROOT / '.github/workflows/release.yml').read_text())['jobs']
        self.assertEqual(set(jobs['flatpak']['needs']), {'version', 'linux'})
        self.assertEqual(jobs['flatpak']['strategy']['matrix']['include'], [
            {'arch': 'x86_64', 'runner': 'ubuntu-24.04'},
            {'arch': 'aarch64', 'runner': 'ubuntu-24.04-arm'},
        ])
        download = next(step for step in jobs['flatpak']['steps'] if step.get('uses', '').startswith('actions/download-artifact@'))
        self.assertEqual(download['with']['name'], 'release-linux-${{ matrix.arch }}')
        for name in ['version', 'linux', 'flatpak', 'freebsd', 'web']:
            self.assertNotIn('environment', jobs[name], name)
        for name in ['macos', 'windows', 'release']:
            self.assertEqual(jobs[name]['environment'], 'release', name)
        self.assertTrue({'flatpak', 'freebsd'} <= set(jobs['release']['needs']))
        vm = next(step for step in jobs['freebsd']['steps'] if step.get('uses', '').startswith('vmactions/freebsd-vm@'))['with']
        self.assertEqual((vm['cpu'], vm['mem']), (2, 4096))
        self.assertIn(r'org\.pagenor\.app\.desktop', vm['run'])
        self.assertNotIn('storyteller', vm['run'])

    def test_appimage_update_metadata_uses_owner_repo_and_removes_stale_zsync(self):
        bin_dir = Path(self.env['CARGO_TARGET_DIR']) / 'release'
        bin_dir.mkdir(parents=True)
        for name in ['pagenor', 'pagenor-cli']:
            p = bin_dir / name
            p.write_text(f'#!/bin/sh\necho "{name} {VERSION} (test stub)"\n')
            p.chmod(0o755)
        tool = self.path / 'image-tool'
        tool.write_text('''#!/usr/bin/env python3
import json, os, sys
from pathlib import Path
Path(os.environ['PAGENOR_TEST_ARGS']).write_text(json.dumps(sys.argv[1:]))
Path(sys.argv[-1]).write_bytes(b'synthetic AppImage fixture')
if os.environ.get('PAGENOR_TEST_ZSYNC')=='1': Path(sys.argv[-1]+'.zsync').write_bytes(b'synthetic zsync fixture')
''')
        tool.chmod(0o755)
        args_file = self.path / 'args.json'
        env = dict(self.env, APPIMAGETOOL='./image-tool', PAGENOR_TEST_ARGS=str(args_file), PAGENOR_TEST_ZSYNC='1')
        for repo in ['appajid/pagenor', 'example/fork']:
            env['GITHUB_REPOSITORY'] = repo
            self.run_script('packaging/linux/package.sh', '--skip-build', '--formats', 'appimage', env=env)
            args = json.loads(args_file.read_text())
            self.assertEqual(args[args.index('-u') + 1], f'gh-releases-zsync|{repo.replace("/", "|")}|latest|pagenor-*-linux-x86_64.AppImage.zsync')
            self.assertTrue(Path(args[-1]).is_absolute())
            self.assertTrue(Path(args[-1] + '.zsync').exists())
            appdir = Path(args[-2])
            for notice in ['NOTICE', 'LICENSE-MIT', 'LICENSE-APACHE', 'OFL-Inter.txt',
                           'LICENSE-Pagenor-brand.txt', 'LICENSE-Pagenor-app-icon.txt']:
                self.assertTrue((appdir / 'usr/share/doc/pagenor' / notice).is_file(), notice)
        env['PAGENOR_TEST_ZSYNC'] = '0'
        result = self.run_script('packaging/linux/package.sh', '--skip-build', '--formats', 'appimage', env=env)
        self.assertIn('zsyncmake not found', result.stderr)
        self.assertFalse(Path(args[-1] + '.zsync').exists(), 'an old update file must never describe a new AppImage')


if __name__ == '__main__':
    unittest.main(verbosity=2)
