param([Parameter(Mandatory=$true)][string]$ReleaseRoot)
$ErrorActionPreference = 'Stop'
$manifest = Get-Content -LiteralPath 'assets/original-assets.manifest.json' -Raw | ConvertFrom-Json
$icons = @($manifest.files | Where-Object { $_.path.StartsWith('src-tauri/resources/icons/') })
$sevenZip = Join-Path $env:ProgramFiles '7-Zip/7z.exe'
if (!(Test-Path -LiteralPath $sevenZip)) { throw 'Preinstalled 7-Zip is required for bundle inspection.' }
$msis = @(Get-ChildItem -LiteralPath (Join-Path $ReleaseRoot 'bundle/msi') -Filter '*.msi' -File)
$nsis = @(Get-ChildItem -LiteralPath (Join-Path $ReleaseRoot 'bundle/nsis') -Filter '*.exe' -File)
if ($msis.Count -ne 1 -or $nsis.Count -ne 1) { throw 'Expected exactly one MSI and one NSIS installer.' }
$temporary = Join-Path $env:RUNNER_TEMP ('deanfinite-bundle-check-' + [Guid]::NewGuid())
New-Item -ItemType Directory -Path $temporary | Out-Null
function Extract-With7Zip([string]$archive, [string]$destination) {
    & $sevenZip x -y "-o$destination" $archive | Out-File -LiteralPath (Join-Path $temporary '7zip.log') -Append
    if ($LASTEXITCODE -ne 0) { throw "7-Zip failed to extract $archive" }
}
function Assert-IconHash([string]$path, $entry) {
    if (!(Test-Path -LiteralPath $path -PathType Leaf)) { throw "Missing packaged icon: $($entry.path)" }
    if ((Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -cne $entry.sha256) {
        throw "Packaged icon hash mismatch: $($entry.path)"
    }
}
try {
    # Resource copies beside the raw EXE must be exactly what resource_dir resolves.
    foreach ($entry in $icons) {
        Assert-IconHash (Join-Path $ReleaseRoot ('icons/' + [IO.Path]::GetFileName($entry.path))) $entry
    }
    $msiRoot = Join-Path $temporary 'msi'
    Extract-With7Zip $msis[0].FullName $msiRoot
    $cabRoot = Join-Path $temporary 'cab'
    $cabs = @(Get-ChildItem -LiteralPath $msiRoot -Filter '*.cab' -Recurse -File)
    foreach ($cab in $cabs) { Extract-With7Zip $cab.FullName $cabRoot }
    $payload = @(Get-ChildItem -LiteralPath $msiRoot -Recurse -File)
    if (Test-Path -LiteralPath $cabRoot) { $payload += @(Get-ChildItem -LiteralPath $cabRoot -Recurse -File) }
    $payloadByName = @{}
    foreach ($file in $payload) {
        if (!$payloadByName.ContainsKey($file.Name)) { $payloadByName[$file.Name] = @() }
        $payloadByName[$file.Name] += $file
    }
    $installer = New-Object -ComObject WindowsInstaller.Installer
    $database = $installer.OpenDatabase($msis[0].FullName, 0)
    $view = $database.OpenView('SELECT `File`, `FileName`, `FileSize` FROM `File`')
    $view.Execute()
    $records = @{}
    while ($null -ne ($record = $view.Fetch())) {
        $name = $record.StringData(2).Split('|')[-1]
        $records[$name] = @{ Id = $record.StringData(1); Size = $record.IntegerData(3) }
    }
    $view.Close()
    foreach ($entry in $icons) {
        $name = [IO.Path]::GetFileName($entry.path)
        $row = $records[$name]
        if (!$row -or $row.Size -ne $entry.bytes) { throw "MSI file table missing/wrong icon: $name" }
        $matches = @($payloadByName[$row.Id])
        if (!$matches[0]) { $matches = @($payloadByName[$name]) }
        if ($matches.Count -ne 1) { throw "Cannot uniquely locate MSI payload for $name ($($row.Id))" }
        Assert-IconHash $matches[0].FullName $entry
    }
    $nsisRoot = Join-Path $temporary 'nsis'
    Extract-With7Zip $nsis[0].FullName $nsisRoot
    $nsisFiles = @(Get-ChildItem -LiteralPath $nsisRoot -Recurse -File)
    $nsisByName = @{}
    foreach ($file in $nsisFiles) {
        if ($file.Directory.Name -ceq 'icons') {
            if ($nsisByName.ContainsKey($file.Name)) { throw 'Duplicate NSIS icon filename' }
            $nsisByName[$file.Name] = $file
        }
    }
    foreach ($entry in $icons) {
        $name = [IO.Path]::GetFileName($entry.path)
        $matches = @($nsisByName[$name])
        if ($matches.Count -ne 1) { throw "NSIS payload missing/duplicate icon: $name" }
        Assert-IconHash $matches[0].FullName $entry
    }
    $exeEntry = @{ path = 'Deanfinite.exe'; sha256 = (Get-FileHash -LiteralPath (Join-Path $ReleaseRoot 'Deanfinite.exe') -Algorithm SHA256).Hash.ToLowerInvariant() }
    $msiExeId = $records['Deanfinite.exe'].Id
    $msiExe = @($payloadByName[$msiExeId])
    if (!$msiExe[0]) { $msiExe = @($payloadByName['Deanfinite.exe']) }
    if ($msiExe.Count -ne 1) { throw 'MSI application payload missing/ambiguous' }
    Assert-IconHash $msiExe[0].FullName $exeEntry
    $nsisExe = @($nsisFiles | Where-Object { $_.Name -ceq 'Deanfinite.exe' })
    if ($nsisExe.Count -ne 1) { throw 'NSIS application payload missing/ambiguous' }
    Assert-IconHash $nsisExe[0].FullName $exeEntry
    Write-Output "Verified original bytes of all $($icons.Count) icons in release resources, MSI CAB and NSIS payload. Animations are embedded in Deanfinite.exe frontend."
} finally {
    # Only this invocation's own extraction directory is removed.
    Remove-Item -LiteralPath $temporary -Recurse -Force
}
