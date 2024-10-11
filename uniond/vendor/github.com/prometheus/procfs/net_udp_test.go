// Copyright 2020 The Prometheus Authors
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package procfs

import (
	"net"
	"reflect"
	"testing"
)

func Test_newNetUDP(t *testing.T) {
	tests := []struct {
		name    string
		file    string
		want    NetUDP
		wantErr bool
	}{
		{
			name: "udp file found, no error should come up",
			file: "testdata/fixtures/proc/net/udp",
			want: []*netIPSocketLine{
				&netIPSocketLine{
					Sl:        0,
					LocalAddr: net.IP{10, 0, 0, 5},
					LocalPort: 22,
					RemAddr:   net.IP{0, 0, 0, 0},
					RemPort:   0,
					St:        10,
					TxQueue:   0,
					RxQueue:   1,
					UID:       0,
					Inode:     2740,
					Drops:     intToU64(100),
				},
				&netIPSocketLine{
					Sl:        1,
					LocalAddr: net.IP{0, 0, 0, 0},
					LocalPort: 22,
					RemAddr:   net.IP{0, 0, 0, 0},
					RemPort:   0,
					St:        10,
					TxQueue:   1,
					RxQueue:   0,
					UID:       0,
					Inode:     2740,
					Drops:     intToU64(100),
				},
				&netIPSocketLine{
					Sl:        2,
					LocalAddr: net.IP{0, 0, 0, 0},
					LocalPort: 22,
					RemAddr:   net.IP{0, 0, 0, 0},
					RemPort:   0,
					St:        10,
					TxQueue:   1,
					RxQueue:   1,
					UID:       0,
					Inode:     2740,
					Drops:     intToU64(100),
				},
			},
			wantErr: false,
		},
		{
			name: "udp6 file found, no error should come up",
			file: "testdata/fixtures/proc/net/udp6",
			want: []*netIPSocketLine{
				&netIPSocketLine{
					Sl:        1315,
					LocalAddr: net.IP{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0},
					LocalPort: 5355,
					RemAddr:   net.IP{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0},
					RemPort:   0,
					St:        7,
					TxQueue:   0,
					RxQueue:   0,
					UID:       981,
					Inode:     21040,
					Drops:     intToU64(0),
				},
				&netIPSocketLine{
					Sl:        6073,
					LocalAddr: net.IP{254, 128, 0, 0, 0, 0, 0, 0, 86, 225, 173, 255, 254, 124, 102, 9},
					LocalPort: 51073,
					RemAddr:   net.IP{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0},
					RemPort:   0,
					St:        7,
					TxQueue:   0,
					RxQueue:   0,
					UID:       1000,
					Inode:     11337031,
					Drops:     intToU64(0),
				},
			},
			wantErr: false,
		},
		{
			name:    "error case - file not found",
			file:    "somewhere over the rainbow",
			want:    nil,
			wantErr: true,
		},
		{
			name:    "error case - parse error",
			file:    "testdata/fixtures/proc/net/udp_broken",
			want:    nil,
			wantErr: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := newNetUDP(tt.file)
			if (err != nil) != tt.wantErr {
				t.Errorf("newNetUDP() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("newNetUDP() = %v, want %v", got, tt.want)
			}
		})
	}
}

func Test_newNetUDPSummary(t *testing.T) {
	tests := []struct {
		name    string
		file    string
		want    *NetUDPSummary
		wantErr bool
	}{
		{
			name:    "udp file found, no error should come up",
			file:    "testdata/fixtures/proc/net/udp",
			want:    &NetUDPSummary{TxQueueLength: 2, RxQueueLength: 2, UsedSockets: 3, Drops: intToU64(300)},
			wantErr: false,
		},
		{
			name:    "udp6 file found, no error should come up",
			file:    "testdata/fixtures/proc/net/udp6",
			want:    &NetUDPSummary{TxQueueLength: 0, RxQueueLength: 0, UsedSockets: 2, Drops: intToU64(0)},
			wantErr: false,
		},
		{
			name:    "error case - file not found",
			file:    "somewhere over the rainbow",
			want:    nil,
			wantErr: true,
		},
		{
			name:    "error case - parse error",
			file:    "testdata/fixtures/proc/net/udp_broken",
			want:    nil,
			wantErr: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := newNetUDPSummary(tt.file)
			if (err != nil) != tt.wantErr {
				t.Errorf("newNetUDPSummary() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("newNetUDPSummary() = %v, want %v", got, tt.want)
			}
		})
	}
}

// intToU64 convert int to uint64 and return it pointer.
func intToU64(i int) *uint64 {
	cast := uint64(i)
	return &cast
}
