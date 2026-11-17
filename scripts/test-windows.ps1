[CmdletBinding()]
param(
    [ValidateSet('debug', 'release')]
    [string]$Profile = 'release',
    [string]$Target = 'x86_64-pc-windows-msvc',
    [string]$ArtifactPath = ''
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repo = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Set-Location $repo

function Invoke-Step([string]$Name, [scriptblock]$Command) {
    Write-Host "`n=== $Name ===" -ForegroundColor Cyan
    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "$Name failed with exit code $LASTEXITCODE"
    }
}

Write-Host "Repository: $repo"
Write-Host "Windows: $([System.Environment]::OSVersion.VersionString)"
Invoke-Step 'Rust version' { rustc --version; cargo --version }
Invoke-Step 'Target availability' { rustup target list --installed }

$installedTargets = @(rustup target list --installed | ForEach-Object { ($_ -split '\s+')[0] })
if ($installedTargets -notcontains $Target) {
    throw "Rust target '$Target' is not installed. Run: rustup target add $Target"
}

Invoke-Step 'Formatting' { cargo fmt --all -- --check }
Invoke-Step 'Cargo check' { cargo check --target $Target }
Invoke-Step 'Rust tests' { cargo test --target $Target }

$buildArgs = @('build', '--target', $Target)
if ($Profile -eq 'release') { $buildArgs += '--release' }
Invoke-Step "Build ($Profile)" { cargo @buildArgs }

$profileDir = if ($Profile -eq 'release') { 'release' } else { 'debug' }
$exe = Join-Path $repo "target\$Target\$profileDir\zTranz.exe"
if (-not (Test-Path -LiteralPath $exe -PathType Leaf)) {
    throw "Expected executable was not produced: $exe"
}

if ($ArtifactPath -eq '') {
    $ArtifactPath = Join-Path $repo "artifacts\zTranz-windows-x64-$Profile.exe"
}
$artifactDir = Split-Path -Parent $ArtifactPath
New-Item -ItemType Directory -Force -Path $artifactDir | Out-Null
Copy-Item -LiteralPath $exe -Destination $ArtifactPath -Force
$size = (Get-Item -LiteralPath $ArtifactPath).Length
if ($size -le 0) { throw "Artifact is empty: $ArtifactPath" }
Write-Host "`nPASS: $ArtifactPath ($size bytes)" -ForegroundColor Green
