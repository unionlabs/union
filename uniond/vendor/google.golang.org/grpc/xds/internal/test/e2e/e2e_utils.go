/*
 *
 * Copyright 2021 gRPC authors.
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

package e2e

import (
	"testing"

	"github.com/google/go-cmp/cmp"
	channelzpb "google.golang.org/grpc/channelz/grpc_channelz_v1"
)

func verifySubConnStates(t *testing.T, scs []*channelzpb.Subchannel, want map[channelzpb.ChannelConnectivityState_State]int) {
	t.Helper()
	var scStatsCount = map[channelzpb.ChannelConnectivityState_State]int{}
	for _, sc := range scs {
		scStatsCount[sc.Data.State.State]++
	}
	if diff := cmp.Diff(scStatsCount, want); diff != "" {
		t.Fatalf("got unexpected number of subchannels in state Ready, %v, scs: %v", diff, scs)
	}
}
