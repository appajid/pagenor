<# Exercise the update installer against the immutable public 0.14.1 MSI.
   Uses only synthetic documents/settings on an ephemeral Windows test runner. #>
param(
  [ValidateSet('x64', 'x86', 'arm64')] [string] $Arch,
  [Parameter(Mandatory = $true)] [string] $Msi,
  [Parameter(Mandatory = $true)] [string] $Directory,
  [Parameter(Mandatory = $true)] [string] $Fixture
)
$ErrorActionPreference = 'Stop'
$Msi = (Resolve-Path -LiteralPath $Msi).Path
$manifest = Get-Content -LiteralPath (Join-Path $PSScriptRoot '../../Cargo.toml') -Raw
$ExpectedVersion = [regex]::Match($manifest, '(?ms)^\[workspace\.package\]\s*.*?^version\s*=\s*"([^"]+)"').Groups[1].Value
if (-not $ExpectedVersion) { throw 'workspace version is missing' }
$MsiVersion = ($ExpectedVersion -split '-')[0]
$hashes = @{
  x64 = '1c471bfc5dfdf411f5f5d1b07da6a007b19ef6f158fb31125e1dd9b7d43c103a'
  x86 = '4bf5b079cc97dd014c8f6d7ec93d2989ae822f62c5ea3a9bc688a23e3d472cfa'
  arm64 = 'c9362ae5a7306de72165ba9c4361a034317f52301642a450029b4792af436986'
}
$previous = Join-Path $env:RUNNER_TEMP "pagenor-0.14.1-windows-$Arch.msi"
Invoke-WebRequest -Uri "https://github.com/appajid/pagenor/releases/download/v0.14.1/pagenor-0.14.1-windows-$Arch.msi" -OutFile $previous
if ((Get-FileHash -Algorithm SHA256 -LiteralPath $previous).Hash.ToLowerInvariant() -ne $hashes[$Arch]) { throw 'previous MSI checksum mismatch' }
function Run-Msi([string] $Operation, [string] $Path) {
  $proc = Start-Process msiexec.exe -ArgumentList $Operation, "`"$Path`"", '/qn', '/l*v', "`"$env:RUNNER_TEMP/pagenor-update.log`"" -Wait -PassThru
  if ($proc.ExitCode -ne 0) { Get-Content "$env:RUNNER_TEMP/pagenor-update.log" -Tail 60; throw "Windows Installer returned $($proc.ExitCode)" }
}
$installed = $null
$settings = Join-Path ([Environment]::GetFolderPath('ApplicationData')) 'Pagenor/ui.json'
$document = Join-Path $env:RUNNER_TEMP 'saved-before-update.pgnr'
$desktop = Join-Path ([Environment]::GetFolderPath('CommonDesktopDirectory')) 'Pagenor.lnk'
try {
  Run-Msi '/i' $previous
  $installed = $previous
  $cli = Join-Path $Directory 'pagenor-cli.exe'
  $version = & $cli --version | Out-String
  if ($LASTEXITCODE -ne 0 -or $version -notmatch '0\.14\.1') { throw 'previous version did not install' }
  [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($settings)) | Out-Null
  [IO.File]::WriteAllText($settings, '{"brightness":"dark"}')
  & $cli run --sample --export $document
  if ($LASTEXITCODE -ne 0) { throw 'previous version could not save the synthetic document' }
  $settingsHash = (Get-FileHash -LiteralPath $settings -Algorithm SHA256).Hash
  $documentHash = (Get-FileHash -LiteralPath $document -Algorithm SHA256).Hash
  Run-Msi '/i' $Msi
  $installed = $Msi
  $version = & $cli --version | Out-String
  if ($LASTEXITCODE -ne 0 -or $version -notmatch [regex]::Escape($ExpectedVersion)) { throw 'update did not replace 0.14.1' }
  if ((Get-FileHash -LiteralPath $settings -Algorithm SHA256).Hash -ne $settingsHash) { throw 'update changed user settings' }
  if ((Get-FileHash -LiteralPath $document -Algorithm SHA256).Hash -ne $documentHash) { throw 'update changed the saved document' }
  & $cli run --in $document --cmd document.inspect
  if ($LASTEXITCODE -ne 0) { throw 'updated app cannot reopen the previous version document' }
  if (-not (Test-Path -LiteralPath $desktop)) { throw 'update did not create a desktop shortcut' }
  $shortcut = (New-Object -ComObject WScript.Shell).CreateShortcut($desktop)
  if ($shortcut.WorkingDirectory.TrimEnd('\') -ne $Directory.TrimEnd('\')) { throw 'desktop shortcut points at the wrong installation' }
  $products = @(Get-ItemProperty 'HKLM:/Software/Microsoft/Windows/CurrentVersion/Uninstall/*', 'HKLM:/Software/Wow6432Node/Microsoft/Windows/CurrentVersion/Uninstall/*' -ErrorAction SilentlyContinue | Where-Object { $_.DisplayName -eq 'Pagenor' })
  if ($products.Count -ne 1 -or $products[0].DisplayVersion -ne $MsiVersion) { throw 'upgrade left duplicate or stale installed products' }
  & (Join-Path $PSScriptRoot 'verify-pagemaker.ps1') -Directory $Directory -Msi $Msi -Fixture $Fixture
  Write-Output "ok 0.14.1 -> $ExpectedVersion update: one installed product, desktop shortcut, saved document/settings preserved, PageMaker available"
} finally {
  if ($installed) { Run-Msi '/x' $installed }
}
if (Test-Path -LiteralPath $desktop) { throw 'uninstall left the desktop shortcut behind' }
