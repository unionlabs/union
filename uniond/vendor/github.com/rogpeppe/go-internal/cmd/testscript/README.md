```
The testscript command runs github.com/rogpeppe/go-internal/testscript scripts
in a fresh temporary work directory tree.

Usage:
    testscript [-v] [-e VAR[=value]]... [-u] [-work] files...

The testscript command is designed to make it easy to create self-contained
reproductions of command sequences.

Each file is opened as a script and run as described in the documentation for
github.com/rogpeppe/go-internal/testscript. The special filename "-" is
interpreted as the standard input.

As a special case, supporting files/directories in the .gomodproxy subdirectory
will be served via a github.com/rogpeppe/go-internal/goproxytest server which
is available to each script via the GOPROXY environment variable. The contents
of the .gomodproxy subdirectory are not available to the script except via the
proxy server. See the documentation for
github.com/rogpeppe/go-internal/goproxytest for details on the format of these
files/directories.

Environment variables can be passed through to each script with the -e flag,
where VAR is the name of the variable. Variables override testscript-defined
values, with the exception of WORK which cannot be overridden. The -e flag can
appear multiple times to specify multiple variables.

The -u flag specifies that if a cmp command within a testscript fails and its
second argument refers to a file inside the testscript file, the command will
succeed and the testscript file will be updated to reflect the actual content.
As such, this is the cmd/testcript equivalent of
testscript.Params.UpdateScripts.

The -work flag prints the temporary work directory path before running each
script, and does not remove that directory when testscript exits.

Examples
========

The following example, fruit.txtar, shows a simple reproduction that includes
.gomodproxy supporting files:

    go get -m fruit.com
    go list fruit.com/...
    stdout 'fruit.com/fruit'

    -- go.mod --
    module mod

    -- .gomodproxy/fruit.com_v1.0.0/.mod --
    module fruit.com

    -- .gomodproxy/fruit.com_v1.0.0/.info --
    {"Version":"v1.0.0","Time":"2018-10-22T18:45:39Z"}

    -- .gomodproxy/fruit.com_v1.0.0/fruit/fruit.go --
    package fruit

    const Name = "Apple"

Running testscript -v fruit.txtar we get:

    ...
    > go get -m fruit.com
    [stderr]
    go: finding fruit.com v1.0.0

    > go list fruit.com/...
    [stdout]
    fruit.com/fruit

    [stderr]
    go: downloading fruit.com v1.0.0

    > stdout 'fruit.com/fruit'
    PASS


The following example, goimports.txtar, shows a simple reproduction involving
goimports:

    go install golang.org/x/tools/cmd/goimports

    # check goimports help information
    exec goimports -d main.go
    stdout 'import "math"'

    -- go.mod --
    module mod

    require golang.org/x/tools v0.0.0-20181221235234-d00ac6d27372

    -- main.go --
    package mod

    const Pi = math.Pi

Running testscript -v goimports.txtar we get:

    ...
    > go install golang.org/x/tools/cmd/goimports
    [stderr]
    go: finding golang.org/x/tools v0.0.0-20181221235234-d00ac6d27372
    go: downloading golang.org/x/tools v0.0.0-20181221235234-d00ac6d27372

    # check goimports help information (0.015s)
    > exec goimports -d main.go
    [stdout]
    diff -u main.go.orig main.go
    --- main.go.orig        2019-01-08 16:03:35.861907738 +0000
    +++ main.go     2019-01-08 16:03:35.861907738 +0000
    @@ -1,3 +1,5 @@
     package mod

    +import "math"
    +
     const Pi = math.Pi
    > stdout 'import "math"'
    PASS
```
