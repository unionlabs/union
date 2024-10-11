/*
 *
 * Copyright 2022 gRPC authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package clusterresolver

import (
	"testing"

	"github.com/google/go-cmp/cmp"
	"google.golang.org/grpc/xds/internal"
	"google.golang.org/grpc/xds/internal/xdsclient/xdsresource"
)

func Test_nameGenerator_generate(t *testing.T) {
	tests := []struct {
		name   string
		prefix uint64
		input1 [][]xdsresource.Locality
		input2 [][]xdsresource.Locality
		want   []string
	}{
		{
			name:   "init, two new priorities",
			prefix: 3,
			input1: nil,
			input2: [][]xdsresource.Locality{
				{{ID: internal.LocalityID{Zone: "L0"}}},
				{{ID: internal.LocalityID{Zone: "L1"}}},
			},
			want: []string{"priority-3-0", "priority-3-1"},
		},
		{
			name:   "one new priority",
			prefix: 1,
			input1: [][]xdsresource.Locality{
				{{ID: internal.LocalityID{Zone: "L0"}}},
			},
			input2: [][]xdsresource.Locality{
				{{ID: internal.LocalityID{Zone: "L0"}}},
				{{ID: internal.LocalityID{Zone: "L1"}}},
			},
			want: []string{"priority-1-0", "priority-1-1"},
		},
		{
			name:   "merge two priorities",
			prefix: 4,
			input1: [][]xdsresource.Locality{
				{{ID: internal.LocalityID{Zone: "L0"}}},
				{{ID: internal.LocalityID{Zone: "L1"}}},
				{{ID: internal.LocalityID{Zone: "L2"}}},
			},
			input2: [][]xdsresource.Locality{
				{{ID: internal.LocalityID{Zone: "L0"}}, {ID: internal.LocalityID{Zone: "L1"}}},
				{{ID: internal.LocalityID{Zone: "L2"}}},
			},
			want: []string{"priority-4-0", "priority-4-2"},
		},
		{
			name: "swap two priorities",
			input1: [][]xdsresource.Locality{
				{{ID: internal.LocalityID{Zone: "L0"}}},
				{{ID: internal.LocalityID{Zone: "L1"}}},
				{{ID: internal.LocalityID{Zone: "L2"}}},
			},
			input2: [][]xdsresource.Locality{
				{{ID: internal.LocalityID{Zone: "L1"}}},
				{{ID: internal.LocalityID{Zone: "L0"}}},
				{{ID: internal.LocalityID{Zone: "L2"}}},
			},
			want: []string{"priority-0-1", "priority-0-0", "priority-0-2"},
		},
		{
			name: "split priority",
			input1: [][]xdsresource.Locality{
				{{ID: internal.LocalityID{Zone: "L0"}}, {ID: internal.LocalityID{Zone: "L1"}}},
				{{ID: internal.LocalityID{Zone: "L2"}}},
			},
			input2: [][]xdsresource.Locality{
				{{ID: internal.LocalityID{Zone: "L0"}}},
				{{ID: internal.LocalityID{Zone: "L1"}}}, // This gets a newly generated name, since "0-0" was already picked.
				{{ID: internal.LocalityID{Zone: "L2"}}},
			},
			want: []string{"priority-0-0", "priority-0-2", "priority-0-1"},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			ng := newNameGenerator(tt.prefix)
			got1 := ng.generate(tt.input1)
			t.Logf("%v", got1)
			got := ng.generate(tt.input2)
			if diff := cmp.Diff(got, tt.want); diff != "" {
				t.Errorf("generate() = got: %v, want: %v, diff (-got +want): %s", got, tt.want, diff)
			}
		})
	}
}
