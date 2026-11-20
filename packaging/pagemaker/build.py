#!/usr/bin/env python3
"""Build an external, static PageMaker helper from checksum-pinned MPL source.

No source or binary is downloaded at application runtime. Release packaging fails
if this build fails; users must never receive a silently incomplete installer.
"""
import argparse
import hashlib
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import urllib.request

INPUTS = {
    'libpagemaker-0.0.4': ('https://dev-www.libreoffice.org/src/libpagemaker/libpagemaker-0.0.4.tar.xz',
                         '66adacd705a7d19895e08eac46d1e851332adf2e736c566bef1164e7a442519d'),
    'librevenge-0.0.6': ('https://downloads.sourceforge.net/project/libwpd/librevenge/librevenge-0.0.6/librevenge-0.0.6.tar.xz',
                       '19eacf5ce55d7fe6a990a45142589cdf7da0c7b68701797f133482cb44f189fa'),
    'zlib-1.3.1': ('https://zlib.net/fossils/zlib-1.3.1.tar.gz',
                   '9a93b2b7dfdac77ceba5a558a580e74667dd6fede4585b91eefb60f03b72df23'),
    'boost_1_86_0': ('https://archives.boost.io/release/1.86.0/source/boost_1_86_0.tar.bz2',
                     '1bed88e40401b2cb7a1f76d4bab499e352fa4d0c5f31c0dbae64e24d34d7513b'),
}


def fetch(name, cache):
    url, checksum = INPUTS[name]
    archive = cache / url.rsplit('/', 1)[1]
    if not archive.exists():
        partial = archive.with_suffix(archive.suffix + '.part')
        with urllib.request.urlopen(url, timeout=120) as response, partial.open('wb') as output:
            shutil.copyfileobj(response, output)
        partial.replace(archive)
    if hashlib.sha256(archive.read_bytes()).hexdigest() != checksum:
        raise RuntimeError(f'checksum mismatch: {archive}; remove it and retry')
    return archive


def apply_patch(source, patch):
    # A build under the Pagenor checkout must not let git discover its parent
    # repository: otherwise git apply can silently skip every upstream path.
    subprocess.run(['git', 'init', '--quiet', str(source)], check=True)
    # Windows checkouts may convert the patch to CRLF, while the checksum-pinned
    # upstream tarballs contain LF. Normalize only the patch and ignore the build
    # machine's Git newline preferences; keep the original archives unchanged.
    normalized = source.parent / 'pagenor-helper.lf.patch'
    normalized.write_bytes(patch.read_bytes().replace(b'\r\n', b'\n'))
    command = ['git', '-c', 'core.autocrlf=false', '-c', 'core.safecrlf=false', 'apply']
    subprocess.run([*command, str(normalized)], cwd=source, check=True)
    subprocess.run([*command, '--reverse', '--check', str(normalized)], cwd=source, check=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--work', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--arch', choices=['x64', 'x86', 'arm64'])
    parser.add_argument('--boost-include', type=Path)
    args = parser.parse_args()
    if sys.version_info < (3, 12):
        parser.error('building the helper requires Python 3.12 or newer (not needed to run Pagenor)')
    root = Path(__file__).resolve().parent
    work, output = args.work.resolve(), args.output.resolve()
    cache, sources = work / 'downloads', work / 'sources'
    cache.mkdir(parents=True, exist_ok=True)
    sources.mkdir(exist_ok=True)
    names = ['libpagemaker-0.0.4', 'librevenge-0.0.6', 'zlib-1.3.1']
    if not args.boost_include:
        names.append('boost_1_86_0')
    archives = {}
    # Fresh sources ensure that a changed patch cannot reuse an old helper.
    for name in names:
        archive = fetch(name, cache)
        archives[name] = archive
        shutil.rmtree(sources / name, ignore_errors=True)
        with tarfile.open(archive) as tar:
            tar.extractall(sources, filter='data')
    apply_patch(sources / 'libpagemaker-0.0.4', root / 'pmd2raw-byte-v2.patch')
    boost = args.boost_include or sources / 'boost_1_86_0'
    if not (boost / 'boost/version.hpp').is_file():
        raise RuntimeError('Boost headers are missing')
    build = work / 'build'
    shutil.rmtree(build, ignore_errors=True)
    cmd = ['cmake', '-S', str(root), '-B', str(build), '-DCMAKE_BUILD_TYPE=Release',
           f'-DHELPER_SOURCES={sources}', f'-DBOOST_INCLUDE_DIR={boost.resolve()}']
    if args.arch:
        cmd += ['-G', 'Visual Studio 17 2022', '-A', {'x64': 'x64', 'x86': 'Win32', 'arm64': 'ARM64'}[args.arch]]
    subprocess.run(cmd, check=True)
    subprocess.run(['cmake', '--build', str(build), '--config', 'Release', '--parallel', '2'], check=True)
    subprocess.run(['cmake', '--install', str(build), '--config', 'Release', '--prefix', str(output)], check=True)
    legal = output / 'licenses/pagemaker'
    legal.mkdir(parents=True, exist_ok=True)
    # MPL corresponding source travels with every helper, including our modifications.
    for name in ['libpagemaker-0.0.4', 'librevenge-0.0.6']:
        shutil.copy2(archives[name], legal)
    for src, dest in [(root / 'pmd2raw-byte-v2.patch', 'pmd2raw-byte-v2.patch'),
                      (root / 'build.py', 'build.py'), (root / 'CMakeLists.txt', 'CMakeLists.txt'),
                      (root / 'README.md', 'README.md'),
                      (root / 'LICENSE-Boost.txt', 'LICENSE-Boost.txt'),
                      (sources / 'libpagemaker-0.0.4/COPYING', 'LICENSE-libpagemaker-MPL-2.0.txt'),
                      (sources / 'librevenge-0.0.6/COPYING.MPL', 'LICENSE-librevenge-MPL-2.0.txt'),
                      (sources / 'zlib-1.3.1/README', 'LICENSE-zlib.txt'),
                      (root / 'LICENSE-MPL-2.0.txt', 'LICENSE-MPL-2.0.txt')]:
        shutil.copy2(src, legal / dest)


if __name__ == '__main__':
    main()
