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

package issuelink

import (
	"bytes"
	"context"
	"fmt"

	"github.com/cockroachdb/errors/errbase"
	"github.com/cockroachdb/redact"
	"github.com/gogo/protobuf/proto"
)

type unimplementedError struct {
	// For now, msg is non-reportable.
	msg string
	IssueLink
}

var _ error = (*unimplementedError)(nil)
var _ fmt.Formatter = (*unimplementedError)(nil)
var _ errbase.SafeFormatter = (*unimplementedError)(nil)
var _ errbase.SafeDetailer = (*unimplementedError)(nil)

func (w *unimplementedError) Error() string { return w.msg }
func (w *unimplementedError) SafeDetails() []string {
	return []string{w.IssueURL, w.Detail}
}

// ErrorHint implements the hintdetail.ErrorHinter interface.
func (w *unimplementedError) ErrorHint() string {
	var hintText bytes.Buffer
	hintText.WriteString(UnimplementedErrorHint)
	maybeAppendReferral(&hintText, w.IssueLink)
	return hintText.String()
}

// UnimplementedErrorHint is the hint emitted upon unimplemented errors.
const UnimplementedErrorHint = `You have attempted to use a feature that is not yet implemented.`

func (w *unimplementedError) Format(s fmt.State, verb rune) { errbase.FormatError(w, s, verb) }

func (w *unimplementedError) SafeFormatError(p errbase.Printer) error {
	// For now, msg is non-reportable.
	p.Print(w.msg)
	if p.Detail() {
		// But the details are.
		p.Printf("unimplemented")
		if w.IssueURL != "" {
			p.Printf("\nissue: %s", redact.Safe(w.IssueURL))
		}
		if w.Detail != "" {
			p.Printf("\ndetail: %s", redact.Safe(w.Detail))
		}
	}
	return nil
}

func decodeUnimplementedError(
	_ context.Context, msg string, details []string, _ proto.Message,
) error {
	var issueLink IssueLink
	if len(details) > 0 {
		issueLink.IssueURL = details[0]
	}
	if len(details) > 1 {
		issueLink.Detail = details[1]
	}
	return &unimplementedError{msg: msg, IssueLink: issueLink}
}

func init() {
	errbase.RegisterLeafDecoder(errbase.GetTypeKey((*unimplementedError)(nil)), decodeUnimplementedError)
}
