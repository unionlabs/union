// +build windows

package readline

import (
	"io"
	"syscall"
)

func SuspendMe() {
}

func GetStdin() int {
	return int(syscall.Stdin)
}

func init() {
	isWindows = true
}

// get width of the terminal
func GetScreenWidth() int {
	info, _ := GetConsoleScreenBufferInfo()
	if info == nil {
		return -1
	}
	return int(info.dwSize.x)
}

// ClearScreen clears the console screen
func ClearScreen(_ io.Writer) error {
	return SetConsoleCursorPosition(&_COORD{0, 0})
}

func DefaultIsTerminal() bool {
	return true
}

func DefaultOnWidthChanged(func()) {

}
