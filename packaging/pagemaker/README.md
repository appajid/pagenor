# Packaged PageMaker converter

Pagenor executes this helper as a separate process. No C++ is linked into the Rust
application. libpagemaker 0.0.4 is MPL-2.0; librevenge 0.0.6 is used under its
MPL-2.0 option (not its alternative LGPL licence). Boost headers are BSL-1.0 and
zlib is under its own permissive licence. Original notices remain in the sources.
All inputs and checksums are pinned in `build.py`; no proprietary fonts or user
publications are included. Source URLs in that file are the upstream projects'
public distributions, not files from any Adobe installation.

Python 3.12+, Git, CMake 3.20+ (on Windows, a version supporting the installed Visual Studio) and Visual Studio C++ tools (2022 or newer) (including the
selected target architecture) are needed on the build machine only:

    python packaging/pagemaker/build.py --arch x64 --work target/pagemaker-x64 --output target/helper-x64

Use x86 or arm64 for those Windows packages; omit --arch for a native Unix build.
For Mac release bundles use --macos-arch universal (both arm64 and x86_64),
aarch64 or x86_64. A Mac with Xcode command-line tools and CMake builds these;
no Visual Studio is involved. The app carries the helper inside
Contents/MacOS/helpers, with corresponding source/notices under
Contents/Resources/Legal/licenses/pagemaker. The standalone CLI ZIP also carries
helpers and licenses/pagemaker. All helper libraries are linked statically; only
macOS system libraries are permitted by the packaged-helper verification.
The helper and C/C++ runtime are statically linked on Windows. Users need neither
a development environment, Homebrew/MSYS2, nor a separate helper installation.
Packaging must fail if it cannot build this required component.

`helpers/pmd2raw-byte-v2.exe` is selected relative to the installed application or
portable executable, independent of PATH/current directory. Explicit converter
overrides retain precedence. The V2 patch preserves source bytes in hex transport,
story identity, embedded raster continuation and all referenced page containers.
It originates in Pagenor's documented October 2026 local MPL-helper work.

The helper's `licenses/pagemaker` folder includes the original MPL source archives,
patch, licence notices and these build instructions. To rebuild from supplied
archives, put them in WORK/downloads, restore the build scripts, patch, README and licence files to one directory,
then run build.py there. Boost and zlib come from the pinned public sources.
Import remains experimental: legacy fonts must be supplied legally by the user;
font conversion, complete PageMaker layout fidelity and automatic Unicode
conversion are not implied by bundling the helper.
