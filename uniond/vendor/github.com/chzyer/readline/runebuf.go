package readline

import (
	"bufio"
	"bytes"
	"io"
	"strconv"
	"strings"
	"sync"
)

type runeBufferBck struct {
	buf []rune
	idx int
}

type RuneBuffer struct {
	buf    []rune
	idx    int
	prompt []rune
	w      io.Writer

	hadClean    bool
	interactive bool
	cfg         *Config

	width int

	bck *runeBufferBck

	offset string

	lastKill []rune

	sync.Mutex
}

func (r *RuneBuffer) pushKill(text []rune) {
	r.lastKill = append([]rune{}, text...)
}

func (r *RuneBuffer) OnWidthChange(newWidth int) {
	r.Lock()
	r.width = newWidth
	r.Unlock()
}

func (r *RuneBuffer) Backup() {
	r.Lock()
	r.bck = &runeBufferBck{r.buf, r.idx}
	r.Unlock()
}

func (r *RuneBuffer) Restore() {
	r.Refresh(func() {
		if r.bck == nil {
			return
		}
		r.buf = r.bck.buf
		r.idx = r.bck.idx
	})
}

func NewRuneBuffer(w io.Writer, prompt string, cfg *Config, width int) *RuneBuffer {
	rb := &RuneBuffer{
		w:           w,
		interactive: cfg.useInteractive(),
		cfg:         cfg,
		width:       width,
	}
	rb.SetPrompt(prompt)
	return rb
}

func (r *RuneBuffer) SetConfig(cfg *Config) {
	r.Lock()
	r.cfg = cfg
	r.interactive = cfg.useInteractive()
	r.Unlock()
}

func (r *RuneBuffer) SetMask(m rune) {
	r.Lock()
	r.cfg.MaskRune = m
	r.Unlock()
}

func (r *RuneBuffer) CurrentWidth(x int) int {
	r.Lock()
	defer r.Unlock()
	return runes.WidthAll(r.buf[:x])
}

func (r *RuneBuffer) PromptLen() int {
	r.Lock()
	width := r.promptLen()
	r.Unlock()
	return width
}

func (r *RuneBuffer) promptLen() int {
	return runes.WidthAll(runes.ColorFilter(r.prompt))
}

func (r *RuneBuffer) RuneSlice(i int) []rune {
	r.Lock()
	defer r.Unlock()

	if i > 0 {
		rs := make([]rune, i)
		copy(rs, r.buf[r.idx:r.idx+i])
		return rs
	}
	rs := make([]rune, -i)
	copy(rs, r.buf[r.idx+i:r.idx])
	return rs
}

func (r *RuneBuffer) Runes() []rune {
	r.Lock()
	newr := make([]rune, len(r.buf))
	copy(newr, r.buf)
	r.Unlock()
	return newr
}

func (r *RuneBuffer) Pos() int {
	r.Lock()
	defer r.Unlock()
	return r.idx
}

func (r *RuneBuffer) Len() int {
	r.Lock()
	defer r.Unlock()
	return len(r.buf)
}

func (r *RuneBuffer) MoveToLineStart() {
	r.Refresh(func() {
		if r.idx == 0 {
			return
		}
		r.idx = 0
	})
}

func (r *RuneBuffer) MoveBackward() {
	r.Refresh(func() {
		if r.idx == 0 {
			return
		}
		r.idx--
	})
}

func (r *RuneBuffer) WriteString(s string) {
	r.WriteRunes([]rune(s))
}

func (r *RuneBuffer) WriteRune(s rune) {
	r.WriteRunes([]rune{s})
}

func (r *RuneBuffer) WriteRunes(s []rune) {
	r.Refresh(func() {
		tail := append(s, r.buf[r.idx:]...)
		r.buf = append(r.buf[:r.idx], tail...)
		r.idx += len(s)
	})
}

func (r *RuneBuffer) MoveForward() {
	r.Refresh(func() {
		if r.idx == len(r.buf) {
			return
		}
		r.idx++
	})
}

