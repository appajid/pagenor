#!/usr/bin/env python3
from pathlib import Path
import os, re, shutil, subprocess, sys

ROOT = Path(sys.argv[1] if len(sys.argv) > 1 else Path.home() / "Developer/pagenor").expanduser().resolve()
if not (ROOT / ".git").exists():
    raise SystemExit(f"Not a git repository: {ROOT}")

# Legal/provenance files must retain original DesignCraft/ArtCraft names.
PROTECTED = {
    "LICENSE-MIT", "LICENSE-APACHE", "NOTICE", "PAGENOR_UPSTREAM.md",
    "docs/brand/LICENSE-brand.txt",
}
SKIP_DIRS = {".git", "target", "dist"}

def run(*args):
    print("+", *map(str,args))
    subprocess.run([str(x) for x in args], cwd=ROOT, check=True)

# Require a clean-enough tree: preserve current Pagenor changes by committing them first if needed.
status = subprocess.check_output(["git","status","--porcelain"], cwd=ROOT, text=True)
if status.strip():
    print("Current changes detected; creating a checkpoint commit before full rename.")
    run("git","add","-A")
    # commit may fail only if nothing stageable
    subprocess.run(["git","commit","-m","Checkpoint before complete Pagenor rename"], cwd=ROOT)

# Product/app directories first.
dir_moves = [
    ("apps/designcraft-web", "apps/pagenor-web"),
    ("apps/designcraft-cli", "apps/pagenor-cli"),
    ("apps/designcraft", "apps/pagenor"),
]
for old,new in dir_moves:
    a,b=ROOT/old,ROOT/new
    if a.exists() and not b.exists():
        b.parent.mkdir(parents=True, exist_ok=True)
        a.rename(b)

# Rename ordinary files/dirs whose names contain designcraft, deepest first.
# Exclude docs/brand and provenance/legal material.
items=[]
for p in ROOT.rglob("*"):
    rel=p.relative_to(ROOT).as_posix()
    if any(part in SKIP_DIRS for part in p.relative_to(ROOT).parts): continue
    if rel in PROTECTED or rel.startswith("docs/brand/"): continue
    if "designcraft" in p.name.lower():
        items.append(p)
for p in sorted(items, key=lambda x: len(x.parts), reverse=True):
    if not p.exists(): continue
    name=p.name
    # Preserve case pattern in filenames.
    name=re.sub("DesignCraft","Pagenor",name)
    name=re.sub("DESIGNCRAFT","PAGENOR",name)
    name=re.sub("designcraft","pagenor",name)
    q=p.with_name(name)
    if q != p and not q.exists():
        p.rename(q)

# Text replacements in source/config/docs, excluding legal/provenance and binary assets.
text_exts={".rs",".toml",".lock",".md",".txt",".yml",".yaml",".json",".sh",".ps1",".html",".css",".js",".ts",".xml",".desktop",".metainfo",".plist",".rc",".wxs",".svg",".pat",".dic"}
special_names={"Cargo.toml","Cargo.lock","Makefile","README","README.md",".gitignore"}
pairs = [
    ("DESIGNCRAFT_", "PAGENOR_"),
    ("DesignCraft", "Pagenor"),
    ("designcraft", "pagenor"),
    (".dcbook", ".pbook"),
    (".dclib", ".plib"),
    (".dcscript", ".pscript"),
]
for p in ROOT.rglob("*"):
    if not p.is_file(): continue
    rel=p.relative_to(ROOT).as_posix()
    if any(part in SKIP_DIRS for part in p.relative_to(ROOT).parts): continue
    if rel in PROTECTED or rel.startswith("docs/brand/"): continue
    if p.suffix.lower() not in text_exts and p.name not in special_names: continue
    try:
        s=p.read_text()
    except UnicodeDecodeError:
        continue
    old=s
    for a,b in pairs: s=s.replace(a,b)
    # Application IDs from upstream.
    s=s.replace("ai.storyteller.pagenor", "org.pagenor.app")
    if s != old:
        p.write_text(s)

# Root package metadata: repo URL must not falsely point to upstream as if Pagenor were DesignCraft.
cargo=ROOT/"Cargo.toml"
s=cargo.read_text()
s=s.replace('repository = "https://github.com/storytold/pagenor"', '# repository = set when the Pagenor repository is published')
cargo.write_text(s)

# Ensure application folders are expected.
assert (ROOT/"apps/pagenor").exists(), "apps/pagenor missing after rename"

# Add explicit provenance document after mechanical rename.
prov=ROOT/"PAGENOR_UPSTREAM.md"
prov.write_text("""# Pagenor upstream provenance

Pagenor is an independent derivative of DesignCraft.

Initial upstream DesignCraft revision:
`f4a4c9709b7c552800f15ba3d1f0f68dd75fa165`

Original DesignCraft copyright:
Copyright (c) 2026 ArtCraft Team and the DesignCraft contributors.

The upstream software is available under MIT OR Apache-2.0. Pagenor retains
the applicable original copyright, licence, attribution, and third-party
licence notices.

The ArtCraft and DesignCraft names are referenced here only to identify the
origin of the upstream work. Pagenor does not use ArtCraft trademarks as its
product identity.
""")

# Restore original legal NOTICE from git if mechanical changes somehow touched it.
# It was protected, so normally no-op.
# Validate Cargo metadata first.
run("cargo","fmt")
run("cargo","check","-p","pagenor")
run("cargo","test","-p","pagenor-fonts","-p","pagenor-compose")

# Search remaining names, ignoring legal/provenance, git and build output.
remaining=[]
for p in ROOT.rglob("*"):
    if not p.is_file(): continue
    rel=p.relative_to(ROOT).as_posix()
    if any(part in SKIP_DIRS for part in p.relative_to(ROOT).parts): continue
    if rel in PROTECTED or rel.startswith("docs/brand/"): continue
    try: s=p.read_text(errors="ignore")
    except Exception: continue
    if re.search(r"designcraft|DesignCraft|DESIGNCRAFT", s):
        remaining.append(rel)
    if re.search(r"designcraft|DesignCraft|DESIGNCRAFT", p.name):
        remaining.append(rel)
remaining=sorted(set(remaining))

report=ROOT/"PAGENOR_RENAME_REPORT.md"
report.write_text("# Pagenor full rename report\n\n"
                  "Internal application/package identifiers were migrated from DesignCraft to Pagenor.\n\n"
                  "Original DesignCraft naming is intentionally retained only in legal/provenance material.\n\n"
                  "## Non-protected leftovers\n\n" +
                  ("\n".join(f"- `{x}`" for x in remaining) if remaining else "None detected.") + "\n")

run("git","add","-A")
run("git","commit","-m","Rename DesignCraft-derived codebase completely to Pagenor")

print("\n=== Pagenor migration complete ===")
print("Desktop package: pagenor")
print("Font crate:       pagenor-fonts")
print("Compose crate:    pagenor-compose")
print("App directory:    apps/pagenor")
print("CLI directory:    apps/pagenor-cli")
print("Web directory:    apps/pagenor-web")
print("Report:           PAGENOR_RENAME_REPORT.md")
print("\nLaunch with:")
print("  cargo run --release -p pagenor")
