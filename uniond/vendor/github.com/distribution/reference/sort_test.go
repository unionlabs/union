/*
   Copyright The containerd Authors.

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

package reference

import (
	"io"
	"math/rand"
	"testing"

	"github.com/opencontainers/go-digest"
)

func TestReferenceSorting(t *testing.T) {
	t.Parallel()
	digested := func(seed int64) string {
		b, err := io.ReadAll(io.LimitReader(rand.New(rand.NewSource(seed)), 64))
		if err != nil {
			panic(err)
		}
		return digest.FromBytes(b).String()
	}
	// Add z. prefix to string sort after "sha256:"
	r1 := func(name, tag string, seed int64) string {
		return "z.containerd.io/" + name + ":" + tag + "@" + digested(seed)
	}
	r2 := func(name, tag string) string {
		return "z.containerd.io/" + name + ":" + tag
	}
	r3 := func(name string, seed int64) string {
		return "z.containerd.io/" + name + "@" + digested(seed)
	}

	for i, tc := range []struct {
		unsorted []string
		expected []string
	}{
		{
			unsorted: []string{r2("name", "latest"), r3("name", 1), r1("name", "latest", 1)},
			expected: []string{r1("name", "latest", 1), r2("name", "latest"), r3("name", 1)},
		},
		{
			unsorted: []string{"can't parse this:latest", r3("name", 1), r2("name", "latest")},
			expected: []string{r2("name", "latest"), r3("name", 1), "can't parse this:latest"},
		},
		{
			unsorted: []string{digested(1), r3("name", 1), r2("name", "latest")},
			expected: []string{r2("name", "latest"), r3("name", 1), digested(1)},
		},
		{
			unsorted: []string{r2("name", "tag2"), r2("name", "tag3"), r2("name", "tag1")},
			expected: []string{r2("name", "tag1"), r2("name", "tag2"), r2("name", "tag3")},
		},
		{
			unsorted: []string{r2("name-2", "tag"), r2("name-3", "tag"), r2("name-1", "tag")},
			expected: []string{r2("name-1", "tag"), r2("name-2", "tag"), r2("name-3", "tag")},
		},
	} {
		sorted := Sort(tc.unsorted)
		if len(sorted) != len(tc.expected) {
			t.Errorf("[%d]: Mismatched sized, got %d, expected %d", i, len(sorted), len(tc.expected))
			continue
		}
		for j := range sorted {
			if sorted[j] != tc.expected[j] {
				t.Errorf("[%d]: Wrong value at %d, got %q, expected %q", i, j, sorted[j], tc.expected[j])
				break
			}
		}
	}
}
