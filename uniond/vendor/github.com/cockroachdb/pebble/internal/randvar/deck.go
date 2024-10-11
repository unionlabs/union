// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package randvar

import (
	"sync"

	"golang.org/x/exp/rand"
)

// Deck is a random number generator that generates numbers in the range
// [0,len(weights)-1] where the probability of i is
// weights(i)/sum(weights). Unlike Weighted, the weights are specified as
// integers and used in a deck-of-cards style random number selection which
// ensures that each element is returned with a desired frequency within the
// size of the deck.
type Deck struct {
	rng *rand.Rand
	mu  struct {
		sync.Mutex
		index int
		deck  []int
	}
}

// NewDeck returns a new deck random number generator.
func NewDeck(rng *rand.Rand, weights ...int) *Deck {
	var sum int
	for i := range weights {
		sum += weights[i]
	}
	deck := make([]int, 0, sum)
	for i := range weights {
		for j := 0; j < weights[i]; j++ {
			deck = append(deck, i)
		}
	}
	d := &Deck{
		rng: ensureRand(rng),
	}
	d.mu.index = len(deck)
	d.mu.deck = deck
	return d
}

// Int returns a random number in the range [0,len(weights)-1] where the
// probability of i is weights(i)/sum(weights).
func (d *Deck) Int() int {
	d.mu.Lock()
	if d.mu.index == len(d.mu.deck) {
		d.rng.Shuffle(len(d.mu.deck), func(i, j int) {
			d.mu.deck[i], d.mu.deck[j] = d.mu.deck[j], d.mu.deck[i]
		})
		d.mu.index = 0
	}
	result := d.mu.deck[d.mu.index]
	d.mu.index++
	d.mu.Unlock()
	return result
}
