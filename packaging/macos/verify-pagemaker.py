#!/usr/bin/env python3
"""Verify the shipped Mac helper using only an original synthetic publication."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', type=Path, required=True)
    parser.add_argument('--legal', type=Path, required=True)
    parser.add_argument('--fixture', type=Path, required=True)
    parser.add_argument('--arch', choices=['universal', 'aarch64', 'x86_64'], required=True)
    args = parser.parse_args()
    directory, legal, fixture = args.directory.resolve(), args.legal.resolve(), args.fixture.resolve()
    helper, cli = directory / 'helpers/pmd2raw-byte-v2', directory / 'pagenor-cli'
    for name in ['libpagemaker-0.0.4.tar.xz', 'librevenge-0.0.6.tar.xz', 'pmd2raw-byte-v2.patch',
                 'LICENSE-MPL-2.0.txt', 'LICENSE-Boost.txt', 'LICENSE-zlib.txt', 'build.py', 'CMakeLists.txt']:
        if not (legal / name).is_file():
            raise RuntimeError(f'missing helper source/notice: {name}')
    slices = {'universal': ['arm64', 'x86_64'], 'aarch64': ['arm64'], 'x86_64': ['x86_64']}[args.arch]
    for binary in [helper, cli]:
        for architecture in slices:
            subprocess.run(['/usr/bin/lipo', str(binary), '-verify_arch', architecture], check=True)
        subprocess.run(['/usr/bin/codesign', '--verify', '--strict', str(binary)], check=True)
    linked = subprocess.check_output(['/usr/bin/otool', '-L', str(helper)], text=True)
    for line in linked.splitlines():
        if line.startswith((' ', '\t')):
            dependency = line.strip().split(' (', 1)[0]
            if not dependency.startswith(('/usr/lib/', '/System/Library/')):
                raise RuntimeError(f'helper requires a non-system library: {dependency}')
    with tempfile.TemporaryDirectory(prefix='pagenor-packaged-pagemaker-') as temp:
        env = {k: v for k, v in os.environ.items() if not k.startswith('PAGENOR_PAGEMAKER_')}
        env.update(HOME=temp, PATH='/usr/bin:/bin', XDG_DATA_HOME=temp)
        version = subprocess.check_output([str(helper), '--version'], env=env, text=True)
        if '0.0.4-pagenor-byte-v2' not in version:
            raise RuntimeError(f'wrong helper: {version}')
        saved = Path(temp) / 'three-pages.pgnr'
        subprocess.run([str(cli), 'run', '--in', str(fixture), '--export', str(saved)], env=env, check=True)
        report = json.loads(subprocess.check_output([str(cli), 'run', '--in', str(saved), '--cmd', 'document.inspect'], env=env))
        expected = json.loads(fixture.with_suffix('.expected.json').read_text())
        if report.get('pageCount') != expected['pages']:
            raise RuntimeError(f'PageMaker/native round trip lost pages: {report}')
        stories = report.get('stories', [])
        if len(stories) != 1 or stories[0].get('preview') != expected['text'] + '\n':
            raise RuntimeError('PageMaker text transport did not decode the English and high-byte source text')
    print('ok packaged Mac converter: matching slices, signed, system libraries only, three pages in two PAGE containers, exact English/high-byte text, native save/reopen, isolated HOME/PATH')


if __name__ == '__main__':
    main()
