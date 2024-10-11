@echo off

SetLocal EnableDelayedExpansion & REM All variables are set local to this run & expanded at execution time rather than at parse time (tip: echo !output!)

set PRYLABS_SIGNING_KEY=0AE0051D647BA3C1A917AF4072E33E4DF1A5036E

REM Complain if invalid arguments were provided.
for %%a in (beacon-chain validator client-stats) do (
    if %1 equ %%a (
        goto validprocess
    )
)
echo [31mERROR: PROCESS missing or invalid[0m
echo Usage: ./prysm.bat PROCESS FLAGS.
echo.
echo PROCESS can be beacon-chain, validator, or client-stats.
echo FLAGS are the flags or arguments passed to the PROCESS.
echo. 
echo Use this script to download the latest Prysm release binaries.
echo Downloaded binaries are saved to .\dist
echo. 
echo To specify a specific release version:
echo  set USE_PRYSM_VERSION=v1.0.0-alpha3
echo  to resume using the latest release:
echo   set USE_PRYSM_VERSION=
echo.
echo To specify a non-portable version of BLST:
echo  set USE_PRYSM_MODERN=1
echo  to resume using the portable version:
echo  unset USE_PRYSM_MODERN=
echo. 
echo To automatically restart crashed processes:
echo  set PRYSM_AUTORESTART=true^& .\prysm.bat beacon-chain
echo  to stop autorestart run:
echo   set PRYSM_AUTORESTART=
echo. 
exit /B 1
:validprocess

REM Get full path to prysm.bat file (excluding filename)
set wrapper_dir=%~dp1dist
reg Query "HKLM\Hardware\Description\System\CentralProcessor\0" | find /i "x86" > NUL && set WinOS=32BIT || set WinOS=64BIT
if %WinOS%==32BIT (
    echo [31mERROR: prysm is only supported on 64-bit Operating Systems [0m
    exit /b 1
)
if %WinOS%==64BIT (
    set arch=amd64.exe
    set system=windows
)

mkdir %wrapper_dir%

REM get_prysm_version - Find the latest Prysm version available for download.
(for /f %%i in ('curl -f -s https://prysmaticlabs.com/releases/latest') do set prysm_version=%%i) || (echo [31mERROR: Starting prysm requires an internet connection. If you are being blocked by your antivirus, you can download the beacon chain and validator executables from our releases page on Github here https://github.com/prysmaticlabs/prysm/releases/ [0m && exit /b 1)
echo [37mLatest prysm release is %prysm_version%.[0m
IF defined USE_PRYSM_VERSION (
    echo [33mdetected variable USE_PRYSM_VERSION=%USE_PRYSM_VERSION%[0m
    set reason=as specified in USE_PRYSM_VERSION
    set prysm_version=%USE_PRYSM_VERSION%
) else (
    set reason=automatically selected latest available release
)
echo Using prysm version %prysm_version%.

IF defined USE_PRYSM_MODERN (
	set BEACON_CHAIN_REAL=%wrapper_dir%\beacon-chain-%prysm_version%-modern-%system%-%arch%
) else (
	set BEACON_CHAIN_REAL=%wrapper_dir%\beacon-chain-%prysm_version%-%system%-%arch%
)
set VALIDATOR_REAL=%wrapper_dir%\validator-%prysm_version%-%system%-%arch%
set CLIENT_STATS_REAL=%wrapper_dir%\client-stats-%prysm_version%-%system%-%arch%

if "%~1"=="beacon-chain" (
    if exist "%BEACON_CHAIN_REAL%" (
        echo [32mBeacon chain is up to date.[0m
    ) else (
        echo [35mDownloading beacon chain %prysm_version% to %BEACON_CHAIN_REAL% %reason%[0m
		if defined USE_PRYSM_MODERN (
			for /f "delims=" %%i in ('curl --silent -o nul -w "%%{http_code}" https://prysmaticlabs.com/releases/beacon-chain-%prysm_version%-modern-%system%-%arch% ') do set "http=%%i" && echo %%i
		) else (
			for /f "delims=" %%i in ('curl --silent -o nul -w "%%{http_code}" https://prysmaticlabs.com/releases/beacon-chain-%prysm_version%-%system%-%arch% ') do set "http=%%i" && echo %%i
		)
		if "!http!"=="404" (
			echo [35mNo prysm beacon chain found for %prysm_version%[0m
			exit /b 1
		)	
		if defined USE_PRYSM_MODERN (
			curl -L https://prysmaticlabs.com/releases/beacon-chain-%prysm_version%-modern-%system%-%arch% -o %BEACON_CHAIN_REAL%
			curl --silent -L https://prysmaticlabs.com/releases/beacon-chain-%prysm_version%-modern-%system%-%arch%.sha256 -o %wrapper_dir%\beacon-chain-%prysm_version%-modern-%system%-%arch%.sha256
			curl --silent -L https://prysmaticlabs.com/releases/beacon-chain-%prysm_version%-modern-%system%-%arch%.sig -o %wrapper_dir%\beacon-chain-%prysm_version%-modern-%system%-%arch%.sig
		) else (

			curl -L https://prysmaticlabs.com/releases/beacon-chain-%prysm_version%-%system%-%arch% -o %BEACON_CHAIN_REAL%
			curl --silent -L https://prysmaticlabs.com/releases/beacon-chain-%prysm_version%-%system%-%arch%.sha256 -o %wrapper_dir%\beacon-chain-%prysm_version%-%system%-%arch%.sha256
			curl --silent -L https://prysmaticlabs.com/releases/beacon-chain-%prysm_version%-%system%-%arch%.sig -o %wrapper_dir%\beacon-chain-%prysm_version%-%system%-%arch%.sig
		)
    )
)

if "%~1"=="validator" (
    if exist "%VALIDATOR_REAL%" (
        echo [32mValidator is up to date.[0m
    ) else (
        echo [35mDownloading validator %prysm_version% to %VALIDATOR_REAL% %reason%[0m
		for /f "delims=" %%i in ('curl --silent -o nul -w "%%{http_code}" https://prysmaticlabs.com/releases/validator-%prysm_version%-%system%-%arch% ') do set "http=%%i" && echo %%i
		if "!http!"=="404"  (
			echo [35mNo prysm validator found for %prysm_version%[0m
			exit /b 1
		)
		curl -L https://prysmaticlabs.com/releases/validator-%prysm_version%-%system%-%arch% -o %VALIDATOR_REAL%
        curl --silent -L https://prysmaticlabs.com/releases/validator-%prysm_version%-%system%-%arch%.sha256 -o %wrapper_dir%\validator-%prysm_version%-%system%-%arch%.sha256
        curl --silent -L https://prysmaticlabs.com/releases/validator-%prysm_version%-%system%-%arch%.sig -o %wrapper_dir%\validator-%prysm_version%-%system%-%arch%.sig
    )
)

if "%~1"=="client-stats" (
    if exist %CLIENT_STATS_REAL% (
        echo [32mClient-stats is up to date.[0m
    ) else (
        echo [35mDownloading client-stats %prysm_version% to %CLIENT_STATS_REAL% %reason%[0m
		for /f "delims=" %%i in ('curl --silent -o nul -w "%%{http_code}" https://prysmaticlabs.com/releases/client-stats-%prysm_version%-%system%-%arch% ') do set "http=%%i" && echo %%i
		if "!http!"=="404" (
			echo [35mNo prysm client stats found for %prysm_version%[0m
			exit /b 1
		)
		curl -L https://prysmaticlabs.com/releases/client-stats-%prysm_version%-%system%-%arch% -o %CLIENT_STATS_REAL%
        curl --silent -L https://prysmaticlabs.com/releases/client-stats-%prysm_version%-%system%-%arch%.sha256 -o %wrapper_dir%\client-stats-%prysm_version%-%system%-%arch%.sha256
        curl --silent -L https://prysmaticlabs.com/releases/client-stats-%prysm_version%-%system%-%arch%.sig -o %wrapper_dir%\client-stats-%prysm_version%-%system%-%arch%.sig
    )
)

if "%~1"=="slasher" (
    echo [31mThe slasher binary is no longer available. Please use the --slasher flag with your beacon node. See: https://docs.prylabs.network/docs/prysm-usage/slasher/[0m
    exit /b 1
)

if "%~1"=="beacon-chain" ( set process=%BEACON_CHAIN_REAL%)
if "%~1"=="validator" ( set process=%VALIDATOR_REAL%) 
if "%~1"=="client-stats" ( set process=%CLIENT_STATS_REAL%)

REM GPG not natively available on Windows, external module required
echo [33mWARN GPG verification is not natively available on Windows.[0m
echo [33mWARN Skipping integrity verification of downloaded binary[0m
REM Check SHA256 File Hash before running
echo [37mVerifying binary authenticity with SHA256 Hash.[0m
for /f "delims=" %%A in ('certutil -hashfile %process% SHA256 ^| find /v "hash"') do (
    set SHA256Hash=%%A
)
set /p ExpectedSHA256=<%process%.sha256
if "%ExpectedSHA256:~0,64%"=="%SHA256Hash%" (
    echo [32mSHA256 Hash Match![0m
) else if "%PRYSM_ALLOW_UNVERIFIED_BINARIES%"=="1" (
    echo [31mWARNING Failed to verify Prysm binary.[0m 
    echo Detected PRYSM_ALLOW_UNVERIFIED_BINARIES=1
    echo Proceeding...
) else (
    echo [31mERROR Failed to verify Prysm binary. Please erase downloads in the
    echo dist directory and run this script again. Alternatively, you can use a
    echo A prior version by specifying environment variable USE_PRYSM_VERSION
    echo with the specific version, as desired. Example: set USE_PRYSM_VERSION=v1.0.0-alpha.5
    echo If you must wish to continue running an unverified binary, use:
    echo set PRYSM_ALLOW_UNVERIFIED_BINARIES=1[0m
    exit /b 1
)

set processargs=%*
echo Starting Prysm %processargs%

set "processargs=!processargs:*%1=!" & REM remove process from the list of arguments

:autorestart
    %process% %processargs% 
    if ERRORLEVEL 1 goto :ERROR
    REM process terminated gracefully
    pause
    exit /b 0

:ERROR
    Echo [91mERROR FAILED[0m
    IF defined PRYSM_AUTORESTART (
        echo PRYSM_autorestart is set, restarting
        GOTO autorestart
    ) else (
        echo an error has occurred, set PRYSM_AUTORESTART=1 to automatically restart
    )

:end
REM Variables are set local to this run
EndLocal
