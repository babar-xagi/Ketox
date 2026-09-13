[CmdletBinding()]
param(
    [string]$Kotlinc = $env:KETOX_KOTLINC
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
# Check native exit codes explicitly, including on PowerShell 7.3+.
$PSNativeCommandUseErrorActionPreference = $false

if ([string]::IsNullOrWhiteSpace($Kotlinc)) { $Kotlinc = 'kotlinc' }
$compiler = Get-Command -Name $Kotlinc -CommandType Application -ErrorAction SilentlyContinue
if ($null -eq $compiler) {
    $root = Split-Path -Parent $PSScriptRoot
    $bundled = Join-Path $root '.tools\kotlin-2.2.0\kotlinc\bin\kotlinc.bat'
    if (Test-Path -LiteralPath $bundled) {
        $compiler = Get-Command -Name $bundled -CommandType Application -ErrorAction SilentlyContinue
    }
}
if ($null -eq $compiler) {
    throw "Kotlin compiler not found: $Kotlinc. Install Kotlin 2.2.0 and put kotlinc on PATH, or pass -Kotlinc 'C:\path\kotlinc.bat' (or set KETOX_KOTLINC)."
}
$java = Get-Command -Name java -CommandType Application -ErrorAction SilentlyContinue
if ($null -eq $java) { throw 'Java not found. Install JDK 21 and put java on PATH.' }
$cargo = Get-Command -Name cargo -CommandType Application -ErrorAction SilentlyContinue
if ($null -eq $cargo) { throw 'Cargo not found. Install Rust and put cargo on PATH.' }

$projectRoot = Split-Path -Parent $PSScriptRoot
Push-Location -LiteralPath $projectRoot
try {
    $metadataJson = & $cargo.Source metadata --locked --format-version=1 --no-deps
    if ($LASTEXITCODE -ne 0) { throw 'cargo metadata failed.' }
    $metadata = ($metadataJson -join "`n") | ConvertFrom-Json
    $packages = @($metadata.packages | Where-Object { $_.name -eq 'ketox-hello' })
    if ($packages.Count -ne 1) { throw 'Expected exactly one ketox-hello workspace package.' }
    $packageId = $packages[0].id

    Write-Host 'Building the Ketox native example...'
    $cargoOutput = @(& $cargo.Source build --locked -p ketox-hello --message-format=json)
    $buildExitCode = $LASTEXITCODE
    $messages = @($cargoOutput | Where-Object { -not [string]::IsNullOrWhiteSpace($_) } | ForEach-Object { $_ | ConvertFrom-Json })
    foreach ($message in $messages) {
        if ($message.reason -eq 'compiler-message' -and $message.message.rendered) {
            Write-Host $message.message.rendered
        }
    }
    if ($buildExitCode -ne 0) { throw "cargo build failed (exit $buildExitCode)." }

    # Match the package ID, not a stale build directory left under target/.
    $outputDirectories = @($messages | Where-Object {
        $_.reason -eq 'build-script-executed' -and $_.package_id -eq $packageId
    } | ForEach-Object { $_.out_dir } | Select-Object -Unique)
    $nativeLibraries = @($messages | Where-Object {
        $_.reason -eq 'compiler-artifact' -and $_.package_id -eq $packageId -and $_.target.kind -contains 'cdylib'
    } | ForEach-Object { $_.filenames } | Where-Object { $_ -match '\.(dll|so|dylib)$' } | Select-Object -Unique)
    if ($outputDirectories.Count -ne 1) { throw 'Cargo did not report exactly one generated output directory for ketox-hello.' }
    if ($nativeLibraries.Count -ne 1) { throw 'Cargo did not report exactly one native library for ketox-hello.' }

    $generatedKotlin = Join-Path $outputDirectories[0] 'RustApi.kt'
    $smokeSource = Join-Path $projectRoot 'integration-tests/jvm/Smoke.kt'
    foreach ($artifact in @($generatedKotlin, $smokeSource, $nativeLibraries[0])) {
        if (-not (Test-Path -LiteralPath $artifact -PathType Leaf)) { throw "Required artifact missing: $artifact" }
    }
    $testOutput = Join-Path $metadata.target_directory 'ketox-jvm'
    New-Item -ItemType Directory -Force -Path $testOutput | Out-Null
    $jar = Join-Path $testOutput 'smoke.jar'
    $nativeDirectory = Split-Path -Parent $nativeLibraries[0]

    Write-Host 'Compiling generated Kotlin and JVM integration checks...'
    & $compiler.Source $generatedKotlin $smokeSource -jvm-target 21 -include-runtime -d $jar
    if ($LASTEXITCODE -ne 0) { throw "Kotlin compilation failed (exit $LASTEXITCODE)." }

    Write-Host 'Running JVM integration checks with -Xcheck:jni...'
    & $java.Source '-Xcheck:jni' "-Djava.library.path=$nativeDirectory" -cp $jar 'dev.ketox.example.SmokeKt'
    if ($LASTEXITCODE -ne 0) { throw "JVM integration checks failed (exit $LASTEXITCODE)." }
    Write-Host "Ketox JVM integration checks passed. Artifact: $jar"
} finally {
    Pop-Location
}
