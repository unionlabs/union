// Copyright 2019 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package dnsdisc

import (
	"math/rand"
	"strconv"
	"testing"
)

func TestLinkCache(t *testing.T) {
	var lc linkCache

	// Check adding links.
	lc.addLink("1", "2")
	if !lc.changed {
		t.Error("changed flag not set")
	}
	lc.changed = false
	lc.addLink("1", "2")
	if lc.changed {
		t.Error("changed flag set after adding link that's already present")
	}
	lc.addLink("2", "3")
	lc.addLink("3", "1")
	lc.addLink("2", "4")
	lc.changed = false

	if !lc.isReferenced("3") {
		t.Error("3 not referenced")
	}
	if lc.isReferenced("6") {
		t.Error("6 is referenced")
	}

	lc.resetLinks("1", nil)
	if !lc.changed {
		t.Error("changed flag not set")
	}
	if len(lc.backrefs) != 0 {
		t.Logf("%+v", lc)
		t.Error("reference maps should be empty")
	}
}

func TestLinkCacheRandom(t *testing.T) {
	tags := make([]string, 1000)
	for i := range tags {
		tags[i] = strconv.Itoa(i)
	}

	// Create random links.
	var lc linkCache
	var remove []string
	for i := 0; i < 100; i++ {
		a, b := tags[rand.Intn(len(tags))], tags[rand.Intn(len(tags))]
		lc.addLink(a, b)
		remove = append(remove, a)
	}

	// Remove all the links.
	for _, s := range remove {
		lc.resetLinks(s, nil)
	}
	if len(lc.backrefs) != 0 {
		t.Logf("%+v", lc)
		t.Error("reference maps should be empty")
	}
}
