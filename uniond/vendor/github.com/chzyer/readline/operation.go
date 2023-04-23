package readline

import (
	"errors"
	"io"
	"sync"
)

var (
	ErrInterrupt = errors.New("Interrupt")
)

type InterruptError struct {
	Line []rune
}

func (*InterruptError) Error() string {
	return "Interrupted"
}

type Operation struct {
	m       sync.Mutex
	cfg     *Config
	t       *Terminal
	buf     *RuneBuffer
	outchan chan []rune
	errchan chan error
	w       io.Writer

	history *opHistory
	*opSearch
	*opCompleter
	*opPassword
	*opVim
}

func (o *Operation) SetBuffer(what string) {
	o.buf.Set([]rune(what))
}

type wrapWriter struct {
	r      *Operation
	t      *Terminal
	target io.Writer
}

func (w *wrapWriter) Write(b []byte) (int, error) {
	if !w.t.IsReading() {
		return w.target.Write(b)
	}

	var (
		n   int
		err error
	)
	w.r.buf.Refresh(func() {
		n, err = w.target.Write(b)
	})

	if w.r.IsSearchMode() {
		w.r.SearchRefresh(-1)
	}
	if w.r.IsInCompleteMode() {
		w.r.CompleteRefresh()
	}
	return n, err
}

func NewOperation(t *Terminal, cfg *Config) *Operation {
	width := cfg.FuncGetWidth()
	op := &Operation{
		t:       t,
		buf:     NewRuneBuffer(t, cfg.Prompt, cfg, width),
		outchan: make(chan []rune),
		errchan: make(chan error, 1),
	}
	op.w = op.buf.w
	op.SetConfig(cfg)
	op.opVim = newVimMode(op)
	op.opCompleter = newOpCompleter(op.buf.w, op, width)
	op.opPassword = newOpPassword(op)
	op.cfg.FuncOnWidthChanged(func() {
		newWidth := cfg.FuncGetWidth()
		op.opCompleter.OnWidthChange(newWidth)
		op.opSearch.OnWidthChange(newWidth)
		op.buf.OnWidthChange(newWidth)
	})
	go op.ioloop()
	return op
}

func (o *Operation) SetPrompt(s string) {
	o.buf.SetPrompt(s)
}

func (o *Operation) SetMaskRune(r rune) {
	o.buf.SetMask(r)
}

func (o *Operation) GetConfig() *Config {
	o.m.Lock()
	cfg := *o.cfg
	o.m.Unlock()
	return &cfg
}