func (r *RuneBuffer) IsCursorInEnd() bool {
	r.Lock()
	defer r.Unlock()
	return r.idx == len(r.buf)
}

func (r *RuneBuffer) Replace(ch rune) {
	r.Refresh(func() {
		r.buf[r.idx] = ch
	})
}

func (r *RuneBuffer) Erase() {
	r.Refresh(func() {
		r.idx = 0
		r.pushKill(r.buf[:])
		r.buf = r.buf[:0]
	})
}

func (r *RuneBuffer) Delete() (success bool) {
	r.Refresh(func() {
		if r.idx == len(r.buf) {
			return
		}
		r.pushKill(r.buf[r.idx : r.idx+1])
		r.buf = append(r.buf[:r.idx], r.buf[r.idx+1:]...)
		success = true
	})
	return
}

func (r *RuneBuffer) DeleteWord() {
	if r.idx == len(r.buf) {
		return
	}
	init := r.idx
	for init < len(r.buf) && IsWordBreak(r.buf[init]) {
		init++
	}
	for i := init + 1; i < len(r.buf); i++ {
		if !IsWordBreak(r.buf[i]) && IsWordBreak(r.buf[i-1]) {
			r.pushKill(r.buf[r.idx : i-1])
			r.Refresh(func() {
				r.buf = append(r.buf[:r.idx], r.buf[i-1:]...)
			})
			return
		}
	}
	r.Kill()
}

func (r *RuneBuffer) MoveToPrevWord() (success bool) {
	r.Refresh(func() {
		if r.idx == 0 {
			return
		}

		for i := r.idx - 1; i > 0; i-- {
			if !IsWordBreak(r.buf[i]) && IsWordBreak(r.buf[i-1]) {
				r.idx = i
				success = true
				return
			}
		}
		r.idx = 0
		success = true
	})
	return
}

func (r *RuneBuffer) KillFront() {
	r.Refresh(func() {
		if r.idx == 0 {
			return
		}

		length := len(r.buf) - r.idx
		r.pushKill(r.buf[:r.idx])
		copy(r.buf[:length], r.buf[r.idx:])
		r.idx = 0
		r.buf = r.buf[:length]
	})
}

func (r *RuneBuffer) Kill() {
	r.Refresh(func() {
		r.pushKill(r.buf[r.idx:])
		r.buf = r.buf[:r.idx]
	})
}

func (r *RuneBuffer) Transpose() {
	r.Refresh(func() {
		if len(r.buf) == 1 {
			r.idx++
		}

		if len(r.buf) < 2 {
			return
		}

		if r.idx == 0 {
			r.idx = 1
		} else if r.idx >= len(r.buf) {
			r.idx = len(r.buf) - 1
		}
		r.buf[r.idx], r.buf[r.idx-1] = r.buf[r.idx-1], r.buf[r.idx]
		r.idx++
	})
}

func (r *RuneBuffer) MoveToNextWord() {
	r.Refresh(func() {
		for i := r.idx + 1; i < len(r.buf); i++ {
			if !IsWordBreak(r.buf[i]) && IsWordBreak(r.buf[i-1]) {
				r.idx = i
				return
			}
		}

		r.idx = len(r.buf)
	})
}

func (r *RuneBuffer) MoveToEndWord() {
	r.Refresh(func() {
		// already at the end, so do nothing
		if r.idx == len(r.buf) {
			return
		}
		// if we are at the end of a word already, go to next
		if !IsWordBreak(r.buf[r.idx]) && IsWordBreak(r.buf[r.idx+1]) {
			r.idx++
		}

		// keep going until at the end of a word
		for i := r.idx + 1; i < len(r.buf); i++ {
			if IsWordBreak(r.buf[i]) && !IsWordBreak(r.buf[i-1]) {
				r.idx = i - 1
				return
			}
		}
		r.idx = len(r.buf)
	})
}

