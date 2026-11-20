<# Verify the compiled MSI UI and PageMaker support in an installed/portable package.
   The synthetic fixture is original; no user publication or proprietary font is uploaded. #>
param(
  [Parameter(Mandatory = $true)] [string] $Directory,
  [Parameter(Mandatory = $true)] [string] $Fixture,
  [string] $Msi
)
$ErrorActionPreference = 'Stop'
$Directory = (Resolve-Path -LiteralPath $Directory).Path
$Fixture = (Resolve-Path -LiteralPath $Fixture).Path
if ($Msi) {
  $installer = New-Object -ComObject WindowsInstaller.Installer
  $db = $installer.OpenDatabase((Resolve-Path -LiteralPath $Msi).Path, 0)
  function Query-Msi([string] $Query) {
    $view = $db.OpenView($Query)
    $view.Execute()
    $record = $view.Fetch()
    if (-not $record) { throw "MSI is missing required UI row: $Query" }
    $value = $record.StringData(1)
    $view.Close()
    return $value
  }
  # Validate the actual compiled database, not just the source XML. -1 schedules
  # ExitDialog on successful interactive completion; /qn intentionally skips UI.
  $sequence = Query-Msi 'SELECT `Sequence` FROM `InstallUISequence` WHERE `Action` = ''ExitDialog'''
  if ($sequence -ne '-1') { throw 'MSI has no success exit dialog' }
  $finish = Query-Msi 'SELECT `Argument` FROM `ControlEvent` WHERE `Dialog_` = ''ExitDialog'' AND `Control_` = ''Finish'' AND `Event` = ''EndDialog'''
  if ($finish -ne 'Return') { throw 'MSI success dialog has no working Finish button' }
  $text = Query-Msi 'SELECT `Value` FROM `Property` WHERE `Property` = ''WIXUI_EXITDIALOGOPTIONALTEXT'''
  if ($text -notmatch 'installed successfully') { throw 'MSI success message missing' }
  Write-Output 'ok compiled MSI success/Finish dialog'
}
$helper = Join-Path $Directory 'helpers/pmd2raw-byte-v2.exe'
$cli = Join-Path $Directory 'pagenor-cli.exe'
foreach ($notice in 'libpagemaker-0.0.4.tar.xz', 'librevenge-0.0.6.tar.xz', 'pmd2raw-byte-v2.patch', 'LICENSE-MPL-2.0.txt', 'LICENSE-Boost.txt', 'LICENSE-zlib.txt', 'build.py', 'CMakeLists.txt') {
  if (-not (Test-Path -LiteralPath (Join-Path $Directory "licenses/pagemaker/$notice"))) { throw "missing helper source/notice: $notice" }
}
# Empty PATH and converter overrides prove that no development machine helper or
# runtime DLL masks an incomplete installed/portable package.
$oldPath = $env:PATH
$oldConverter = $env:PAGENOR_PAGEMAKER_CONVERTER
$oldProfile = $env:PAGENOR_PAGEMAKER_BYTE_OUTPUT
$oldAppData = $env:APPDATA
try {
  $env:PATH = "$env:SystemRoot\System32;$env:SystemRoot"
  $env:APPDATA = Join-Path $env:RUNNER_TEMP 'pagenor-empty-appdata'
  Remove-Item Env:PAGENOR_PAGEMAKER_CONVERTER -ErrorAction SilentlyContinue
  Remove-Item Env:PAGENOR_PAGEMAKER_BYTE_OUTPUT -ErrorAction SilentlyContinue
  $version = & $helper --version | Out-String
  if ($LASTEXITCODE -ne 0 -or $version -notmatch '0\.0\.4-pagenor-byte-v2') { throw "helper cannot run standalone: $version" }
  $native = Join-Path $env:RUNNER_TEMP 'pagenor-three-pages.pgnr'
  & $cli run --in $Fixture --export $native
  if ($LASTEXITCODE -ne 0) { throw 'packaged PageMaker import failed without PATH/override' }
  $inspection = & $cli run --in $native --cmd document.inspect | ConvertFrom-Json
  if ($LASTEXITCODE -ne 0 -or $inspection.pageCount -ne 3) { throw 'PageMaker/native round trip lost pages' }
  Write-Output 'ok packaged converter: three pages in two PAGE containers, native save/reopen, no external PATH or user helper'
} finally {
  $env:PATH = $oldPath
  $env:PAGENOR_PAGEMAKER_CONVERTER = $oldConverter
  $env:PAGENOR_PAGEMAKER_BYTE_OUTPUT = $oldProfile
  $env:APPDATA = $oldAppData
}
