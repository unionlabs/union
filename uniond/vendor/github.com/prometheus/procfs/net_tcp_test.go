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

func Test_newNetTCP(t *testing.T) {
	tests := []struct {
		name    string
		file    string
		want    NetTCP
		wantErr bool
	}{
		{
			name: "tcp file found, no error should come up",
			file: "testdata/fixtures/proc/net/tcp",
			want: []*netIPSocketLine{
				{
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
				},
				{
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
				},
				{
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
				},
			},
			wantErr: false,
		},
		{
			name: "tcp6 file found, no error should come up",
			file: "testdata/fixtures/proc/net/tcp6",
			want: []*netIPSocketLine{
				{
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
				},
				{
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
			file:    "testdata/fixtures/proc/net/tcp_broken",
			want:    nil,
			wantErr: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := newNetTCP(tt.file)
			if (err != nil) != tt.wantErr {
				t.Errorf("newNetTCP() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("newNetTCP() = %v, want %v", got, tt.want)
			}
		})
	}
}

func Test_newNetTCPSummary(t *testing.T) {
	tests := []struct {
		name    string
		file    string
		want    *NetTCPSummary
		wantErr bool
	}{
		{
			name:    "tcp file found, no error should come up",
			file:    "testdata/fixtures/proc/net/tcp",
			want:    &NetTCPSummary{TxQueueLength: 2, RxQueueLength: 2, UsedSockets: 3},
			wantErr: false,
		},
		{
			name:    "tcp6 file found, no error should come up",
			file:    "testdata/fixtures/proc/net/tcp6",
			want:    &NetTCPSummary{TxQueueLength: 0, RxQueueLength: 0, UsedSockets: 2},
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
			file:    "testdata/fixtures/proc/net/tcp_broken",
			want:    nil,
			wantErr: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := newNetTCPSummary(tt.file)
			if (err != nil) != tt.wantErr {
				t.Errorf("newNetTCPSummary() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("newNetTCPSummary() = %v, want %v", got, tt.want)
			}
		})
	}
}