func (r *RuneBuffer) BackEscapeWord() {
	r.Refresh(func() {
		if r.idx == 0 {
			return
		}
		for i := r.idx - 1; i > 0; i-- {
			if !IsWordBreak(r.buf[i]) && IsWordBreak(r.buf[i-1]) {
				r.pushKill(r.buf[i:r.idx])
				r.buf = append(r.buf[:i], r.buf[r.idx:]...)
				r.idx = i
				return
			}
		}

		r.buf = r.buf[:0]
		r.idx = 0
	})
}

func (r *RuneBuffer) Yank() {
	if len(r.lastKill) == 0 {
		return
	}
	r.Refresh(func() {
		buf := make([]rune, 0, len(r.buf)+len(r.lastKill))
		buf = append(buf, r.buf[:r.idx]...)
		buf = append(buf, r.lastKill...)
		buf = append(buf, r.buf[r.idx:]...)
		r.buf = buf
		r.idx += len(r.lastKill)
	})
}

func (r *RuneBuffer) Backspace() {
	r.Refresh(func() {
		if r.idx == 0 {
			return
		}

		r.idx--
		r.buf = append(r.buf[:r.idx], r.buf[r.idx+1:]...)
	})
}

func (r *RuneBuffer) MoveToLineEnd() {
	r.Refresh(func() {
		if r.idx == len(r.buf) {
			return
		}

		r.idx = len(r.buf)
	})
}

func (r *RuneBuffer) LineCount(width int) int {
	if width == -1 {
		width = r.width
	}
	return LineCount(width,
		runes.WidthAll(r.buf)+r.PromptLen())
}

func (r *RuneBuffer) MoveTo(ch rune, prevChar, reverse bool) (success bool) {
	r.Refresh(func() {
		if reverse {
			for i := r.idx - 1; i >= 0; i-- {
				if r.buf[i] == ch {
					r.idx = i
					if prevChar {
						r.idx++
					}
					success = true
					return
				}
			}
			return
		}
		for i := r.idx + 1; i < len(r.buf); i++ {
			if r.buf[i] == ch {
				r.idx = i
				if prevChar {
					r.idx--
				}
				success = true
				return
			}
		}
	})
	return
}

func (r *RuneBuffer) isInLineEdge() bool {
	if isWindows {
		return false
	}
	sp := r.getSplitByLine(r.buf)
	return len(sp[len(sp)-1]) == 0
}

func (r *RuneBuffer) getSplitByLine(rs []rune) []string {
	return SplitByLine(r.promptLen(), r.width, rs)
}

func (r *RuneBuffer) IdxLine(width int) int {
	r.Lock()
	defer r.Unlock()
	return r.idxLine(width)
}

func (r *RuneBuffer) idxLine(width int) int {
	if width == 0 {
		return 0
	}
	sp := r.getSplitByLine(r.buf[:r.idx])
	return len(sp) - 1
}

func (r *RuneBuffer) CursorLineCount() int {
	return r.LineCount(r.width) - r.IdxLine(r.width)
}

func (r *RuneBuffer) Refresh(f func()) {
	r.Lock()
	defer r.Unlock()

	if !r.interactive {
		if f != nil {
			f()
		}
		return
	}

	r.clean()
	if f != nil {
		f()
	}
	r.print()
}

func (r *RuneBuffer) SetOffset(offset string) {
	r.Lock()
	r.offset = offset
	r.Unlock()
}

func (r *RuneBuffer) print() {
	r.w.Write(r.output())
	r.hadClean = false
}

