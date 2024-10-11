// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package externalaccount

import (
	"context"
	"errors"
	"testing"
)

func TestRetrieveSubjectToken_ProgrammaticAuth(t *testing.T) {
	want := "subjectToken"
	opts := cloneTestOpts()

	opts.SubjectTokenProvider = fakeSubjectTokenProvider{
		subjectToken: want,
	}

	base, err := newSubjectTokenProvider(opts)
	if err != nil {
		t.Fatalf("newSubjectTokenProvider(): %v", err)
	}

	got, err := base.subjectToken(context.Background())
	if err != nil {
		t.Fatalf("subjectToken(): %v", err)
	}

	if got != want {
		t.Fatalf("got %s, want %s", got, want)
	}
}

func TestRetrieveSubjectToken_ProgrammaticAuthFails(t *testing.T) {
	want := errors.New("test error")
	opts := cloneTestOpts()

	opts.SubjectTokenProvider = fakeSubjectTokenProvider{
		err: want,
	}

	base, err := newSubjectTokenProvider(opts)
	if err != nil {
		t.Fatalf("newSubjectTokenProvider(): %v", err)
	}

	_, got := base.subjectToken(context.Background())
	if got == nil {
		t.Fatalf("subjectToken() = %v, want nil", got)
	}
	if got != want {
		t.Errorf("got %v, want %v", got, want)
	}
}

func TestRetrieveSubjectToken_ProgrammaticAuthOptions(t *testing.T) {
	opts := cloneTestOpts()
	tokOpts := &RequestOptions{Audience: opts.Audience, SubjectTokenType: opts.SubjectTokenType}

	opts.SubjectTokenProvider = fakeSubjectTokenProvider{
		subjectToken:    "subjectToken",
		expectedOptions: tokOpts,
	}

	base, err := newSubjectTokenProvider(opts)
	if err != nil {
		t.Fatalf("newSubjectTokenProvider(): %v", err)
	}

	if _, err = base.subjectToken(context.Background()); err != nil {
		t.Fatalf("subjectToken(): %v", err)
	}
}

type fakeSubjectTokenProvider struct {
	err             error
	subjectToken    string
	expectedOptions *RequestOptions
}

func (p fakeSubjectTokenProvider) SubjectToken(ctx context.Context, options *RequestOptions) (string, error) {
	if p.err != nil {
		return "", p.err
	}
	if p.expectedOptions != nil {
		if p.expectedOptions.Audience != options.Audience {
			return "", errors.New("audience does not match")
		}
		if p.expectedOptions.SubjectTokenType != options.SubjectTokenType {
			return "", errors.New("audience does not match")
		}
	}

	return p.subjectToken, nil
}
