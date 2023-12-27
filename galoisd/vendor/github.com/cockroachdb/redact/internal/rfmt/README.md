Overall code structure
======================

This directory imports the "printf" logic as-is from the Go standard
library and instruments it for redaction of sensitive data.

Overall, the original Go code is structured as follows:

- the top-level API functions (`Printf` `Fprintf` etc) instantiate a
  "printer" struct called `pp`, then call the `doPrint*()` methods
  on it.

- the `pp` contains a byte slice (`pp.buf`, type `buffer`) that
  accumulates the result of formatting *regardless of the final output
  of the API* - i.e. a buffer is used even when using `Fprint` to an
  `io.Writer`. Only after the `doPrint()` method finishes, is the
  `buffer` copied to the final `io.Writer` in the `Fprint*` variants.

- each of the `doPrint` methods does some analysis on the argument
  list - quite simple for e.g. `doPrintln`, more intricate for
  `doPrintf`.  As part of the analysis it emits "spacing" or no-op
  bytes directly on the `pp.buf`. For example `doPrint` emits spaces
  between arguments directly, `doPrintf` emits the non-formatting
  characters from the format string, as well as certain *constant*
  error strings (e.g. `%(BADPREC)`).

Refreshing the sources
======================

The files in this directory have been imported from

`$GOROOT/src/fmt/{format,print}.go`

and

`$GOROOT/src/internal/fmtsort`

And patched using the included `.diff` files.

To upgrade to a newer Go implementation, import the files anew and
re-apply the patches.

See the script `refresh.sh` for details.
