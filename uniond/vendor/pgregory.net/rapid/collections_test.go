// Copyright 2019 Gregory Petrosyan <gregory.petrosyan@gmail.com>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package rapid

import "testing"

func TestCollectionsWithImpossibleMinSize(t *testing.T) {
	t.Parallel()

	s := createRandomBitStream(t)
	gens := []*Generator[any]{
		MapOfN(Bool(), Int(), 10, -1).AsAny(),
		SliceOfNDistinct(Int(), 10, -1, func(i int) int { return i % 5 }).AsAny(),
	}

	for _, g := range gens {
		t.Run(g.String(), func(t *testing.T) {
			_, err := recoverValue(g, newT(nil, s, false, nil))
			if err == nil || !err.isInvalidData() {
				t.Fatalf("got error %v instead of invalid data", err)
			}
		})
	}
}
