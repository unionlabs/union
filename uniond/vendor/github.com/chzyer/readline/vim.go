package readline

const (
	VIM_NORMAL = iota
	VIM_INSERT
	VIM_VISUAL
)

type opVim struct {
	cfg     *Config
	op      *Operation
	vimMode int
}

func newVimMode(op *Operation) *opVim {
	ov := &opVim{
		cfg: op.cfg,
		op:  op,
	}
	ov.SetVimMode(ov.cfg.VimMode)
	return ov
}

func (o *opVim) SetVimMode(on bool) {
	if o.cfg.VimMode && !on { // turn off
		o.ExitVimMode()
	}
	o.cfg.VimMode = on
	o.vimMode = VIM_INSERT
}

func (o *opVim) ExitVimMode() {
	o.vimMode = VIM_INSERT
}

func (o *opVim) IsEnableVimMode() bool {
	return o.cfg.VimMode
}

func (o *opVim) handleVimNormalMovement(r rune, readNext func() rune) (t rune, handled bool) {
	rb := o.op.buf
	handled = true
	switch r {
	case 'h':
		t = CharBackward
	case 'j':
		t = CharNext
	case 'k':
		t = CharPrev
	case 'l':
		t = CharForward
	case '0', '^':
		rb.MoveToLineStart()
	case '$':
		rb.MoveToLineEnd()
	case 'x':
		rb.Delete()
		if rb.IsCursorInEnd() {
			rb.MoveBackward()
		}
	case 'r':
		rb.Replace(readNext())
	case 'd':
		next := readNext()
		switch next {
		case 'd':
			rb.Erase()
		case 'w':
			rb.DeleteWord()
		case 'h':
			rb.Backspace()
		case 'l':
			rb.Delete()
		}
	case 'p':
		rb.Yank()
	case 'b', 'B':
		rb.MoveToPrevWord()
	case 'w', 'W':
		rb.MoveToNextWord()
	case 'e', 'E':
		rb.MoveToEndWord()
	case 'f', 'F', 't', 'T':
		next := readNext()
		prevChar := r == 't' || r == 'T'
		reverse := r == 'F' || r == 'T'
		switch next {
		case CharEsc:
		default:
			rb.MoveTo(next, prevChar, reverse)
		}
	default:
		return r, false
	}
	return t, true
}

func (o *opVim) handleVimNormalEnterInsert(r rune, readNext func() rune) (t rune, handled bool) {
	rb := o.op.buf
	handled = true
	switch r {
	case 'i':
	case 'I':
		rb.MoveToLineStart()
	case 'a':
		rb.MoveForward()
	case 'A':
		rb.MoveToLineEnd()
	case 's':
		rb.Delete()
	case 'S':
		rb.Erase()
	case 'c':
		next := readNext()
		switch next {
		case 'c':
			rb.Erase()
		case 'w':
			rb.DeleteWord()
		case 'h':
			rb.Backspace()
		case 'l':
			rb.Delete()
		}
	default:
		return r, false
	}

	o.EnterVimInsertMode()
	return
}

func (o *opVim) HandleVimNormal(r rune, readNext func() rune) (t rune) {
	switch r {
	case CharEnter, CharInterrupt:
		o.ExitVimMode()
		return r
	}

	if r, handled := o.handleVimNormalMovement(r, readNext); handled {
		return r
	}

	if r, handled := o.handleVimNormalEnterInsert(r, readNext); handled {
		return r
	}

	// invalid operation
	o.op.t.Bell()
	return 0
}

func (o *opVim) EnterVimInsertMode() {
	o.vimMode = VIM_INSERT
}

func (o *opVim) ExitVimInsertMode() {
	o.vimMode = VIM_NORMAL
}

func (o *opVim) HandleVim(r rune, readNext func() rune) rune {
	if o.vimMode == VIM_NORMAL {
		return o.HandleVimNormal(r, readNext)
	}
	if r == CharEsc {
		o.ExitVimInsertMode()
		return 0
	}

	switch o.vimMode {
	case VIM_INSERT:
		return r
	case VIM_VISUAL:
	}
	return r
}
