// Copyright 2013 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// +build aix os400 solaris

package readline

import "golang.org/x/sys/unix"

// GetSize returns the dimensions of the given terminal.
func GetSize(fd int) (int, int, error) {
	ws, err := unix.IoctlGetWinsize(fd, unix.TIOCGWINSZ)
	if err != nil {
		return 0, 0, err
	}
	return int(ws.Col), int(ws.Row), nil
}

type Termios unix.Termios

func getTermios(fd int) (*Termios, error) {
	termios, err := unix.IoctlGetTermios(fd, unix.TCGETS)
	if err != nil {
		return nil, err
	}
	return (*Termios)(termios), nil
}

func setTermios(fd int, termios *Termios) error {
	return unix.IoctlSetTermios(fd, unix.TCSETSF, (*unix.Termios)(termios))
}
