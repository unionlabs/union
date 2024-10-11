#!/bin/sh

# Remove files and directory that are needed to build and run the .NET tests.
# The script NetTest.sh installs these as needed.

[ -d .dotnet_tmp ] && rm -rf .dotnet_tmp
[ -d packages ] && rm -rf packages
[ -d .tmp ] && rm -rf .tmp
[ -f nuget.exe ] && rm nuget.exe
[ -f dotnet-intall.sh ] && rm dotnet-install.sh
