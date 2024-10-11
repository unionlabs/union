// Copyright 2015 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package flags

import (
	"os"
	"os/user"
	"runtime"
	"testing"
)

func TestPathExpansion(t *testing.T) {
	user, _ := user.Current()
	var tests map[string]string

	if runtime.GOOS == "windows" {
		tests = map[string]string{
			`/home/someuser/tmp`:        `\home\someuser\tmp`,
			`~/tmp`:                     user.HomeDir + `\tmp`,
			`~thisOtherUser/b/`:         `~thisOtherUser\b`,
			`$DDDXXX/a/b`:               `\tmp\a\b`,
			`/a/b/`:                     `\a\b`,
			`C:\Documents\Newsletters\`: `C:\Documents\Newsletters`,
			`C:\`:                       `C:\`,
			`\\.\pipe\\pipe\geth621383`: `\\.\pipe\\pipe\geth621383`,
		}
	} else {
		tests = map[string]string{
			`/home/someuser/tmp`:        `/home/someuser/tmp`,
			`~/tmp`:                     user.HomeDir + `/tmp`,
			`~thisOtherUser/b/`:         `~thisOtherUser/b`,
			`$DDDXXX/a/b`:               `/tmp/a/b`,
			`/a/b/`:                     `/a/b`,
			`C:\Documents\Newsletters\`: `C:\Documents\Newsletters\`,
			`C:\`:                       `C:\`,
			`\\.\pipe\\pipe\geth621383`: `\\.\pipe\\pipe\geth621383`,
		}
	}

	os.Setenv(`DDDXXX`, `/tmp`)
	for test, expected := range tests {
		got := expandPath(test)
		if got != expected {
			t.Errorf(`test %s, got %s, expected %s\n`, test, got, expected)
		}
	}
}
