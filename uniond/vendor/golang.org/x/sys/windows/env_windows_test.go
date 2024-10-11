// Copyright 2024 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package windows_test

import (
	"fmt"
	"slices"
	"testing"

	"golang.org/x/sys/windows"
)

func TestEnvironUTF8(t *testing.T) {
	testEnvVariable1Key := "__GO_X_SYS_WINDOWS_ENV_WINDOWS_TEST_VAR_BEAVER"
	testEnvVariable1Val := "🦫"
	t.Setenv(testEnvVariable1Key, testEnvVariable1Val)

	testEnvVariable2Key := "__GO_X_SYS_WINDOWS_ENV_WINDOWS_TEST_VAR_WHALE"
	testEnvVariable2Val := "🐳"
	t.Setenv(testEnvVariable2Key, testEnvVariable2Val)

	var userToken windows.Token

	env, err := userToken.Environ(true)
	if err != nil {
		t.Error(err)
	}

	testEnvVariable1 := fmt.Sprintf("%s=%s", testEnvVariable1Key, testEnvVariable1Val)
	if !slices.Contains(env, testEnvVariable1) {
		t.Fatalf("expected to find %s in env", testEnvVariable1)
	}

	testEnvVariable2 := fmt.Sprintf("%s=%s", testEnvVariable2Key, testEnvVariable2Val)
	if !slices.Contains(env, testEnvVariable2) {
		t.Fatalf("expected to find %s in env", testEnvVariable2)
	}
}
