// +build windows

package readline

import (
	"reflect"
	"syscall"
	"unsafe"
)

var (
	kernel = NewKernel()
	stdout = uintptr(syscall.Stdout)
	stdin  = uintptr(syscall.Stdin)
)

type Kernel struct {
	SetConsoleCursorPosition,
	SetConsoleTextAttribute,
	FillConsoleOutputCharacterW,
	FillConsoleOutputAttribute,
	ReadConsoleInputW,
	GetConsoleScreenBufferInfo,
	GetConsoleCursorInfo,
	GetStdHandle CallFunc
}

type short int16
type word uint16
type dword uint32
type wchar uint16

type _COORD struct {
	x short
	y short
}

func (c *_COORD) ptr() uintptr {
	return uintptr(*(*int32)(unsafe.Pointer(c)))
}

const (
	EVENT_KEY                = 0x0001
	EVENT_MOUSE              = 0x0002
	EVENT_WINDOW_BUFFER_SIZE = 0x0004
	EVENT_MENU               = 0x0008
	EVENT_FOCUS              = 0x0010
)

type _KEY_EVENT_RECORD struct {
	bKeyDown          int32
	wRepeatCount      word
	wVirtualKeyCode   word
	wVirtualScanCode  word
	unicodeChar       wchar
	dwControlKeyState dword
}

// KEY_EVENT_RECORD          KeyEvent;
// MOUSE_EVENT_RECORD        MouseEvent;
// WINDOW_BUFFER_SIZE_RECORD WindowBufferSizeEvent;
// MENU_EVENT_RECORD         MenuEvent;
// FOCUS_EVENT_RECORD        FocusEvent;
type _INPUT_RECORD struct {
	EventType word
	Padding   uint16
	Event     [16]byte
}

type _CONSOLE_SCREEN_BUFFER_INFO struct {
	dwSize              _COORD
	dwCursorPosition    _COORD
	wAttributes         word
	srWindow            _SMALL_RECT
	dwMaximumWindowSize _COORD
}

type _SMALL_RECT struct {
	left   short
	top    short
	right  short
	bottom short
}

type _CONSOLE_CURSOR_INFO struct {
	dwSize   dword
	bVisible bool
}

type CallFunc func(u ...uintptr) error

func NewKernel() *Kernel {
	k := &Kernel{}
	kernel32 := syscall.NewLazyDLL("kernel32.dll")
	v := reflect.ValueOf(k).Elem()
	t := v.Type()
	for i := 0; i < t.NumField(); i++ {
		name := t.Field(i).Name
		f := kernel32.NewProc(name)
		v.Field(i).Set(reflect.ValueOf(k.Wrap(f)))
	}
	return k
}

func (k *Kernel) Wrap(p *syscall.LazyProc) CallFunc {
	return func(args ...uintptr) error {
		var r0 uintptr
		var e1 syscall.Errno
		size := uintptr(len(args))
		if len(args) <= 3 {
			buf := make([]uintptr, 3)
			copy(buf, args)
			r0, _, e1 = syscall.Syscall(p.Addr(), size,
				buf[0], buf[1], buf[2])
		} else {
			buf := make([]uintptr, 6)
			copy(buf, args)
			r0, _, e1 = syscall.Syscall6(p.Addr(), size,
				buf[0], buf[1], buf[2], buf[3], buf[4], buf[5],
			)
		}

		if int(r0) == 0 {
			if e1 != 0 {
				return error(e1)
			} else {
				return syscall.EINVAL
			}
		}
		return nil
	}

}

func GetConsoleScreenBufferInfo() (*_CONSOLE_SCREEN_BUFFER_INFO, error) {
	t := new(_CONSOLE_SCREEN_BUFFER_INFO)
	err := kernel.GetConsoleScreenBufferInfo(
		stdout,
		uintptr(unsafe.Pointer(t)),
	)
	return t, err
}

func GetConsoleCursorInfo() (*_CONSOLE_CURSOR_INFO, error) {
	t := new(_CONSOLE_CURSOR_INFO)
	err := kernel.GetConsoleCursorInfo(stdout, uintptr(unsafe.Pointer(t)))
	return t, err
}

func SetConsoleCursorPosition(c *_COORD) error {
	return kernel.SetConsoleCursorPosition(stdout, c.ptr())
}