func (o *Operation) ioloop() {
	for {
		keepInSearchMode := false
		keepInCompleteMode := false
		r := o.t.ReadRune()

		if o.GetConfig().FuncFilterInputRune != nil {
			var process bool
			r, process = o.GetConfig().FuncFilterInputRune(r)
			if !process {
				o.t.KickRead()
				o.buf.Refresh(nil) // to refresh the line
				continue           // ignore this rune
			}
		}

		if r == 0 { // io.EOF
			if o.buf.Len() == 0 {
				o.buf.Clean()
				select {
				case o.errchan <- io.EOF:
				}
				break
			} else {
				// if stdin got io.EOF and there is something left in buffer,
				// let's flush them by sending CharEnter.
				// And we will got io.EOF int next loop.
				r = CharEnter
			}
		}
		isUpdateHistory := true

		if o.IsInCompleteSelectMode() {
			keepInCompleteMode = o.HandleCompleteSelect(r)
			if keepInCompleteMode {
				continue
			}

			o.buf.Refresh(nil)
			switch r {
			case CharEnter, CharCtrlJ:
				o.history.Update(o.buf.Runes(), false)
				fallthrough
			case CharInterrupt:
				o.t.KickRead()
				fallthrough
			case CharBell:
				continue
			}
		}

		if o.IsEnableVimMode() {
			r = o.HandleVim(r, o.t.ReadRune)
			if r == 0 {
				continue
			}
		}

		switch r {
		case CharBell:
			if o.IsSearchMode() {
				o.ExitSearchMode(true)
				o.buf.Refresh(nil)
			}
			if o.IsInCompleteMode() {
				o.ExitCompleteMode(true)
				o.buf.Refresh(nil)
			}
		case CharTab:
			if o.GetConfig().AutoComplete == nil {
				o.t.Bell()
				break
			}
			if o.OnComplete() {
				keepInCompleteMode = true
			} else {
				o.t.Bell()
				break
			}

		case CharBckSearch:
			if !o.SearchMode(S_DIR_BCK) {
				o.t.Bell()
				break
			}
			keepInSearchMode = true
		case CharCtrlU:
			o.buf.KillFront()
		case CharFwdSearch:
			if !o.SearchMode(S_DIR_FWD) {
				o.t.Bell()
				break
			}
			keepInSearchMode = true
		case CharKill:
			o.buf.Kill()
			keepInCompleteMode = true
		case MetaForward:
			o.buf.MoveToNextWord()
		case CharTranspose:
			o.buf.Transpose()
		case MetaBackward:
			o.buf.MoveToPrevWord()
		case MetaDelete:
			o.buf.DeleteWord()
		case CharLineStart:
			o.buf.MoveToLineStart()
		case CharLineEnd:
			o.buf.MoveToLineEnd()
		case CharBackspace, CharCtrlH:
			if o.IsSearchMode() {
				o.SearchBackspace()
				keepInSearchMode = true
				break
			}

			if o.buf.Len() == 0 {
				o.t.Bell()
				break
			}
			o.buf.Backspace()
			if o.IsInCompleteMode() {
				o.OnComplete()
			}
		case CharCtrlZ:
			o.buf.Clean()
			o.t.SleepToResume()
			o.Refresh()
		case CharCtrlL:
			ClearScreen(o.w)
			o.Refresh()
		case MetaBackspace, CharCtrlW:
			o.buf.BackEscapeWord()
		case CharCtrlY:
			o.buf.Yank()
		case CharEnter, CharCtrlJ:
			if o.IsSearchMode() {
				o.ExitSearchMode(false)
			}
			o.buf.MoveToLineEnd()
			var data []rune
			if !o.GetConfig().UniqueEditLine {
				o.buf.WriteRune('\n')
				data = o.buf.Reset()
				data = data[:len(data)-1] // trim \n
			} else {
				o.buf.Clean()
				data = o.buf.Reset()
			}
			o.outchan <- data
			if !o.GetConfig().DisableAutoSaveHistory {
				// ignore IO error
				_ = o.history.New(data)
			} else {
				isUpdateHistory = false
			}
		case CharBackward:
			o.buf.MoveBackward()
		case CharForward:
			o.buf.MoveForward()
		case CharPrev:
			buf := o.history.Prev()
			if buf != nil {
				o.buf.Set(buf)
			} else {
				o.t.Bell()
			}
		case CharNext:
			buf, ok := o.history.Next()
			if ok {
				o.buf.Set(buf)
			} else {
				o.t.Bell()
			}
		case CharDelete:
			if o.buf.Len() > 0 || !o.IsNormalMode() {
				o.t.KickRead()
				if !o.buf.Delete() {
					o.t.Bell()
				}
				break
			}

			// treat as EOF
			if !o.GetConfig().UniqueEditLine {
				o.buf.WriteString(o.GetConfig().EOFPrompt + "\n")
			}
			o.buf.Reset()
			isUpdateHistory = false
			o.history.Revert()
			o.errchan <- io.EOF
			if o.GetConfig().UniqueEditLine {
				o.buf.Clean()
			}
		case CharInterrupt:
			if o.IsSearchMode() {
				o.t.KickRead()
				o.ExitSearchMode(true)
				break
			}
			if o.IsInCompleteMode() {
				o.t.KickRead()
				o.ExitCompleteMode(true)
				o.buf.Refresh(nil)
				break
			}
			o.buf.MoveToLineEnd()
			o.buf.Refresh(nil)
			hint := o.GetConfig().InterruptPrompt + "\n"
			if !o.GetConfig().UniqueEditLine {
				o.buf.WriteString(hint)
			}
			remain := o.buf.Reset()
			if !o.GetConfig().UniqueEditLine {
				remain = remain[:len(remain)-len([]rune(hint))]
			}
			isUpdateHistory = false
			o.history.Revert()
			o.errchan <- &InterruptError{remain}
		default:
			if o.IsSearchMode() {
				o.SearchChar(r)
				keepInSearchMode = true
				break
			}
			o.buf.WriteRune(r)
			if o.IsInCompleteMode() {
				o.OnComplete()
				keepInCompleteMode = true
			}
		}

		listener := o.GetConfig().Listener
		if listener != nil {
			newLine, newPos, ok := listener.OnChange(o.buf.Runes(), o.buf.Pos(), r)
			if ok {
				o.buf.SetWithIdx(newPos, newLine)
			}
		}

		o.m.Lock()
		if !keepInSearchMode && o.IsSearchMode() {
			o.ExitSearchMode(false)
			o.buf.Refresh(nil)
		} else if o.IsInCompleteMode() {
			if !keepInCompleteMode {
				o.ExitCompleteMode(false)
				o.Refresh()
			} else {
				o.buf.Refresh(nil)
				o.CompleteRefresh()
			}
		}
		if isUpdateHistory && !o.IsSearchMode() {
			// it will cause null history
			o.history.Update(o.buf.Runes(), false)
		}
		o.m.Unlock()
	}
}

