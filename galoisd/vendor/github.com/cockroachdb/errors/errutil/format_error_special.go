// Copyright 2020 The Cockroach Authors.
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

package errutil

import (
	"context"
	"net"
	"os"
	"runtime"
	"syscall"

	"github.com/cockroachdb/errors/errbase"
	"github.com/cockroachdb/errors/markers"
	"github.com/cockroachdb/redact"
)

func init() {
	errbase.RegisterSpecialCasePrinter(specialCaseFormat)
}

func specialCaseFormat(err error, p errbase.Printer, isLeaf bool) (handled bool, next error) {
	if isLeaf && markers.IsAny(err,
		context.DeadlineExceeded,
		context.Canceled,
		os.ErrInvalid,
		os.ErrPermission,
		os.ErrExist,
		os.ErrNotExist,
		os.ErrClosed,
		os.ErrNoDeadline) {
		p.Print(redact.Safe(err.Error()))
		return true, nil
	}

	switch v := err.(type) {
	// The following two types are safe too.
	case runtime.Error:
		p.Print(redact.Safe(v.Error()))
		return true, err
	case syscall.Errno:
		p.Print(redact.Safe(v.Error()))
		return true, err
	case *os.SyscallError:
		p.Print(redact.Safe(v.Syscall))
		return true, err
	case *os.PathError:
		p.Printf("%s %s", redact.Safe(v.Op), v.Path)
		return true, err
	case *os.LinkError:
		p.Printf("%s %s %s", redact.Safe(v.Op), v.Old, v.New)
		return true, err
	case *net.OpError:
		p.Print(redact.Safe(v.Op))
		if v.Net != "" {
			p.Printf(" %s", redact.Safe(v.Net))
		}
		if v.Source != nil {
			p.Printf(" %s", v.Source)
		}
		if v.Addr != nil {
			if v.Source != nil {
				p.Printf(" ->")
			}
			p.Printf(" %s", v.Addr)
		}
		return true, err
	case redact.SafeMessager:
		// Backward-compatibility with previous versions
		// of the errors library: if an error type implements
		// SafeMessage(), use that instead of its error message.
		p.Print(redact.Safe(v.SafeMessage()))
		// It also short-cuts any further causes.
		return true, nil
	}
	return false, nil
}
