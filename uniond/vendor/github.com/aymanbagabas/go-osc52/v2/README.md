
# go-osc52

<p>
    <a href="https://github.com/aymanbagabas/go-osc52/releases"><img src="https://img.shields.io/github/release/aymanbagabas/go-osc52.svg" alt="Latest Release"></a>
    <a href="https://pkg.go.dev/github.com/aymanbagabas/go-osc52/v2?tab=doc"><img src="https://godoc.org/github.com/golang/gddo?status.svg" alt="GoDoc"></a>
</p>

A Go library to work with the [ANSI OSC52](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Operating-System-Commands) terminal sequence.

## Usage

You can use this small library to construct an ANSI OSC52 sequence suitable for
your terminal.


### Example

```go
import (
  "os"
  "fmt"

  "github.com/aymanbagabas/go-osc52/v2"
)

func main() {
  s := "Hello World!"

  // Copy `s` to system clipboard
  osc52.New(s).WriteTo(os.Stderr)

  // Copy `s` to primary clipboard (X11)
  osc52.New(s).Primary().WriteTo(os.Stderr)

  // Query the clipboard
  osc52.Query().WriteTo(os.Stderr)

  // Clear system clipboard
  osc52.Clear().WriteTo(os.Stderr)

  // Use the fmt.Stringer interface to copy `s` to system clipboard
  fmt.Fprint(os.Stderr, osc52.New(s))

  // Or to primary clipboard
  fmt.Fprint(os.Stderr, osc52.New(s).Primary())
}
```

## SSH Example

You can use this over SSH using [gliderlabs/ssh](https://github.com/gliderlabs/ssh) for instance:

```go
var sshSession ssh.Session
seq := osc52.New("Hello awesome!")
// Check if term is screen or tmux
pty, _, _ := s.Pty()
if pty.Term == "screen" {
  seq = seq.Screen()
} else if isTmux {
  seq = seq.Tmux()
}
seq.WriteTo(sshSession.Stderr())
```

## Tmux

Make sure you have `set-clipboard on` in your config, otherwise, tmux won't
allow your application to access the clipboard [^1].

Using the tmux option, `osc52.TmuxMode` or `osc52.New(...).Tmux()`, wraps the
OSC52 sequence in a special tmux DCS sequence and pass it to the outer
terminal. This requires `allow-passthrough on` in your config.
`allow-passthrough` is no longer enabled by default
[since tmux 3.3a](https://github.com/tmux/tmux/issues/3218#issuecomment-1153089282) [^2].

[^1]: See [tmux clipboard](https://github.com/tmux/tmux/wiki/Clipboard)
[^2]: [What is allow-passthrough](https://github.com/tmux/tmux/wiki/FAQ#what-is-the-passthrough-escape-sequence-and-how-do-i-use-it)

## Credits

* [vim-oscyank](https://github.com/ojroques/vim-oscyank) this is heavily inspired by vim-oscyank.
