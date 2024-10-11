// Copyright 2019 The Prometheus Authors
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

//go:build linux
// +build linux

package sysfs

import (
	"testing"

	"github.com/google/go-cmp/cmp"
)

func TestFibreChannelClass(t *testing.T) {
	fs, err := NewFS(sysTestFixtures)
	if err != nil {
		t.Fatal(err)
	}

	got, err := fs.FibreChannelClass()
	if err != nil {
		t.Fatal(err)
	}

	// "host0" FibreChannelHost constants.
	var (
		name0             = "host0"
		speed0            = "16 Gbit"
		portState0        = "Online"
		portType0         = "Point-To-Point (direct nport connection)"
		portName0         = "1000e0071bce95f2"
		symbolicName0     = "Emulex SN1100E2P FV12.4.270.3 DV12.4.0.0. HN:gotest. OS:Linux"
		nodeName0         = "2000e0071bce95f2"
		portID0           = "000002"
		fabricName0       = "0"
		devLossTMO0       = "30"
		supportedClasses0 = "Class 3"
		supportedSpeeds0  = "4 Gbit, 8 Gbit, 16 Gbit"

		// "host0" FibreChannelHost.Counters constants.
		dumpedFrames0          = ^uint64(0)
		errorFrames0           = uint64(0)
		invalidCRCCount0       = uint64(0x2)
		rxFrames0              = uint64(0x3)
		rxWords0               = uint64(0x4)
		txFrames0              = uint64(0x5)
		txWords0               = uint64(0x6)
		secondsSinceLastReset0 = uint64(0x7)
		invalidTXWordCount0    = uint64(0x8)
		linkFailureCount0      = uint64(0x9)
		lossOfSyncCount0       = uint64(0x10)
		lossOfSignalCount0     = uint64(0x11)
		nosCount0              = uint64(0x12)
		fcpPacketAborts0       = uint64(0x13)
	)

	// "host1" FibreChannelHost constants.
	var (
		name1      = "host1"
		portState1 = "Online"

		// "host1" FibreChannelHost.Counters constants.
		dumpedFrames1          = uint64(0)
		errorFrames1           = ^uint64(0)
		invalidCRCCount1       = uint64(0x20)
		rxFrames1              = uint64(0x30)
		rxWords1               = uint64(0x40)
		txFrames1              = uint64(0x50)
		txWords1               = uint64(0x60)
		secondsSinceLastReset1 = uint64(0x70)
		invalidTXWordCount1    = uint64(0x80)
		linkFailureCount1      = uint64(0x90)
		lossOfSyncCount1       = uint64(0x100)
		lossOfSignalCount1     = uint64(0x110)
		nosCount1              = uint64(0x120)
		fcpPacketAborts1       = uint64(0x130)
	)

	want := FibreChannelClass{
		"host0": FibreChannelHost{
			Name:             &name0,
			Speed:            &speed0,
			PortState:        &portState0,
			PortType:         &portType0,
			PortName:         &portName0,
			SymbolicName:     &symbolicName0,
			NodeName:         &nodeName0,
			PortID:           &portID0,
			FabricName:       &fabricName0,
			DevLossTMO:       &devLossTMO0,
			SupportedClasses: &supportedClasses0,
			SupportedSpeeds:  &supportedSpeeds0,
			Counters: &FibreChannelCounters{
				DumpedFrames:          &dumpedFrames0,
				ErrorFrames:           &errorFrames0,
				InvalidCRCCount:       &invalidCRCCount0,
				RXFrames:              &rxFrames0,
				RXWords:               &rxWords0,
				TXFrames:              &txFrames0,
				TXWords:               &txWords0,
				SecondsSinceLastReset: &secondsSinceLastReset0,
				InvalidTXWordCount:    &invalidTXWordCount0,
				LinkFailureCount:      &linkFailureCount0,
				LossOfSyncCount:       &lossOfSyncCount0,
				LossOfSignalCount:     &lossOfSignalCount0,
				NosCount:              &nosCount0,
				FCPPacketAborts:       &fcpPacketAborts0,
			},
		},
		"host1": FibreChannelHost{
			Name:      &name1,
			PortState: &portState1,
			Counters: &FibreChannelCounters{
				DumpedFrames:          &dumpedFrames1,
				ErrorFrames:           &errorFrames1,
				InvalidCRCCount:       &invalidCRCCount1,
				RXFrames:              &rxFrames1,
				RXWords:               &rxWords1,
				TXFrames:              &txFrames1,
				TXWords:               &txWords1,
				SecondsSinceLastReset: &secondsSinceLastReset1,
				InvalidTXWordCount:    &invalidTXWordCount1,
				LinkFailureCount:      &linkFailureCount1,
				LossOfSyncCount:       &lossOfSyncCount1,
				LossOfSignalCount:     &lossOfSignalCount1,
				NosCount:              &nosCount1,
				FCPPacketAborts:       &fcpPacketAborts1,
			},
		},
	}

	if diff := cmp.Diff(want, got); diff != "" {
		t.Fatalf("unexpected FibreChannel class (-want +got):\n%s", diff)
	}
}
