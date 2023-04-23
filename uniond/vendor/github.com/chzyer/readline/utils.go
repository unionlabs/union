package readline

import (
	"bufio"
	"bytes"
	"container/list"
	"fmt"
	"os"
	"os/signal"
	"strconv"
	"strings"
	"sync"
	"syscall"
	"time"
	"unicode"
)

var (
	isWindows = false
)

const (
	CharLineStart = 1
	CharBackward  = 2
	CharInterrupt = 3
	CharDelete    = 4
	CharLineEnd   = 5
	CharForward   = 6
	CharBell      = 7
	CharCtrlH     = 8
	CharTab       = 9
	CharCtrlJ     = 10
	CharKill      = 11
	CharCtrlL     = 12
	CharEnter     = 13
	CharNext      = 14
	CharPrev      = 16
	CharBckSearch = 18
	CharFwdSearch = 19
	CharTranspose = 20
	CharCtrlU     = 21
	CharCtrlW     = 23
	CharCtrlY     = 25
	CharCtrlZ     = 26
	CharEsc       = 27
	CharO         = 79
	CharEscapeEx  = 91
	CharBackspace = 127
)

const (
	MetaBackward rune = -iota - 1
	MetaForward
	MetaDelete
	MetaBackspace
	MetaTranspose
)

// WaitForResume need to call before current process got suspend.
// It will run a ticker until a long duration is occurs,
// which means this process is resumed.
func WaitForResume() chan struct{} {
	ch := make(chan struct{})
	var wg sync.WaitGroup
	wg.Add(1)
	go func() {
		ticker := time.NewTicker(10 * time.Millisecond)
		t := time.Now()
		wg.Done()
		for {
			now := <-ticker.C
			if now.Sub(t) > 100*time.Millisecond {
				break
			}
			t = now
		}
		ticker.Stop()
		ch <- struct{}{}
	}()
	wg.Wait()
	return ch
}

func Restore(fd int, state *State) error {
	err := restoreTerm(fd, state)
	if err != nil {
		// errno 0 means everything is ok :)
		if err.Error() == "errno 0" {
			return nil
		} else {
			return err
		}
	}
	return nil
}

func IsPrintable(key rune) bool {
	isInSurrogateArea := key >= 0xd800 && key <= 0xdbff
	return key >= 32 && !isInSurrogateArea
}

// translate Esc[X
func escapeExKey(key *escapeKeyPair) rune {
	var r rune
	switch key.typ {
	case 'D':
		r = CharBackward
	case 'C':
		r = CharForward
	case 'A':
		r = CharPrev
	case 'B':
		r = CharNext
	case 'H':
		r = CharLineStart
	case 'F':
		r = CharLineEnd
	case '~':
		if key.attr == "3" {
			r = CharDelete
		}
	default:
	}
	return r
}

// translate EscOX SS3 codes for up/down/etc.
func escapeSS3Key(key *escapeKeyPair) rune {
	var r rune
	switch key.typ {
	case 'D':
		r = CharBackward
	case 'C':
		r = CharForward
	case 'A':
		r = CharPrev
	case 'B':
		r = CharNext
	case 'H':
		r = CharLineStart
	case 'F':
		r = CharLineEnd
	default:
	}
	return r
}

type escapeKeyPair struct {
	attr string
	typ  rune
}

func (e *escapeKeyPair) Get2() (int, int, bool) {
	sp := strings.Split(e.attr, ";")
	if len(sp) < 2 {
		return -1, -1, false
	}
	s1, err := strconv.Atoi(sp[0])
	if err != nil {
		return -1, -1, false
	}
	s2, err := strconv.Atoi(sp[1])
	if err != nil {
		return -1, -1, false
	}
	return s1, s2, true
}

func readEscKey(r rune, reader *bufio.Reader) *escapeKeyPair {
	p := escapeKeyPair{}
	buf := bytes.NewBuffer(nil)
	for {
		if r == ';' {
		} else if unicode.IsNumber(r) {
		} else {
			p.typ = r
			break
		}
		buf.WriteRune(r)
		r, _, _ = reader.ReadRune()
	}
	p.attr = buf.String()
	return &p
}

// translate EscX to Meta+X
func escapeKey(r rune, reader *bufio.Reader) rune {
	switch r {
	case 'b':
		r = MetaBackward
	case 'f':
		r = MetaForward
	case 'd':
		r = MetaDelete
	case CharTranspose:
		r = MetaTranspose
	case CharBackspace:
		r = MetaBackspace
	case 'O':
		d, _, _ := reader.ReadRune()
		switch d {
		case 'H':
			r = CharLineStart
		case 'F':
			r = CharLineEnd
		default:
			reader.UnreadRune()
		}
	case CharEsc:

	}
	return r
}

func SplitByLine(start, screenWidth int, rs []rune) []string {
	var ret []string
	buf := bytes.NewBuffer(nil)
	currentWidth := start
	for _, r := range rs {
		w := runes.Width(r)
		currentWidth += w
		buf.WriteRune(r)
		if currentWidth >= screenWidth {
			ret = append(ret, buf.String())
			buf.Reset()
			currentWidth = 0
		}
	}
	ret = append(ret, buf.String())
	return ret
}

// calculate how many lines for N character
func LineCount(screenWidth, w int) int {
	r := w / screenWidth
	if w%screenWidth != 0 {
		r++
	}
	return r
}

func IsWordBreak(i rune) bool {
	switch {
	case i >= 'a' && i <= 'z':
	case i >= 'A' && i <= 'Z':
	case i >= '0' && i <= '9':
	default:
		return true
	}
	return false
}

func GetInt(s []string, def int) int {
	if len(s) == 0 {
		return def
	}
	c, err := strconv.Atoi(s[0])
	if err != nil {
		return def
	}
	return c
}

type RawMode struct {
	state *State
}

func (r *RawMode) Enter() (err error) {
	r.state, err = MakeRaw(GetStdin())
	return err
}

func (r *RawMode) Exit() error {
	if r.state == nil {
		return nil
	}
	return Restore(GetStdin(), r.state)
}

// -----------------------------------------------------------------------------

func sleep(n int) {
	Debug(n)
	time.Sleep(2000 * time.Millisecond)
}

// print a linked list to Debug()
func debugList(l *list.List) {
	idx := 0
	for e := l.Front(); e != nil; e = e.Next() {
		Debug(idx, fmt.Sprintf("%+v", e.Value))
		idx++
	}
}

// append log info to another file
func Debug(o ...interface{}) {
	f, _ := os.OpenFile("debug.tmp", os.O_RDWR|os.O_CREATE|os.O_APPEND, 0666)
	fmt.Fprintln(f, o...)
	f.Close()
}

func CaptureExitSignal(f func()) {
	cSignal := make(chan os.Signal, 1)
	signal.Notify(cSignal, os.Interrupt, syscall.SIGTERM)
	go func() {
		for range cSignal {
			f()
		}
	}()
}
