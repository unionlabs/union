# Remote Debugging with go-delve

[Delve](https://github.com/go-delve/delve) is a debugger for the Go programming language. The goal of the project is to provide a simple, full featured debugging tool for Go. Delve should be easy to invoke and easy to use. Chances are if you're using a debugger, things aren't going your way. With that in mind, Delve should stay out of your way as much as possible.

## Use-case

Cosmos-SDK provides you with a local network to bootstrap a chain in your machine, but how does one debug a node or module?

If we start a single node, we won't be able to debug transactions as the machine will be in bootstrapping phase trying to find peers to connect too, that's why we need to start a local network.

But the current `localnet-start` does not provide us with debugging tools so that's why there is a different image for debugging a local network, that is to avoid any issues in the future were debugging won't be needed.

Both `simd-env` and `simd-dlv` work and run the same, except that `simd-dlv` uses `go-delve` to run the binaries.

## How to use

The command to start a local network in debug mode is:

```shell
# make localnet-debug
```

The command to stop the local network and destroy its containers is:

```shell
# make localnet-stop
```

__note: this works the same for both `localnet-start` and `localnet-debug`__

Now, by default only `simdnode0` is run in debug mode, but you can run any of the other nodes in debug mode by changing the `DEBUG` environment variable to `1` in `docker-compose.yml`.

## How to connect

Delve will open a port on `2345` for `simdnode0` and it will increment for each of the other nodes that have `DEBUG` set to `1`.

You can connect to the debugging server either through [GoLand IDE](https://www.jetbrains.com/help/go/attach-to-running-go-processes-with-debugger.html)  or you can use  [delve cli](https://github.com/go-delve/delve/blob/master/Documentation/usage/dlv_connect.md) command.