func (o *Operation) Stderr() io.Writer {
	return &wrapWriter{target: o.GetConfig().Stderr, r: o, t: o.t}
}

func (o *Operation) Stdout() io.Writer {
	return &wrapWriter{target: o.GetConfig().Stdout, r: o, t: o.t}
}

func (o *Operation) String() (string, error) {
	r, err := o.Runes()
	return string(r), err
}

func (o *Operation) Runes() ([]rune, error) {
	o.t.EnterRawMode()
	defer o.t.ExitRawMode()

	listener := o.GetConfig().Listener
	if listener != nil {
		listener.OnChange(nil, 0, 0)
	}

	o.buf.Refresh(nil) // print prompt
	o.t.KickRead()
	select {
	case r := <-o.outchan:
		return r, nil
	case err := <-o.errchan:
		if e, ok := err.(*InterruptError); ok {
			return e.Line, ErrInterrupt
		}
		return nil, err
	}
}

func (o *Operation) PasswordEx(prompt string, l Listener) ([]byte, error) {
	cfg := o.GenPasswordConfig()
	cfg.Prompt = prompt
	cfg.Listener = l
	return o.PasswordWithConfig(cfg)
}

func (o *Operation) GenPasswordConfig() *Config {
	return o.opPassword.PasswordConfig()
}

func (o *Operation) PasswordWithConfig(cfg *Config) ([]byte, error) {
	if err := o.opPassword.EnterPasswordMode(cfg); err != nil {
		return nil, err
	}
	defer o.opPassword.ExitPasswordMode()
	return o.Slice()
}

func (o *Operation) Password(prompt string) ([]byte, error) {
	return o.PasswordEx(prompt, nil)
}

func (o *Operation) SetTitle(t string) {
	o.w.Write([]byte("\033[2;" + t + "\007"))
}

func (o *Operation) Slice() ([]byte, error) {
	r, err := o.Runes()
	if err != nil {
		return nil, err
	}
	return []byte(string(r)), nil
}

func (o *Operation) Close() {
	select {
	case o.errchan <- io.EOF:
	default:
	}
	o.history.Close()
}

func (o *Operation) SetHistoryPath(path string) {
	if o.history != nil {
		o.history.Close()
	}
	o.cfg.HistoryFile = path
	o.history = newOpHistory(o.cfg)
}

func (o *Operation) IsNormalMode() bool {
	return !o.IsInCompleteMode() && !o.IsSearchMode()
}

func (op *Operation) SetConfig(cfg *Config) (*Config, error) {
	op.m.Lock()
	defer op.m.Unlock()
	if op.cfg == cfg {
		return op.cfg, nil
	}
	if err := cfg.Init(); err != nil {
		return op.cfg, err
	}
	old := op.cfg
	op.cfg = cfg
	op.SetPrompt(cfg.Prompt)
	op.SetMaskRune(cfg.MaskRune)
	op.buf.SetConfig(cfg)
	width := op.cfg.FuncGetWidth()

	if cfg.opHistory == nil {
		op.SetHistoryPath(cfg.HistoryFile)
		cfg.opHistory = op.history
		cfg.opSearch = newOpSearch(op.buf.w, op.buf, op.history, cfg, width)
	}
	op.history = cfg.opHistory

	// SetHistoryPath will close opHistory which already exists
	// so if we use it next time, we need to reopen it by `InitHistory()`
	op.history.Init()

	if op.cfg.AutoComplete != nil {
		op.opCompleter = newOpCompleter(op.buf.w, op, width)
	}

	op.opSearch = cfg.opSearch
	return old, nil
}

func (o *Operation) ResetHistory() {
	o.history.Reset()
}

// if err is not nil, it just mean it fail to write to file
// other things goes fine.
func (o *Operation) SaveHistory(content string) error {
	return o.history.New([]rune(content))
}

func (o *Operation) Refresh() {
	if o.t.IsReading() {
		o.buf.Refresh(nil)
	}
}

func (o *Operation) Clean() {
	o.buf.Clean()
}

func FuncListener(f func(line []rune, pos int, key rune) (newLine []rune, newPos int, ok bool)) Listener {
	return &DumpListener{f: f}
}

type DumpListener struct {
	f func(line []rune, pos int, key rune) (newLine []rune, newPos int, ok bool)
}

func (d *DumpListener) OnChange(line []rune, pos int, key rune) (newLine []rune, newPos int, ok bool) {
	return d.f(line, pos, key)
}

type Listener interface {
	OnChange(line []rune, pos int, key rune) (newLine []rune, newPos int, ok bool)
}

type Painter interface {
	Paint(line []rune, pos int) []rune
}

type defaultPainter struct{}

func (p *defaultPainter) Paint(line []rune, _ int) []rune {
	return line
}
