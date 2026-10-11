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
import xml.etree.ElementTree as ET

ROOT = Path(os.environ.get('PAGENOR_PACKAGE_TEST_ROOT', Path(__file__).resolve().parents[2]))
VERSION = tomllib.loads((ROOT / 'Cargo.toml').read_text())['workspace']['package']['version']
APP_ID = 'org.pagenor.app'


class ReleasePackages(unittest.TestCase):
    def test_windows_extensions_share_a_single_open_verb_and_document_icon(self):
        ns = {'w': 'http://wixtoolset.org/schemas/v4/wxs'}
        wix = ET.parse(ROOT / 'packaging/windows/pagenor.wxs')
        progid = wix.find('.//w:ProgId', ns)
        self.assertEqual(progid.attrib['Id'], 'Pagenor.Document')
        self.assertEqual(progid.attrib['Icon'], 'PagenorDocumentIcon')
        extensions = progid.findall('w:Extension', ns)
        self.assertEqual([e.attrib['Id'] for e in extensions], ['pgnr'])
        self.assertEqual(len(progid.findall('.//w:Verb', ns)), 1)
        aliases = wix.findall('.//w:RegistryValue', ns)
        legacy = [r.attrib for r in aliases if r.attrib['Key'] == r'Software\Classes\.pagenor']
        self.assertTrue(any(r.get('Name') is None and r['Value'] == 'Pagenor.Document' for r in legacy))
        self.assertTrue(any(r.get('Name') == 'Content Type' and r['Value'] == extensions[0].attrib['ContentType'] for r in legacy))

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
        for name in ['CROSS_ARCH', 'CROSS_TARGET', 'CROSS_COMPILE', 'EMULATOR']:
            self.env.pop(name, None)

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
            f'share/icons/hicolor/256x256/mimetypes/{APP_ID}.document.png': b'synthetic document icon',
            f'share/icons/hicolor/scalable/mimetypes/{APP_ID}.document.svg': b'<svg/>',
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

    def cross_env(self):
        env = dict(self.env, CROSS_ARCH='riscv64', CROSS_TARGET='riscv64gc-unknown-linux-gnu',
                   CROSS_COMPILE='riscv64-linux-gnu-')
        binary = Path(env['CARGO_TARGET_DIR']) / env['CROSS_TARGET'] / 'release'
        binary.mkdir(parents=True)
        for name in ['pagenor', 'pagenor-cli']:
            source = binary / name
            source.write_text('#!/bin/sh\nexit 99 # foreign binary must never execute directly\n')
            source.chmod(0o755)
        self.tool('riscv64-linux-gnu-strip', '#!/bin/sh\necho "$*" > "$CARGO_TARGET_DIR/strip-args"\n')
        return env

    def test_linux_cross_tar_uses_target_binaries_and_never_runs_them_on_the_host(self):
        env = self.cross_env()
        result = self.run_script('packaging/linux/package.sh', '--skip-build', '--formats', 'tar', env=env)
        self.assertIn('CLI execution skipped', result.stdout)
        out = Path(env['DIST']) / f'pagenor-{VERSION}-linux-riscv64.tar.gz'
        with tarfile.open(out) as tar:
            base = f'pagenor-{VERSION}-linux-riscv64'
            self.assertIn(b'foreign binary', tar.extractfile(base + '/bin/pagenor-cli').read())
            self.assertIn(base + '/share/doc/pagenor/LICENSE-Pagenor-brand.txt', tar.getnames())
            self.assertIn(base + f'/share/icons/hicolor/scalable/mimetypes/{APP_ID}.document.svg', tar.getnames())
        self.assertIn('/usr/bin/pagenor-cli', (Path(env['CARGO_TARGET_DIR']) / 'strip-args').read_text())

    def test_linux_cross_build_routes_cargo_and_emulator_and_propagates_smoke_failure(self):
        env = self.cross_env()
        self.tool('cargo', '#!/bin/sh\necho "$*" > "$CARGO_TARGET_DIR/cargo-args"\n')
        self.tool('qemu-riscv64', '#!/bin/sh\necho "$*" > "$CARGO_TARGET_DIR/qemu-args"\nexit "${PAGENOR_TEST_QEMU_EXIT:-0}"\n')
        env['EMULATOR'] = 'qemu-riscv64'
        self.run_script('packaging/linux/package.sh', '--formats', 'tar', env=env)
        target = Path(env['CARGO_TARGET_DIR'])
        self.assertIn('--target riscv64gc-unknown-linux-gnu', (target / 'cargo-args').read_text())
        self.assertIn('/usr/bin/pagenor-cli --version', (target / 'qemu-args').read_text())
        env['PAGENOR_TEST_QEMU_EXIT'] = '7'
        result = self.run_script('packaging/linux/package.sh', '--skip-build', '--formats', 'tar', env=env, success=False)
        self.assertEqual(result.returncode, 7)

    def test_linux_rejects_mislabeled_targets_and_unsupported_formats_before_staging(self):
        env = self.cross_env()
        for args, changes in [
            (['--formats', 'tar'], {'CROSS_TARGET': ''}),
            (['--formats', 'tar'], {'CROSS_TARGET': '../release'}),
            (['--formats', 'tar'], {'CROSS_ARCH': 'aarch64'}),
            (['--formats', 'tar appimage'], {}),
            (['--formats', 'deb'], {}),
            (['--formats', 'unknown'], {}),
            (['--formats', ''], {}),
            (['--formats'], {}),
        ]:
            self.run_script('packaging/linux/package.sh', '--skip-build', *args, env=dict(env, **changes), success=False)
            self.assertFalse((Path(env['CARGO_TARGET_DIR']) / 'linux-package').exists())

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
                         f'share/applications/{APP_ID}.desktop', f'share/icons/hicolor/256x256/apps/{APP_ID}.png',
                         f'share/icons/hicolor/256x256/mimetypes/{APP_ID}.document.png',
                         f'share/icons/hicolor/scalable/mimetypes/{APP_ID}.document.svg']:
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
        for name in ['version', 'linux', 'linux-riscv64', 'flatpak', 'freebsd', 'web']:
            self.assertNotIn('environment', jobs[name], name)
        for name in ['macos', 'windows', 'release']:
            self.assertEqual(jobs[name]['environment'], 'release', name)
        self.assertTrue({'flatpak', 'freebsd', 'linux-riscv64'} <= set(jobs['release']['needs']))
        riscv = jobs['linux-riscv64']
        self.assertEqual(riscv['runs-on'], 'ubuntu-24.04')
        build = next(step for step in riscv['steps'] if step.get('name') == 'Build and package (tar.gz)')
        self.assertEqual(build['run'], 'packaging/linux/package.sh --formats tar')
        self.assertEqual(build['env']['CROSS_TARGET'], 'riscv64gc-unknown-linux-gnu')
        self.assertEqual(build['env']['EMULATOR'], 'qemu-riscv64')
        self.assertEqual(build['env']['CC_riscv64gc_unknown_linux_gnu'], 'riscv64-linux-gnu-gcc')
        self.assertEqual(build['env']['AR_riscv64gc_unknown_linux_gnu'], 'riscv64-linux-gnu-ar')
        self.assertEqual(build['env']['PKG_CONFIG_LIBDIR'], '/usr/lib/riscv64-linux-gnu/pkgconfig:/usr/share/pkgconfig')
        vm = next(step for step in jobs['freebsd']['steps'] if step.get('uses', '').startswith('vmactions/freebsd-vm@'))['with']
        self.assertEqual((vm['cpu'], vm['mem']), (2, 4096))
        self.assertIn(r'org\.pagenor\.app\.desktop', vm['run'])
        self.assertNotIn('storyteller', vm['run'])

    def test_macos_installer_keeps_owner_icons_and_stages_finder_layout(self):
        # Exercise the real staging script on any host; only platform tools are shims.
        self.tool('lipo', """#!/usr/bin/env python3
import shutil, sys
if sys.argv[1]=='-create': shutil.copy2(sys.argv[-1], sys.argv[sys.argv.index('-output')+1])
""")
        for name in ['codesign', 'plutil']:
            self.tool(name, '#!/bin/sh\nexit 0\n')
        self.tool('ditto', """#!/usr/bin/env python3
from pathlib import Path
import shutil, sys
if '-c' in sys.argv: Path(sys.argv[-1]).write_bytes(b'synthetic CLI ZIP')
else: shutil.copytree(sys.argv[-2],sys.argv[-1])
""")
        self.tool('hdiutil', """#!/usr/bin/env python3
from pathlib import Path
import json, os, shutil, sys
args=sys.argv[1:]
if args[0]=='create':
    Path(os.environ['PAGENOR_TEST_DMG_ARGS']).write_text(json.dumps(args))
    Path(args[-1]).write_bytes(b'synthetic DMG fixture')
elif args[0]=='attach':
    stage=Path(os.environ['CARGO_TARGET_DIR'])/'macos-package/dmg/Pagenor.app'
    mount=Path(args[args.index('-mountpoint')+1])/'Pagenor.app'
    shutil.copytree(stage,mount)
elif args[0]!='detach':
    raise SystemExit('unsupported synthetic hdiutil operation')
""")
        inputs = Path(self.env['CARGO_TARGET_DIR']) / 'aarch64-apple-darwin/release'
        inputs.mkdir(parents=True)
        for name in ['pagenor', 'pagenor-cli']:
            binary=inputs/name
            binary.write_text(f'#!/bin/sh\necho "{name} {VERSION} (test stub)"\n')
            binary.chmod(0o755)
        args_path=self.path/'dmg-args.json'
        env=dict(self.env, PAGENOR_TEST_DMG_ARGS=str(args_path), MACOS_SIGN_IDENTITY='-')
        self.run_script('packaging/macos/package.sh','--skip-build','--arch','aarch64',env=env)
        args=json.loads(args_path.read_text())
        self.assertEqual(args[args.index('-volname')+1], 'Pagenor')
        self.assertEqual(args[args.index('-fs')+1], 'HFS+')
        stage=Path(self.env['CARGO_TARGET_DIR'])/'macos-package/dmg'
        self.assertEqual(os.readlink(stage/'Applications'), '/Applications')
        for src,dest in [('background.tiff','.background/background.tiff'),
                         ('dmg-layout.DS_Store','.DS_Store'), ('LICENSE.txt','.background/LICENSE.txt')]:
            self.assertEqual((stage/dest).read_bytes(), (ROOT/'packaging/macos/dmg'/src).read_bytes())
        app=stage/'Pagenor.app/Contents'
        for source,dest in [('pagenor.icns','Pagenor.icns'),('pagenor-document.icns','PagenorDocument.icns')]:
            self.assertEqual((app/'Resources'/dest).read_bytes(),(ROOT/'assets/app-icon'/source).read_bytes())
        import plistlib
        info=plistlib.loads((app/'Info.plist').read_bytes())
        self.assertEqual(info['CFBundleShortVersionString'],VERSION)
        self.assertEqual(info['CFBundleIconFile'],'Pagenor.icns')
        self.assertTrue(all(t['CFBundleTypeIconFile']=='PagenorDocument.icns' for t in info['CFBundleDocumentTypes']))
        self.assertTrue((stage/'.background/LICENSE-Pagenor-brand.txt').is_file())
        self.assertTrue((stage/'.background/OFL-Inter.txt').is_file())

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
            self.assertTrue((appdir / f'usr/share/icons/hicolor/256x256/mimetypes/{APP_ID}.document.png').is_file())
            self.assertTrue((appdir / f'usr/share/icons/hicolor/scalable/mimetypes/{APP_ID}.document.svg').is_file())
        env['PAGENOR_TEST_ZSYNC'] = '0'
        for actions in ['', 'true']:
            env['GITHUB_ACTIONS'] = actions
            result = self.run_script('packaging/linux/package.sh', '--skip-build', '--formats', 'appimage', env=env)
            diagnostics = result.stdout if actions else result.stderr
            self.assertIn('zsyncmake not found', diagnostics)
            if actions:
                self.assertIn('::warning::', diagnostics)
            self.assertFalse(Path(args[-1] + '.zsync').exists(), 'an old update file must never describe a new AppImage')


if __name__ == '__main__':
    unittest.main(verbosity=2)
