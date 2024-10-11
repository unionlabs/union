// Copyright 2024 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package externalaccount

import (
	"context"
	"errors"
	"testing"
)

func TestRetrieveSubjectToken_ProgrammaticAuth(t *testing.T) {
	tfc := testConfig

	tfc.SubjectTokenSupplier = testSubjectTokenSupplier{
		subjectToken: "subjectToken",
	}

	base, err := tfc.parse(context.Background())
	if err != nil {
		t.Fatalf("parse() failed %v", err)
	}

	out, err := base.subjectToken()
	if err != nil {
		t.Fatalf("retrieveSubjectToken() failed: %v", err)
	}

	if out != "subjectToken" {
		t.Errorf("subjectToken = \n%q\n want \nSubjectToken", out)
	}
}

func TestRetrieveSubjectToken_ProgrammaticAuthFails(t *testing.T) {
	tfc := testConfig
	testError := errors.New("test error")

	tfc.SubjectTokenSupplier = testSubjectTokenSupplier{
		err: testError,
	}

	base, err := tfc.parse(context.Background())
	if err != nil {
		t.Fatalf("parse() failed %v", err)
	}

	_, err = base.subjectToken()
	if err == nil {
		t.Fatalf("subjectToken() should have failed")
	}
	if testError != err {
		t.Errorf("subjectToken = %e, want %e", err, testError)
	}
}

func TestRetrieveSubjectToken_ProgrammaticAuthOptions(t *testing.T) {
	tfc := testConfig
	expectedOptions := SupplierOptions{Audience: tfc.Audience, SubjectTokenType: tfc.SubjectTokenType}

	tfc.SubjectTokenSupplier = testSubjectTokenSupplier{
		subjectToken:    "subjectToken",
		expectedOptions: &expectedOptions,
	}

	base, err := tfc.parse(context.Background())
	if err != nil {
		t.Fatalf("parse() failed %v", err)
	}

	_, err = base.subjectToken()
	if err != nil {
		t.Fatalf("retrieveSubjectToken() failed: %v", err)
	}
}

func TestRetrieveSubjectToken_ProgrammaticAuthContext(t *testing.T) {
	tfc := testConfig
	ctx := context.Background()

	tfc.SubjectTokenSupplier = testSubjectTokenSupplier{
		subjectToken:    "subjectToken",
		expectedContext: ctx,
	}

	base, err := tfc.parse(ctx)
	if err != nil {
		t.Fatalf("parse() failed %v", err)
	}

	_, err = base.subjectToken()
	if err != nil {
		t.Fatalf("retrieveSubjectToken() failed: %v", err)
	}
}

type testSubjectTokenSupplier struct {
	err             error
	subjectToken    string
	expectedOptions *SupplierOptions
	expectedContext context.Context
}

func (supp testSubjectTokenSupplier) SubjectToken(ctx context.Context, options SupplierOptions) (string, error) {
	if supp.err != nil {
		return "", supp.err
	}
	if supp.expectedOptions != nil {
		if supp.expectedOptions.Audience != options.Audience {
			return "", errors.New("Audience does not match")
		}
		if supp.expectedOptions.SubjectTokenType != options.SubjectTokenType {
			return "", errors.New("Audience does not match")
		}
	}
	if supp.expectedContext != nil {
		if supp.expectedContext != ctx {
			return "", errors.New("Context does not match")
		}
	}
	return supp.subjectToken, nil
}
