#!/bin/bash

mkdir binaries
go build -o ./binaries/client ../../../../interop/xds/client/
go build -o ./binaries/server ../../../../interop/xds/server/
go test . 
