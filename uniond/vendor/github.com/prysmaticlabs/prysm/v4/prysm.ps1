$folderDist = "dist";
$ProgressPreference = 'SilentlyContinue' # Disable Invoke-WebRequest progress bar, makes it silent and faster.

# Complain if invalid arguments were provided.
if ("beacon-chain", "validator", "client-stats" -notcontains $args[0]) {
    Write-Host @"
Usage: ./prysm.sh1 PROCESS FLAGS.

PROCESS can be beacon-chain, validator, or client-stats.
FLAGS are the flags or arguments passed to the PROCESS.
 
Use this script to download the latest Prysm release binaries.
Downloaded binaries are saved to .\dist
 
To specify a specific release version:
  `$env:USE_PRYSM_VERSION="v1.0.0-beta.3"
 to resume using the latest release:
  Remove-Item env:USE_PRYSM_VERSION
 
To automatically restart crashed processes:
  `$env:PRYSM_AUTORESTART=`$TRUE ; .\prysm.sh1 beacon-chain
 to stop autorestart run:
  Remove-Item env:PRYSM_AUTORESTART
"@;

    exit;
}

# 64bit check.
if (-not [Environment]::Is64BitOperatingSystem) {
    Write-Host "ERROR: prysm is only supported on 64-bit Operating Systems" -ForegroundColor Red;
    exit;
}

# Create directory for us to download our binaries to, if it not yet exists.
if (-not (Test-Path $folderDist)) {
    New-Item -Path . -Name $folderDist -ItemType "directory";
}

# Determine the latest release version, unless specified by $USE_PRYSM_VERSION.
if (Test-Path env:USE_PRYSM_VERSION) {
    Write-Host "Detected variable `$env:USE_PRYSM_VERSION=$env:USE_PRYSM_VERSION";
    $version = $env:USE_PRYSM_VERSION;
}
else {
    try {
        $response = Invoke-WebRequest -Uri "https://prysmaticlabs.com/releases/latest";
        $version = $response.Content.Trim();

        Write-Host "Using (latest) prysm version: $version";
    }
    catch {
        Write-Host "ERROR: Failed to get the latest version:" -ForegroundColor Red;
        Write-Host "  $($_.Exception.Message)" -ForegroundColor Red;
        exit;
    }
}

# Make sure the binary we want to use is up to date, otherwise download (and possibly verify) it.
$fileName = "$($args[0])-$version-windows-amd64.exe";
$folderBin = "$folderDist\$fileName";

if ((Test-Path $folderBin) -and (Test-Path "$folderBin.sha256") -and (Test-Path "$folderBin.sig")) {
    Write-Host "$($args[0]) is up to date with version: $version" -ForegroundColor Green;
}
else {
    try {
        Write-Host "Downloading $fileName" -ForegroundColor Green;

        Invoke-WebRequest -Uri "https://prysmaticlabs.com/releases/$fileName" -OutFile "$folderBin";
        Invoke-WebRequest -Uri "https://prysmaticlabs.com/releases/$fileName.sha256" -OutFile "$folderBin.sha256";
        Invoke-WebRequest -Uri "https://prysmaticlabs.com/releases/$fileName.sig" -OutFile "$folderBin.sig";

        Write-Host "Downloading complete!" -ForegroundColor Green;
    }
    catch {
        Write-Host "ERROR: Failed to get download $fileName`:" -ForegroundColor Red;
        Write-Host "  $($_.Exception.Message)" -ForegroundColor Red;
        exit;
    }
}

# GPG not natively available on Windows, external module required.
Write-Host "WARN GPG verification is not natively available on Windows" -ForegroundColor Yellow;
Write-Host "WARN Skipping integrity verification of downloaded binary" -ForegroundColor Yellow;

# Check SHA256 File Hash before running
Write-Host "Verifying binary authenticity with SHA256 Hash";
$hashExpected = (Get-Content -Path "$folderBin.sha256" -Delimiter " ")[0].Trim().ToUpperInvariant();
$hashActual = (Get-FileHash -Path $folderBin -Algorithm SHA256).Hash.ToUpperInvariant();
if ($hashExpected -eq $hashActual) {
    Write-Host "SHA256 Hash Match!" -ForegroundColor Green;
}
elseif ((Test-Path env:PRYSM_ALLOW_UNVERIFIED_BINARIES) -and $env:PRYSM_ALLOW_UNVERIFIED_BINARIES -eq $TRUE) {
    Write-Host "WARN Failed to verify prysm binary" -ForegroundColor Yellow;
    Write-Host "Detected `$env:PRYSM_ALLOW_UNVERIFIED_BINARIES=`$TRUE";
    Write-Host "Proceeding...";
}
else {
    Write-Host "ERROR Failed to verify prysm binary" -ForegroundColor Red;
    Write-Host @"
Please erase downloads in the
dist directory and run this script again. Alternatively, you can use a
a prior version by specifying environment variable `$env:USE_PRYSM_VERSION
with the specific version, as desired. Example: `$env:USE_PRYSM_VERSION=v1.0.0-alpha.5
If you must wish to continue running an unverified binary, use:
`$env:PRYSM_ALLOW_UNVERIFIED_BINARIES=`$TRUE
"@;

    exit;
}

# Finally, start the process.
do {
    Write-Host "Starting: prysm $($args -join ' ')";

    $argumentList = $args | Select-Object -Skip 1;

    if ($argumentList.Length -gt 0) {
        $process = Start-Process -FilePath $folderBin -ArgumentList $argumentList -NoNewWindow -PassThru -Wait;
    }
    else {
        $process = Start-Process -FilePath $folderBin -NoNewWindow -PassThru -Wait;
    }
    
    $restart = (Test-Path env:PRYSM_AUTORESTART) -and $env:PRYSM_AUTORESTART -eq $TRUE -and $process.ExitCode -ne 0;
} while ($restart)