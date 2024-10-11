# .NET Tests

## Running on Linux

### Prerequisites
To run the tests on a Linux a few prerequisites are needed:

1) mono
2) msbuild

### Running

To run the tests:

```
./NetTest.sh
```

This will download the .NET installer and core SDK if those are not already
installed. Then it will build the tests using `msbuild` and run the resulting
test binary with `mono`.

After running the tests, the downloaded .NET installer and SDK are *not* removed
as they can be reused in subsequent invocations. The files are ignored by git by
default, and can remain in the working directory.

### Cleaning

If you want to clean up the downloaded .NET installer and SDK, run:

```
./clean.sh
```

This will wipe away the downloaded files and directories. Those will be
automatically re-downloaded when running `NetTest.sh`.


