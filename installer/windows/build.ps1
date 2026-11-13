param(
    [string]$Version = "0.1.0",
    [string]$OutDir = "..\..\target\installer"
)
$ErrorActionPreference = "Stop"
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
# Requires WiX v4: https://wixtoolset.org
if (-not (Get-Command wix -ErrorAction SilentlyContinue)) {
    Write-Error "wix.exe not found. Install WiX v4 and add to PATH."
    exit 1
}
wix build Myco.wxs -o "$OutDir\Mycor-$Version.msi" -d "Version=$Version"
Write-Host "built $OutDir\Mycor-$Version.msi"