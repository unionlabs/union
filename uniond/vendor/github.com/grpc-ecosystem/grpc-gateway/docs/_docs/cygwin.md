---
category: documentation
title: Installation for Cygwin
order: 1000
---

# Installation for Cygwin

![cygwin-logo](https://upload.wikimedia.org/wikipedia/commons/thumb/2/29/Cygwin_logo.svg/145px-Cygwin_logo.svg.png)

## Installation

First you need to install the [Go language](https://golang.org/dl/). Please install the latest version, not the one that is listed here.

    wget -N https://storage.googleapis.com/golang/go1.8.1.windows-amd64.msi
    msiexec /i go1.8.1.windows-amd64.msi /passive /promptrestart

Then you need to install [ProtocolBuffers 3.0.0-beta-3](https://github.com/google/protobuf/releases) or later. Use the Windows release as no native Cygwin protoc with version 3 is available yet.

    wget -N https://github.com/google/protobuf/releases/download/v3.2.0/protoc-3.2.0-win32.zip`
    7z x protoc-3.2.0-win32.zip -o/usr/local/

Then you need to setup your Go workspace. Create the workspace dir.

    mkdir /home/user/go
    mkdir /home/user/go/bin
    mkdir /home/user/go/pkg
    mkdir /home/user/go/src

From an elevated cmd.exe prompt set the GOPATH variable in Windows and add the `$GOPATH/bin` directory to your path using `reg add` instead of `setx` because [setx can truncate your PATH variable to 1024 characters](https://encrypted.google.com/search?hl=en&q=setx%20truncates%20PATH%201024#safe=off&hl=en&q=setx+truncated+PATH+1024).

    setx GOPATH c:\path\to\your\cygwin\home\user\go /M
    set pathkey="HKEY_LOCAL_MACHINE\System\CurrentControlSet\Control\Session Manager\Environment"
    for /F "usebackq skip=2 tokens=2*" %A IN (`reg query %pathkey% /v Path`) do (reg add %pathkey% /f /v Path /t REG_SZ /d "%B;c:\path\to\your\cygwin\home\user\go\bin")

Then `go get -u -v` the following packages:

    go get -u -v github.com/grpc-ecosystem/grpc-gateway/protoc-gen-grpc-gateway
    go get -u -v github.com/grpc-ecosystem/grpc-gateway/protoc-gen-swagger
    go get -u -v github.com/golang/protobuf/protoc-gen-go

This will probably fail with a similar output to this:

    github.com/grpc-ecosystem/grpc-gateway (download)
    # cd .; git clone https://github.com/grpc-ecosystem/grpc-gateway C:\path\to\your\cygwin\home\user\go\src\github.com\grpc-ecosystem\grpc-gateway
    Cloning into 'C:\path\to\your\cygwin\home\user\go\src\github.com\grpc-ecosystem\grpc-gateway'...
    fatal: Invalid path '/home/user/go/C:\path\to\your\cygwin\home\user\go\src\github.com\grpc-ecosystem\grpc-gateway': No such file or directory
    package github.com/grpc-ecosystem/grpc-gateway/protoc-gen-grpc-gateway: exit status 128

To fix this you need to run the `go get -u -v` commands and look for all lines starting with `# cd .;`.
Copy and paste these lines into your shell and change the clone destination directories.

    git clone https://github.com/grpc-ecosystem/grpc-gateway $(cygpath -u $GOPATH)/src/github.com/grpc-ecosystem/grpc-gateway
    git clone https://github.com/golang/glog $(cygpath -u $GOPATH)/src/github.com/golang/glog
    git clone https://github.com/golang/protobuf $(cygpath -u $GOPATH)/src/github.com/golang/protobuf
    git clone https://github.com/google/go-genproto $(cygpath -u $GOPATH)/src/google.golang.org/genproto

Once the clone operations are finished the `go get -u -v` commands shouldn't give you an error anymore.

## Usage

Follow the [instructions](https://github.com/grpc-ecosystem/grpc-gateway#usage) in the [README](https://github.com/grpc-ecosystem/grpc-gateway#readme).

Adjust steps 3, 5 and 7 like this. protoc expects native Windows paths.

    protoc -I. -I$(cygpath -w /usr/local/include) -I${GOPATH}/src -I${GOPATH}/src/github.com/grpc-ecosystem/grpc-gateway/third_party/googleapis --go_out=plugins=grpc:. ./path/to/your_service.proto
    protoc -I. -I$(cygpath -w /usr/local/include) -I${GOPATH}/src -I${GOPATH}/src/github.com/grpc-ecosystem/grpc-gateway/third_party/googleapis --grpc-gateway_out=logtostderr=true:. ./path/to/your_service.proto
    protoc -I. -I$(cygpath -w /usr/local/include) -I${GOPATH}/src -I${GOPATH}/src/github.com/grpc-ecosystem/grpc-gateway/third_party/googleapis --swagger_out=logtostderr=true:. ./path/to/your_service.proto

Then `cd` into the directory where your entry-point `main.go` file is located and run:

    go get -v

This will fail in this same way as it did during the installation. Look for all lines starting with `# cd .;`. Copy and paste these lines into your shell and change the clone destination directories.

    git clone https://go.googlesource.com/net $(cygpath -u $GOPATH)/src/golang.org/x/net
    git clone https://go.googlesource.com/text $(cygpath -u $GOPATH)/src/golang.org/x/text
    git clone https://github.com/grpc/grpc-go $(cygpath -u $GOPATH)/src/google.golang.org/grpc

Once the clone operations are finished the `go get -v` commands shouldn't give you an error anymore.

Then run:

    go install

This will compile and install your grpc-gateway service into `$GOPATH/bin`.
