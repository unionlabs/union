/*
Copyright 2014 Google Inc. All rights reserved.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/
package bytesource

import (
	"math/rand"
	"testing"
)

func TestByteSource(t *testing.T) {
	t.Parallel()

	randFromSource := func(s ...byte) *rand.Rand {
		return rand.New(New(s))
	}

	randFromSeed := func(seed int64) *rand.Rand {
		var s1 ByteSource
		s1.Seed(seed)
		return rand.New(&s1)
	}

	t.Run("Inputs with identical 8 byte prefix", func(t *testing.T) {
		rand1 := randFromSource(1, 2, 3, 4, 5, 6, 7, 8, 9)
		rand2 := randFromSource(1, 2, 3, 4, 5, 6, 7, 8, 9)
		if rand1.Int() != rand2.Int() {
			t.Errorf("Inputs with identical 9 byte prefix result in different 1st output.")
		}
		if rand1.Int() != rand2.Int() {
			t.Errorf("Inputs with identical 9 byte prefix result in different 2nd output.")
		}
	})

	t.Run("Inputs with different 8 byte prefix", func(t *testing.T) {
		rand2 := randFromSource(1, 2, 3, 4, 5, 6, 7, 1, 9)
		rand1 := randFromSource(1, 2, 3, 4, 5, 6, 7, 8, 9)
		if rand1.Int() != rand2.Int() {
			t.Errorf("Inputs with identical 9th byte prefix result in different 1st output.")
		}
		if rand1.Int() == rand2.Int() {
			t.Errorf("Inputs with different 8 bytes prefix result in identical 2nd output.")
		}
	})

	t.Run("Multiple invocation", func(t *testing.T) {
		// First random from input byte, second from random source.
		r := randFromSource(1, 2, 3, 4, 6, 7, 8, 9)
		if r.Int() == r.Int() {
			t.Errorf("Two random numbers are identical.")
		}
		// First and second numbers from random source.
		r = randFromSource(1)
		if r.Int() == r.Int() {
			t.Errorf("Two random numbers are identical.")
		}
	})

	t.Run("Seed", func(t *testing.T) {
		if randFromSeed(42).Int() != randFromSeed(42).Int() {
			t.Error("Two random numbers from the same seed differ.")
		}
		if randFromSeed(42).Int() == randFromSeed(43).Int() {
			t.Error("Two random numbers from different seeds are identical.")
		}
	})
}

func TestByteSourceValues(t *testing.T) {
	t.Parallel()

	// Data in chunks of 8 bytes.
	data := []byte{
		99, 12, 23, 12, 65, 34, 12, 12,
		99, 12, 23, 12, 25, 34, 15, 13,
		99, 12, 23, 42, 25, 34, 11, 14,
		99, 12, 54, 12, 25, 34, 99, 11,
	}

	r := rand.New(New(data))

	got := []int{r.Int(), r.Int(), r.Int(), r.Int(), r.Int()}

	want := []int{
		3568552425102051206,
		3568552489526560135,
		3568569467532292485,
		7616166771204380295,
		5210010188159375967,
	}

	for i := range got {
		if want[i] != got[i] {
			t.Errorf("want[%d] = %d, got: %d", i, want[i], got[i])
		}
	}
}
