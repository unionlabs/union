# Copyright 2022 Google LLC.
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     https://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

$OutputFolder = ".\build\bin\windows_amd64"
If (Test-Path $OutputFolder) {
    # Remove existing binaries
    Remove-Item -Path ".\build\bin\windows_amd64\*"
} else {
    # Create the folder to hold the binaries
    New-Item -Path $OutputFolder -ItemType Directory -Force
}

# Build the signer binary
Set-Location .\internal\signer\windows
go build
Move-Item .\windows.exe ..\..\..\build\bin\windows_amd64\ecp.exe
Set-Location ..\..\..\

# Build the signer library
go build -buildmode=c-shared -o .\build\bin\windows_amd64\libecp.dll .\cshared\main.go
Remove-Item .\build\bin\windows_amd64\libecp.h
