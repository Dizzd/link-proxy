param([Parameter(Mandatory = $true)][string]$SettingsPath)
$ErrorActionPreference = 'Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)
$browserSettings = Get-Content -LiteralPath $SettingsPath -Raw | ConvertFrom-Json
foreach ($property in $browserSettings.PSObject.Properties) {
    [Environment]::SetEnvironmentVariable($property.Name, $property.Value, 'Process')
}
& node node_modules/@playwright/test/cli.js test
$browserExitCode = $LASTEXITCODE
[IO.File]::WriteAllText([IO.Path]::ChangeExtension($SettingsPath, '.result'), [string]$browserExitCode)
exit $browserExitCode
