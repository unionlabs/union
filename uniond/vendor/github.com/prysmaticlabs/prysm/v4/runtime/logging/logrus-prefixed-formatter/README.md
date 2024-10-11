# Logrus Prefixed Log Formatter

Originally from [github.com/x-cray/logrus-prefixed-formatter(https://github.com/x-cray/logrus-prefixed-formatter)

[Logrus](https://github.com/sirupsen/logrus) formatter mainly based on original `logrus.TextFormatter` but with slightly
modified colored output and support for log entry prefixes, e.g. message source followed by a colon. In addition, custom
color themes are supported.

![Formatter screenshot](http://cl.ly/image/1w0B3F233F3z/formatter-screenshot@2x.png)

Just like with the original `logrus.TextFormatter` when a TTY is not attached, the output is compatible with the
[logfmt](http://godoc.org/github.com/kr/logfmt) format:

```text
time="Oct 27 00:44:26" level=debug msg="Started observing beach" animal=walrus number=8
time="Oct 27 00:44:26" level=info msg="A group of walrus emerges from the ocean" animal=walrus size=10
time="Oct 27 00:44:26" level=warning msg="The group's number increased tremendously!" number=122 omg=true
time="Oct 27 00:44:26" level=debug msg="Temperature changes" temperature=-4
time="Oct 27 00:44:26" level=panic msg="It's over 9000!" animal=orca size=9009
time="Oct 27 00:44:26" level=fatal msg="The ice breaks!" number=100 omg=true
exit status 1
```

## Installation
To install formatter, use `go get`:

```sh
$ go get github.com/prysmaticlabs/prysm/runtime/logging/logrus-prefixed-formatter
```

## Usage
Here is how it should be used:

```go
package main

import (
	"github.com/sirupsen/logrus"
	prefixed "github.com/prysmaticlabs/prysm/runtime/logging/logrus-prefixed-formatter"
)

var log = logrus.New()

func init() {
	log.Formatter = new(prefixed.TextFormatter)
	log.Level = logrus.DebugLevel
}

func main() {
	log.WithFields(logrus.Fields{
		"prefix": "main",
		"animal": "walrus",
		"number": 8,
	}).Debug("Started observing beach")

	log.WithFields(logrus.Fields{
		"prefix":      "sensor",
		"temperature": -4,
	}).Info("Temperature changes")
}
```

## API
`prefixed.TextFormatter` exposes the following fields and methods.

### Fields

* `ForceColors bool` — set to true to bypass checking for a TTY before outputting colors.
* `DisableColors bool` — force disabling colors. For a TTY colors are enabled by default.
* `DisableUppercase bool` — set to true to turn off the conversion of the log level names to uppercase.
* `ForceFormatting bool` — force formatted layout, even for non-TTY output.
* `DisableTimestamp bool` — disable timestamp logging. Useful when output is redirected to logging system that already adds timestamps.
* `FullTimestamp bool` — enable logging the full timestamp when a TTY is attached instead of just the time passed since beginning of execution.
* `TimestampFormat string` — timestamp format to use for display when a full timestamp is printed.
* `DisableSorting bool` — the fields are sorted by default for a consistent output. For applications that log extremely frequently and don't use the JSON formatter this may not be desired.
* `QuoteEmptyFields bool` — wrap empty fields in quotes if true.
* `QuoteCharacter string` — can be set to the override the default quoting character `"` with something else. For example: `'`, or `` ` ``.
* `SpacePadding int` — pad msg field with spaces on the right for display. The value for this parameter will be the size of padding. Its default value is zero, which means no padding will be applied.

### Methods

#### `SetColorScheme(colorScheme *prefixed.ColorScheme)`

Sets an alternative color scheme for colored output. `prefixed.ColorScheme` struct supports the following fields:
* `InfoLevelStyle string` — info level style.
* `WarnLevelStyle string` — warn level style.
* `ErrorLevelStyle string` — error style.
* `FatalLevelStyle string` — fatal level style.
* `PanicLevelStyle string` — panic level style.
* `DebugLevelStyle string` — debug level style.
* `PrefixStyle string` — prefix style.
* `TimestampStyle string` — timestamp style.

Color styles should be specified using [mgutz/ansi](https://github.com/mgutz/ansi#style-format) style syntax. For example, here is the default theme:

```go
InfoLevelStyle:  "green",
WarnLevelStyle:  "yellow",
ErrorLevelStyle: "red",
FatalLevelStyle: "red",
PanicLevelStyle: "red",
DebugLevelStyle: "blue",
PrefixStyle:     "cyan",
TimestampStyle:  "black+h"
```

It's not necessary to specify all colors when changing color scheme if you want to change just specific ones:

```go
formatter.SetColorScheme(&prefixed.ColorScheme{
    PrefixStyle:    "blue+b",
    TimestampStyle: "white+h",
})
```

# License
MIT