func (r *RuneBuffer) output() []byte {
	buf := bytes.NewBuffer(nil)
	buf.WriteString(string(r.prompt))
	if r.cfg.EnableMask && len(r.buf) > 0 {
		buf.Write([]byte(strings.Repeat(string(r.cfg.MaskRune), len(r.buf)-1)))
		if r.buf[len(r.buf)-1] == '\n' {
			buf.Write([]byte{'\n'})
		} else {
			buf.Write([]byte(string(r.cfg.MaskRune)))
		}
		if len(r.buf) > r.idx {
			buf.Write(r.getBackspaceSequence())
		}

	} else {
		for _, e := range r.cfg.Painter.Paint(r.buf, r.idx) {
			if e == '\t' {
				buf.WriteString(strings.Repeat(" ", TabWidth))
			} else {
				buf.WriteRune(e)
			}
		}
		if r.isInLineEdge() {
			buf.Write([]byte(" \b"))
		}
	}
	// cursor position
	if len(r.buf) > r.idx {
		buf.Write(r.getBackspaceSequence())
	}
	return buf.Bytes()
}

func (r *RuneBuffer) getBackspaceSequence() []byte {
	var sep = map[int]bool{}

	var i int
	for {
		if i >= runes.WidthAll(r.buf) {
			break
		}

		if i == 0 {
			i -= r.promptLen()
		}
		i += r.width

		sep[i] = true
	}
	var buf []byte
	for i := len(r.buf); i > r.idx; i-- {
		// move input to the left of one
		buf = append(buf, '\b')
		if sep[i] {
			// up one line, go to the start of the line and move cursor right to the end (r.width)
			buf = append(buf, "\033[A\r"+"\033["+strconv.Itoa(r.width)+"C"...)
		}
	}

	return buf

}

func (r *RuneBuffer) Reset() []rune {
	ret := runes.Copy(r.buf)
	r.buf = r.buf[:0]
	r.idx = 0
	return ret
}

func (r *RuneBuffer) calWidth(m int) int {
	if m > 0 {
		return runes.WidthAll(r.buf[r.idx : r.idx+m])
	}
	return runes.WidthAll(r.buf[r.idx+m : r.idx])
}

func (r *RuneBuffer) SetStyle(start, end int, style string) {
	if end < start {
		panic("end < start")
	}

	// goto start
	move := start - r.idx
	if move > 0 {
		r.w.Write([]byte(string(r.buf[r.idx : r.idx+move])))
	} else {
		r.w.Write(bytes.Repeat([]byte("\b"), r.calWidth(move)))
	}
	r.w.Write([]byte("\033[" + style + "m"))
	r.w.Write([]byte(string(r.buf[start:end])))
	r.w.Write([]byte("\033[0m"))
	// TODO: move back
}

func (r *RuneBuffer) SetWithIdx(idx int, buf []rune) {
	r.Refresh(func() {
		r.buf = buf
		r.idx = idx
	})
}

func (r *RuneBuffer) Set(buf []rune) {
	r.SetWithIdx(len(buf), buf)
}

func (r *RuneBuffer) SetPrompt(prompt string) {
	r.Lock()
	r.prompt = []rune(prompt)
	r.Unlock()
}

func (r *RuneBuffer) cleanOutput(w io.Writer, idxLine int) {
	buf := bufio.NewWriter(w)

	if r.width == 0 {
		buf.WriteString(strings.Repeat("\r\b", len(r.buf)+r.promptLen()))
		buf.Write([]byte("\033[J"))
	} else {
		buf.Write([]byte("\033[J")) // just like ^k :)
		if idxLine == 0 {
			buf.WriteString("\033[2K")
			buf.WriteString("\r")
		} else {
			for i := 0; i < idxLine; i++ {
				io.WriteString(buf, "\033[2K\r\033[A")
			}
			io.WriteString(buf, "\033[2K\r")
		}
	}
	buf.Flush()
	return
}

func (r *RuneBuffer) Clean() {
	r.Lock()
	r.clean()
	r.Unlock()
}

func (r *RuneBuffer) clean() {
	r.cleanWithIdxLine(r.idxLine(r.width))
}

func (r *RuneBuffer) cleanWithIdxLine(idxLine int) {
	if r.hadClean || !r.interactive {
		return
	}
	r.hadClean = true
	r.cleanOutput(r.w, idxLine)
}
