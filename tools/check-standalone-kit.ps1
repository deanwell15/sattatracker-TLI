param(
    [Parameter(Mandatory=$true)][string]$KitRoot,
    [string]$ContractPath = (Join-Path $PSScriptRoot 'RESOURCE_COMPATIBILITY.json')
)
$ErrorActionPreference = 'Stop'
$contract = Get-Content -LiteralPath $ContractPath -Raw | ConvertFrom-Json
if ($contract.schemaVersion -ne 1 -or $contract.directory -cne 'icons' -or
    $contract.protocol -cne 'deanfinite-icon' -or $contract.productName -cne 'Deanfinite' -or
    $contract.identifier -cne 'io.github.deanwell15.deanfinite') { throw 'Unsupported resource contract.' }
$exe = Join-Path $KitRoot 'Deanfinite.exe'
if (!(Test-Path -LiteralPath $exe -PathType Leaf)) { throw 'Existing standalone Deanfinite.exe is missing.' }
$icons = Join-Path $KitRoot 'icons'
$files = @(Get-ChildItem -LiteralPath $icons -File)
if ($files.Count -ne $contract.iconCount) { throw 'Wrong resource edition: icon count differs. Use a matching Full Build.' }
# Ordinal ordering is identical to the ASCII filename sort used by CI.
$names = [string[]]@($files | ForEach-Object { $_.Name })
[Array]::Sort($names, [StringComparer]::Ordinal)
$lines = [Text.StringBuilder]::new()
foreach ($name in $names) {
    if ($name -cnotmatch '^(?:[0-9]{1,12}|fe)\.png$') { throw 'Unexpected icon filename.' }
    $hash = (Get-FileHash -LiteralPath (Join-Path $icons $name) -Algorithm SHA256).Hash.ToLowerInvariant()
    [void]$lines.Append($name).Append(' ').Append($hash).Append("`n")
}
$sha = [Security.Cryptography.SHA256]::Create()
try { $digest = [BitConverter]::ToString($sha.ComputeHash([Text.Encoding]::UTF8.GetBytes($lines.ToString()))).Replace('-', '').ToLowerInvariant() }
finally { $sha.Dispose() }
if ($digest -cne $contract.iconsSha256) { throw 'Wrong or corrupted resource edition. Keep your backup and use a matching Full Build.' }
Write-Output "PASS: existing standalone resource layout and all $($files.Count) icon hashes match. Close Deanfinite, back up the kit, replace only EXE, keep icons/."
