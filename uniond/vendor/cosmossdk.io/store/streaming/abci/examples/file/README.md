# File Plugin

The file plugin is an example plugin written in Go. It is intended for local testing and should not be used in production environments.

## Build

To build the plugin run the following command:

```shell
cd store
```

```shell
go build -o streaming/abci/examples/file/file streaming/abci/examples/file/file.go
```

* The plugin will write files to the users home directory `~/`.
