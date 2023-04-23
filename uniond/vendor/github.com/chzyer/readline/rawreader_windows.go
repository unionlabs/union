// +build windows

package readline

import "unsafe"

const (
	VK_CANCEL   = 0x03
	VK_BACK     = 0x08
	VK_TAB      = 0x09
	VK_RETURN   = 0x0D
	VK_SHIFT    = 0x10
	VK_CONTROL  = 0x11
	VK_MENU     = 0x12
	VK_ESCAPE   = 0x1B
	VK_LEFT     = 0x25
	VK_UP       = 0x26
	VK_RIGHT    = 0x27
	VK_DOWN     = 0x28
	VK_DELETE   = 0x2E
	VK_LSHIFT   = 0xA0
	VK_RSHIFT   = 0xA1
	VK_LCONTROL = 0xA2
	VK_RCONTROL = 0xA3
)

// RawReader translate input record to ANSI escape sequence.
// To provides same behavior as unix terminal.
type RawReader struct {
	ctrlKey bool
	altKey  bool
}

func NewRawReader() *RawReader {
	r := new(RawReader)
	return r
}

// only process one action in one read
func (r *RawReader) Read(buf []byte) (int, error) {
	ir := new(_INPUT_RECORD)
	var read int
	var err error
next:
	err = kernel.ReadConsoleInputW(stdin,
		uintptr(unsafe.Pointer(ir)),
		1,
		uintptr(unsafe.Pointer(&read)),
	)
	if err != nil {
		return 0, err
	}
	if ir.EventType != EVENT_KEY {
		goto next
	}
	ker := (*_KEY_EVENT_RECORD)(unsafe.Pointer(&ir.Event[0]))
	if ker.bKeyDown == 0 { // keyup
		if r.ctrlKey || r.altKey {
			switch ker.wVirtualKeyCode {
			case VK_RCONTROL, VK_LCONTROL:
				r.ctrlKey = false
			case VK_MENU: //alt
				r.altKey = false
			}
		}
		goto next
	}

	if ker.unicodeChar == 0 {
		var target rune
		switch ker.wVirtualKeyCode {
		case VK_RCONTROL, VK_LCONTROL:
			r.ctrlKey = true
		case VK_MENU: //alt
			r.altKey = true
		case VK_LEFT:
			target = CharBackward
		case VK_RIGHT:
			target = CharForward
		case VK_UP:
			target = CharPrev
		case VK_DOWN:
			target = CharNext
		}
		if target != 0 {
			return r.write(buf, target)
		}
		goto next
	}
	char := rune(ker.unicodeChar)
	if r.ctrlKey {
		switch char {
		case 'A':
			char = CharLineStart
		case 'E':
			char = CharLineEnd
		case 'R':
			char = CharBckSearch
		case 'S':
			char = CharFwdSearch
		}
	} else if r.altKey {
		switch char {
		case VK_BACK:
			char = CharBackspace
		}
		return r.writeEsc(buf, char)
	}
	return r.write(buf, char)
}

func (r *RawReader) writeEsc(b []byte, char rune) (int, error) {
	b[0] = '\033'
	n := copy(b[1:], []byte(string(char)))
	return n + 1, nil
}

func (r *RawReader) write(b []byte, char rune) (int, error) {
	n := copy(b, []byte(string(char)))
	return n, nil
}

func (r *RawReader) Close() error {
	return nil
}
