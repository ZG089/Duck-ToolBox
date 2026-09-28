param(
  [string]$BinaryPath = "target\aarch64-linux-android\release\duckd",
  [string]$DistDir = "dist"
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$moduleDir = Join-Path $repoRoot "module"
$modulePropPath = Join-Path $moduleDir "module.prop"
$binaryFullPath = Join-Path $repoRoot $BinaryPath
$distRoot = Join-Path $repoRoot $DistDir

if (-not (Test-Path $binaryFullPath)) {
  throw "Android backend binary not found at '$binaryFullPath'. Build it first or pass -BinaryPath."
}

$webrootIndex = Join-Path $moduleDir "webroot\index.html"
if (-not (Test-Path $webrootIndex)) {
  throw "WebUI build output is missing at '$webrootIndex'. Run 'pnpm build' in ui first."
}

$versionLine = Get-Content $modulePropPath | Where-Object { $_ -match '^version=' } | Select-Object -First 1
if (-not $versionLine) {
  throw "Could not read version from '$modulePropPath'."
}

$version = $versionLine.Split('=', 2)[1].Trim()
New-Item -ItemType Directory -Force -Path $distRoot | Out-Null

Copy-Item (Join-Path $repoRoot "README.md") (Join-Path $moduleDir "README.md") -Force
Copy-Item (Join-Path $repoRoot "CHANGELOG.md") (Join-Path $moduleDir "CHANGELOG.md") -Force
Copy-Item $binaryFullPath (Join-Path $moduleDir "bin\duckd") -Force

$archivePath = Join-Path $distRoot "duck-toolbox-$version.zip"
if (Test-Path $archivePath) {
  Remove-Item $archivePath -Force
}

Compress-Archive -Path "$moduleDir\*" -DestinationPath $archivePath -CompressionLevel Optimal

Write-Host "Module archive created at $archivePath"
