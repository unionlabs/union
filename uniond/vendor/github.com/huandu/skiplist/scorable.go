// Copyright 2011 Huan Du. All rights reserved.
// Licensed under the MIT license that can be found in the LICENSE file.

package skiplist

import "reflect"

// Scorable is used by skip list to optimize comparing performance.
// If two keys have different score values, they must be different keys.
//
// For any key `k1` and `k2`, the calculated score must follow these rules.
//
//     - If Compare(k1, k2) is positive, k1.Score() >= k2.Score() must be true.
//     - If Compare(k1, k2) is negative, k1.Score() <= k2.Score() must be true.
//     - If Compare(k1, k2) is 0, k1.Score() == k2.Score() must be true.
type Scorable interface {
	Score() float64
}

// CalcScore calculates score of a key.
//
// The score is a hint to optimize comparable performance.
// A skip list keeps all elements sorted by score from smaller to largest.
// If there are keys with different scores, these keys must be different.
func CalcScore(key interface{}) (score float64) {
	if scorable, ok := key.(Scorable); ok {
		score = scorable.Score()
		return
	}

	val := reflect.ValueOf(key)
	score = calcScore(val)
	return
}
