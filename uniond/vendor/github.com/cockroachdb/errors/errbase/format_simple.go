// Copyright 2019 The Cockroach Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied. See the License for the specific language governing
// permissions and limitations under the License.

package errbase

import "io"

// formatSimpleError is a helper used by FormatError() for the top
// level error/wrapper argument, if it does not implement the
// errors.Formatter interface.
func formatSimpleError(err error, p *state, sep string) error {
	if cause := UnwrapOnce(err); cause != nil {
		pref := extractPrefix(err, cause)
		p.buf.WriteString(pref)
		if pref != "" {
			p.buf.WriteByte(':')
			p.buf.WriteString(sep)
		}
		err = cause
	} else {
		io.WriteString(&p.buf, err.Error())
		err = nil
	}
	return err
}
