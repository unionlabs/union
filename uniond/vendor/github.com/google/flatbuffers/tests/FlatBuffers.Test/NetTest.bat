@echo off
@REM Builds a .NET solution file, adds the project, builds it
@REM and executes it. Cleans up all generated files and directories.

set TEMP_BIN=.tmp

@REM Run the .NET tests
set CORE_FILE=FlatBuffers.Test
set CORE_PROJ_FILE=%CORE_FILE%.csproj
set CORE_SLN_FILE=%CORE_FILE%.sln
dotnet new sln --force --name %CORE_FILE%
dotnet sln %CORE_SLN_FILE% add %CORE_PROJ_FILE%
dotnet build -c Release -o %TEMP_BIN% -v quiet %CORE_PROJ_FILE%
%TEMP_BIN%\%CORE_FILE%.exe
del /f %CORE_SLN_FILE%

@REM TODO(dbaileychess): Support the other configurations in NetTest.sh

@REM remove the temp bin directory, with files (/S) and quietly (/Q)
RD /S /Q %TEMP_BIN%
