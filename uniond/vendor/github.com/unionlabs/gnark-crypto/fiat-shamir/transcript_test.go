// Copyright 2020 ConsenSys Software Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package fiatshamir

import (
	"bytes"
	"crypto/sha256"
	"testing"
)

func initTranscript() Transcript {

	fs := NewTranscript(sha256.New(), "alpha", "beta", "gamma")

	values := [][]byte{[]byte("v1"), []byte("v2"), []byte("v3"), []byte("v4"), []byte("v5"), []byte("v6")}
	if err := fs.Bind("alpha", values[0]); err != nil {
		panic(err)
	}
	if err := fs.Bind("alpha", values[1]); err != nil {
		panic(err)
	}
	if err := fs.Bind("beta", values[2]); err != nil {
		panic(err)
	}
	if err := fs.Bind("beta", values[3]); err != nil {
		panic(err)
	}
	if err := fs.Bind("gamma", values[4]); err != nil {
		panic(err)
	}
	if err := fs.Bind("gamma", values[5]); err != nil {
		panic(err)
	}

	return fs
}

func TestTranscript(t *testing.T) {
	t.Parallel()

	fs := initTranscript()

	// test when everything is fine
	alpha, err := fs.ComputeChallenge("alpha")
	if err != nil {
		t.Fatal(err)
	}
	beta, err := fs.ComputeChallenge("beta")
	if err != nil {
		t.Fatal(err)
	}
	gamma, err := fs.ComputeChallenge("gamma")
	if err != nil {
		t.Fatal(err)
	}

	if len(alpha) == 0 || len(beta) == 0 || len(gamma) == 0 {
		t.Fatal("one of the challenge result is empty")
	}

	// re compute the challenges to verify they are the same
	alphaBis, err := fs.ComputeChallenge("alpha")
	if err != nil {
		t.Fatal(err)
	}
	betaBis, err := fs.ComputeChallenge("beta")
	if err != nil {
		t.Fatal(err)
	}
	gammaBis, err := fs.ComputeChallenge("gamma")
	if err != nil {
		t.Fatal(err)
	}

	if !bytes.Equal(alpha, alphaBis) {
		t.Fatal("computing the same challenge twice should return the same value")
	}
	if !bytes.Equal(beta, betaBis) {
		t.Fatal("computing the same challenge twice should return the same value")
	}
	if !bytes.Equal(gamma, gammaBis) {
		t.Fatal("computing the same challenge twice should return the same value")
	}

}

func TestNonExistingChallenge(t *testing.T) {
	t.Parallel()

	fs := initTranscript()

	// query inexisting challenges
	_, err := fs.ComputeChallenge("delta")
	if err == nil {
		t.Fatal(err)
	}

}

func TestWrongOrder(t *testing.T) {
	t.Parallel()

	fs := initTranscript()

	// query inexisting challenges
	_, err := fs.ComputeChallenge("beta")
	if err == nil {
		t.Fatal(err)
	}

}

func TestBindToComputedChallenge(t *testing.T) {
	t.Parallel()

	fs := initTranscript()

	_, err := fs.ComputeChallenge("alpha")
	if err != nil {
		t.Fatal(err)
	}

	// bind value to an already computed challenge
	err = fs.Bind("alpha", []byte("test"))
	if err == nil {
		t.Fatal(err)
	}

}
