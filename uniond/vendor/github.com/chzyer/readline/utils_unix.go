// +build aix darwin dragonfly freebsd linux,!appengine netbsd openbsd os400 solaris

package readline

import (
	"io"
	"os"
	"os/signal"
	"sync"
	"syscall"
)

type winsize struct {
	Row    uint16
	Col    uint16
	Xpixel uint16
	Ypixel uint16
}

// SuspendMe use to send suspend signal to myself, when we in the raw mode.
// For OSX it need to send to parent's pid
// For Linux it need to send to myself
func SuspendMe() {
	p, _ := os.FindProcess(os.Getppid())
	p.Signal(syscall.SIGTSTP)
	p, _ = os.FindProcess(os.Getpid())
	p.Signal(syscall.SIGTSTP)
}

// get width of the terminal
func getWidth(stdoutFd int) int {
	cols, _, err := GetSize(stdoutFd)
	if err != nil {
		return -1
	}
	return cols
}

func GetScreenWidth() int {
	w := getWidth(syscall.Stdout)
	if w < 0 {
		w = getWidth(syscall.Stderr)
	}
	return w
}

// ClearScreen clears the console screen
func ClearScreen(w io.Writer) (int, error) {
	return w.Write([]byte("\033[H"))
}

func DefaultIsTerminal() bool {
	return IsTerminal(syscall.Stdin) && (IsTerminal(syscall.Stdout) || IsTerminal(syscall.Stderr))
}

func GetStdin() int {
	return syscall.Stdin
}

// -----------------------------------------------------------------------------

var (
	widthChange         sync.Once
	widthChangeCallback func()
)

func DefaultOnWidthChanged(f func()) {
	widthChangeCallback = f
	widthChange.Do(func() {
		ch := make(chan os.Signal, 1)
		signal.Notify(ch, syscall.SIGWINCH)

		go func() {
			for {
				_, ok := <-ch
				if !ok {
					break
				}
				widthChangeCallback()
			}
		}()
	})
}
