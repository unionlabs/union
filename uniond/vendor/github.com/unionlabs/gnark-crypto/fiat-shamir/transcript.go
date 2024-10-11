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
	"errors"
	"hash"
)

// errChallengeNotFound is returned when a wrong challenge name is provided.
var (
	errChallengeNotFound            = errors.New("challenge not recorded in the transcript")
	errChallengeAlreadyComputed     = errors.New("challenge already computed, cannot be binded to other values")
	errPreviousChallengeNotComputed = errors.New("the previous challenge is needed and has not been computed")
)

// Transcript handles the creation of challenges for Fiat Shamir.
type Transcript struct {
	// hash function that is used.
	h hash.Hash

	challenges map[string]challenge
	previous   *challenge
}

type challenge struct {
	position   int      // position of the challenge in the Transcript. order matters.
	bindings   [][]byte // bindings stores the variables a challenge is binded to.
	value      []byte   // value stores the computed challenge
	isComputed bool
}

// NewTranscript returns a new transcript.
// h is the hash function that is used to compute the challenges.
// challenges are the name of the challenges. The order of the challenges IDs matters.
func NewTranscript(h hash.Hash, challengesID ...string) Transcript {
	n := len(challengesID)
	t := Transcript{
		challenges: make(map[string]challenge, n),
		h:          h,
	}

	for i := 0; i < n; i++ {
		t.challenges[challengesID[i]] = challenge{position: i}
	}

	return t
}

// Bind binds the challenge to value. A challenge can be binded to an
// arbitrary number of values, but the order in which the binded values
// are added is important. Once a challenge is computed, it cannot be
// binded to other values.
func (t *Transcript) Bind(challengeID string, bValue []byte) error {

	challenge, ok := t.challenges[challengeID]
	if !ok {
		return errChallengeNotFound
	}

	if challenge.isComputed {
		return errChallengeAlreadyComputed
	}

	bCopy := make([]byte, len(bValue))
	copy(bCopy, bValue)
	challenge.bindings = append(challenge.bindings, bCopy)
	t.challenges[challengeID] = challenge

	return nil

}

// ComputeChallenge computes the challenge corresponding to the given name.
// The challenge is:
// * H(name || previous_challenge || binded_values...) if the challenge is not the first one
// * H(name || binded_values... ) if it is the first challenge
func (t *Transcript) ComputeChallenge(challengeID string) ([]byte, error) {

	challenge, ok := t.challenges[challengeID]
	if !ok {
		return nil, errChallengeNotFound
	}

	// if the challenge was already computed we return it
	if challenge.isComputed {
		return challenge.value, nil
	}

	// reset before populating the internal state
	t.h.Reset()
	defer t.h.Reset()

	// write the challenge name, the purpose is to have a domain separator
	if hashToField, ok := t.h.(interface {
		WriteString(rawBytes []byte)
	}); ok {
		hashToField.WriteString([]byte(challengeID)) // TODO: Replace with a function returning field identifier, whence we can find the correct hash to field function. Better than confusingly embedding hash to field into another hash
	} else {
		if _, err := t.h.Write([]byte(challengeID)); err != nil {
			return nil, err
		}
	}

	// write the previous challenge if it's not the first challenge
	if challenge.position != 0 {
		if t.previous == nil || (t.previous.position != challenge.position-1) {
			return nil, errPreviousChallengeNotComputed
		}
		if _, err := t.h.Write(t.previous.value[:]); err != nil {
			return nil, err
		}
	}

	// write the binded values in the order they were added
	for _, b := range challenge.bindings {
		if _, err := t.h.Write(b); err != nil {
			return nil, err
		}
	}

	// compute the hash of the accumulated values
	res := t.h.Sum(nil)

	challenge.value = make([]byte, len(res))
	copy(challenge.value, res)
	challenge.isComputed = true

	t.challenges[challengeID] = challenge
	t.previous = &challenge

	return res, nil

}
