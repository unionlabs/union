package term

import (
	"fmt"
	"syscall"
	"testing"
)

type myWriter struct {
	fd uintptr
}

func (w *myWriter) Write(p []byte) (int, error) {
	return 0, fmt.Errorf("not implemented")
}

func (w *myWriter) Fd() uintptr {
	return w.fd
}

var procGetStdHandle = kernel32.NewProc("GetStdHandle") //lint:ignore U1000 unused

const stdOutputHandle = ^uintptr(0) - 11 + 1 //lint:ignore U1000 unused

func getConsoleHandle() syscall.Handle {
	ptr, err := syscall.UTF16PtrFromString("CONOUT$")
	if err != nil {
		panic(err)
	}

	handle, err := syscall.CreateFile(ptr, syscall.GENERIC_READ|syscall.GENERIC_WRITE, syscall.FILE_SHARE_READ, nil, syscall.OPEN_EXISTING, 0, 0)
	if err != nil {
		panic(err)
	}

	return handle
}

func TestIsTerminal(t *testing.T) {
	// This is necessary because depending on whether `go test` is called with
	// the `-v` option, stdout will or will not be bound, changing the behavior
	// of the test. So we refer to it directly to avoid flakyness.
	handle := getConsoleHandle()

	writer := &myWriter{
		fd: uintptr(handle),
	}

	if !IsTerminal(writer) {
		t.Errorf("output is supposed to be a terminal")
	}
}

func TestIsConsole(t *testing.T) {
	// This is necessary because depending on whether `go test` is called with
	// the `-v` option, stdout will or will not be bound, changing the behavior
	// of the test. So we refer to it directly to avoid flakyness.
	handle := getConsoleHandle()

	writer := &myWriter{
		fd: uintptr(handle),
	}

	if !IsConsole(writer) {
		t.Errorf("output is supposed to be a console")
	}
}
