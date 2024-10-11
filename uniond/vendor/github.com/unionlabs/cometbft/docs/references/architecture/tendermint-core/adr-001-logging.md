# ADR 1: Logging

## Context

Current logging system in Tendermint is very static and not flexible enough.

Issues: [358](https://github.com/tendermint/tendermint/issues/358), [375](https://github.com/tendermint/tendermint/issues/375).

What we want from the new system:

- per package dynamic log levels
- dynamic logger setting (logger tied to the processing struct)
- conventions
- be more visually appealing

"dynamic" here means the ability to set smth in runtime.

## Decision

### 1) An interface

First, we will need an interface for all of our libraries (`tmlibs`, Tendermint, etc.). My personal preference is go-kit `Logger` interface (see Appendix A.), but that is too much a bigger change. Plus we will still need levels.

```go
# log.go
type Logger interface {
    Debug(msg string, keyvals ...interface{}) error
    Info(msg string, keyvals ...interface{}) error
    Error(msg string, keyvals ...interface{}) error

	  With(keyvals ...interface{}) Logger
}
```

On a side note: difference between `Info` and `Notice` is subtle. We probably
could do without `Notice`. Don't think we need `Panic` or `Fatal` as a part of
the interface. These funcs could be implemented as helpers. In fact, we already
have some in `tmlibs/common`.

- `Debug` - extended output for devs
- `Info` - all that is useful for a user
- `Error` - errors

`Notice` should become `Info`, `Warn` either `Error` or `Debug` depending on the message, `Crit` -> `Error`.

This interface should go into `tmlibs/log`. All libraries which are part of the core (tendermint/tendermint) should obey it.

### 2) Logger with our current formatting

On top of this interface, we will need to implement a stdout logger, which will be used when Tendermint is configured to output logs to STDOUT.

Many people say that they like the current output, so let's stick with it.

```
NOTE[2017-04-25|14:45:08] ABCI Replay Blocks                       module=consensus appHeight=0 storeHeight=0 stateHeight=0
```

Couple of minor changes:

```
I[2017-04-25|14:45:08.322] ABCI Replay Blocks            module=consensus appHeight=0 storeHeight=0 stateHeight=0
```

Notice the level is encoded using only one char plus milliseconds.

Note: there are many other formats out there like [logfmt](https://brandur.org/logfmt).

This logger could be implemented using any logger - [logrus](https://github.com/sirupsen/logrus), [go-kit/log](https://github.com/go-kit/kit/tree/master/log), [zap](https://github.com/uber-go/zap), log15 so far as it

a) supports coloring output<br>
b) is moderately fast (buffering) <br>
c) conforms to the new interface or adapter could be written for it <br>
d) is somewhat configurable<br>

go-kit is my favorite so far. Check out how easy it is to color errors in red https://github.com/go-kit/kit/blob/master/log/term/example_test.go#L12. Although, coloring could only be applied to the whole string :(

```
go-kit +: flexible, modular
go-kit “-”: logfmt format https://brandur.org/logfmt

logrus +: popular, feature rich (hooks), API and output is more like what we want
logrus -: not so flexible
```

```go
# tm_logger.go
// NewTmLogger returns a logger that encodes keyvals to the Writer in
// tm format.
func NewTmLogger(w io.Writer) Logger {
  return &tmLogger{kitlog.NewLogfmtLogger(w)}
}

func (l tmLogger) SetLevel(level string() {
  switch (level) {
    case "debug":
      l.sourceLogger = level.NewFilter(l.sourceLogger, level.AllowDebug())
  }
}

func (l tmLogger) Info(msg string, keyvals ...interface{}) error {
  l.sourceLogger.Log("msg", msg, keyvals...)
}

# log.go
func With(logger Logger, keyvals ...interface{}) Logger {
  kitlog.With(logger.sourceLogger, keyvals...)
}
```

Usage:

```go
logger := log.NewTmLogger(os.Stdout)
logger.SetLevel(config.GetString("log_level"))
node.SetLogger(log.With(logger, "node", Name))
```

**Other log formatters**

In the future, we may want other formatters like JSONFormatter.

```
{ "level": "notice", "time": "2017-04-25 14:45:08.562471297 -0400 EDT", "module": "consensus", "msg": "ABCI Replay Blocks", "appHeight": 0, "storeHeight": 0, "stateHeight": 0 }
```

### 3) Dynamic logger setting

https://dave.cheney.net/2017/01/23/the-package-level-logger-anti-pattern

This is the hardest part and where the most work will be done. logger should be tied to the processing struct, or the context if it adds some fields to the logger.

```go
type BaseService struct {
    log     log15.Logger
    name    string
    started uint32 // atomic
    stopped uint32 // atomic
...
}
```

BaseService already contains `log` field, so most of the structs embedding it should be fine. We should rename it to `logger`.

The only thing missing is the ability to set logger:

```
func (bs *BaseService) SetLogger(l log.Logger) {
  bs.logger = l
}
```

### 4) Conventions

Important keyvals should go first. Example:

```
correct
I[2017-04-25|14:45:08.322] ABCI Replay Blocks                       module=consensus instance=1 appHeight=0 storeHeight=0 stateHeight=0
```

not

```
wrong
I[2017-04-25|14:45:08.322] ABCI Replay Blocks                       module=consensus appHeight=0 storeHeight=0 stateHeight=0 instance=1
```

for that in most cases you'll need to add `instance` field to a logger upon creating, not when u log a particular message:

```go
colorFn := func(keyvals ...interface{}) term.FgBgColor {
		for i := 1; i < len(keyvals); i += 2 {
			if keyvals[i] == "instance" && keyvals[i+1] == "1" {
				return term.FgBgColor{Fg: term.Blue}
			} else if keyvals[i] == "instance" && keyvals[i+1] == "1" {
				return term.FgBgColor{Fg: term.Red}
			}
		}
		return term.FgBgColor{}
	}
logger := term.NewLogger(os.Stdout, log.NewTmLogger, colorFn)

c1 := NewConsensusReactor(...)
c1.SetLogger(log.With(logger, "instance", 1))

c2 := NewConsensusReactor(...)
c2.SetLogger(log.With(logger, "instance", 2))
```

## Status

Implemented

## Consequences

### Positive

Dynamic logger, which could be turned off for some modules at runtime. Public interface for other projects using Tendermint libraries.

### Negative

We may loose the ability to color keys in keyvalue pairs. go-kit allow you to easily change foreground / background colors of the whole string, but not its parts.

### Neutral

## Appendix A.

I really like a minimalistic approach go-kit took with his logger https://github.com/go-kit/kit/tree/master/log:

```
type Logger interface {
    Log(keyvals ...interface{}) error
}
```

See [The Hunt for a Logger Interface](https://web.archive.org/web/20210902161539/https://go-talks.appspot.com/github.com/ChrisHines/talks/structured-logging/structured-logging.slide#1). The advantage is greater composability (check out how go-kit defines colored logging or log-leveled logging on top of this interface https://github.com/go-kit/kit/tree/master/log).
