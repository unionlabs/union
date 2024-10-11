// Copyright 2018 The go-ethereum Authors
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

package core

import "testing"

func TestPasswordValidation(t *testing.T) {
	testcases := []struct {
		pw         string
		shouldFail bool
	}{
		{"test", true},
		{"testtest\xbd\xb2\x3d\xbc\x20\xe2\x8c\x98", true},
		{"placeOfInterest⌘", true},
		{"password\nwith\nlinebreak", true},
		{"password\twith\vtabs", true},
		// Ok passwords
		{"password WhichIsOk", false},
		{"passwordOk!@#$%^&*()", false},
		{"12301203123012301230123012", false},
	}
	for _, test := range testcases {
		err := ValidatePasswordFormat(test.pw)
		if err == nil && test.shouldFail {
			t.Errorf("password '%v' should fail validation", test.pw)
		} else if err != nil && !test.shouldFail {
			t.Errorf("password '%v' shound not fail validation, but did: %v", test.pw, err)
		}
	}
}
