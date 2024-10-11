// Copyright 2018 The Prometheus Authors
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
	"fmt"
	"reflect"
	"strings"
	"testing"
	"time"
)

func TestMountStats(t *testing.T) {
	tests := []struct {
		name    string
		s       string
		mounts  []*Mount
		invalid bool
	}{
		{
			name: "no devices",
			s:    `hello`,
		},
		{
			name:    "device has too few fields",
			s:       `device foo`,
			invalid: true,
		},
		{
			name:    "device incorrect format",
			s:       `device rootfs BAD on / with fstype rootfs`,
			invalid: true,
		},
		{
			name:    "device incorrect format",
			s:       `device rootfs mounted BAD / with fstype rootfs`,
			invalid: true,
		},
		{
			name:    "device incorrect format",
			s:       `device rootfs mounted on / BAD fstype rootfs`,
			invalid: true,
		},
		{
			name:    "device incorrect format",
			s:       `device rootfs mounted on / with BAD rootfs`,
			invalid: true,
		},
		{
			name:    "device rootfs cannot have stats",
			s:       `device rootfs mounted on / with fstype rootfs stats`,
			invalid: true,
		},
		{
			name:    "NFSv4 device with too little info",
			s:       "device 192.168.1.1:/srv mounted on /mnt/nfs with fstype nfs4 statvers=1.1\nopts:",
			invalid: true,
		},
		{
			name:    "NFSv4 device with bad bytes",
			s:       "device 192.168.1.1:/srv mounted on /mnt/nfs with fstype nfs4 statvers=1.1\nbytes: 0",
			invalid: true,
		},
		{
			name:    "NFSv4 device with bad events",
			s:       "device 192.168.1.1:/srv mounted on /mnt/nfs with fstype nfs4 statvers=1.1\nevents: 0",
			invalid: true,
		},
		{
			name:    "NFSv4 device with bad per-op stats",
			s:       "device 192.168.1.1:/srv mounted on /mnt/nfs with fstype nfs4 statvers=1.1\nper-op statistics\nFOO 0",
			invalid: true,
		},
		{
			name:    "NFSv4 device with bad transport stats",
			s:       "device 192.168.1.1:/srv mounted on /mnt/nfs with fstype nfs4 statvers=1.1\nxprt: tcp",
			invalid: true,
		},
		{
			name:    "NFSv4 device with bad transport version",
			s:       "device 192.168.1.1:/srv mounted on /mnt/nfs with fstype nfs4 statvers=foo\nxprt: tcp 0",
			invalid: true,
		},
		{
			name:    "NFSv4 device with bad transport stats version 1.0",
			s:       "device 192.168.1.1:/srv mounted on /mnt/nfs with fstype nfs4 statvers=1.0\nxprt: tcp 0 0 0 0 0 0 0 0 0 0 0 0 0",
			invalid: true,
		},
		{
			name:    "NFSv4 device with bad transport stats version 1.1",
			s:       "device 192.168.1.1:/srv mounted on /mnt/nfs with fstype nfs4 statvers=1.1\nxprt: tcp 0 0 0 0 0 0 0 0 0 0",
			invalid: true,
		},
		{
			name:    "NFSv3 device with bad transport protocol",
			s:       "device 192.168.1.1:/srv mounted on /mnt/nfs with fstype nfs4 statvers=1.1\nxprt: tcpx 0 0 0 0 0 0 0 0 0 0",
			invalid: true,
		},
		{
			name: "NFSv3 device using TCP with transport stats version 1.0 OK",
			s:    "device 192.168.1.1:/srv mounted on /mnt/nfs with fstype nfs statvers=1.0\nxprt: tcp 1 2 3 4 5 6 7 8 9 10",
			mounts: []*Mount{{
				Device: "192.168.1.1:/srv",
				Mount:  "/mnt/nfs",
				Type:   "nfs",
				Stats: &MountStatsNFS{
					StatVersion: "1.0",
					Transport: []NFSTransportStats{
						{
							Protocol:                 "tcp",
							Port:                     1,
							Bind:                     2,
							Connect:                  3,
							ConnectIdleTime:          4,
							IdleTimeSeconds:          5,
							Sends:                    6,
							Receives:                 7,
							BadTransactionIDs:        8,
							CumulativeActiveRequests: 9,
							CumulativeBacklog:        10,
							MaximumRPCSlotsUsed:      0, // these three are not
							CumulativeSendingQueue:   0, // present in statvers=1.0
							CumulativePendingQueue:   0, //
						},
					},
				},
			}},
		},
		{
			name: "NFSv3 device using UDP with transport stats version 1.0 OK",
			s:    "device 192.168.1.1:/srv mounted on /mnt/nfs with fstype nfs statvers=1.0\nxprt: udp 1 2 3 4 5 6 7",
			mounts: []*Mount{{
				Device: "192.168.1.1:/srv",
				Mount:  "/mnt/nfs",
				Type:   "nfs",
				Stats: &MountStatsNFS{
					StatVersion: "1.0",
					Transport: []NFSTransportStats{
						{
							Protocol:                 "udp",
							Port:                     1,
							Bind:                     2,
							Connect:                  0,
							ConnectIdleTime:          0,
							IdleTimeSeconds:          0,
							Sends:                    3,
							Receives:                 4,
							BadTransactionIDs:        5,
							CumulativeActiveRequests: 6,
							CumulativeBacklog:        7,
							MaximumRPCSlotsUsed:      0, // these three are not
							CumulativeSendingQueue:   0, // present in statvers=1.0
							CumulativePendingQueue:   0, //
						},
					},
				},
			}},
		},
		{
			name: "NFSv3 device using TCP with transport stats version 1.1 OK",
			s:    "device 192.168.1.1:/srv mounted on /mnt/nfs with fstype nfs statvers=1.1\nxprt: tcp 1 2 3 4 5 6 7 8 9 10 11 12 13",
			mounts: []*Mount{{
				Device: "192.168.1.1:/srv",
				Mount:  "/mnt/nfs",
				Type:   "nfs",
				Stats: &MountStatsNFS{
					StatVersion: "1.1",
					Transport: []NFSTransportStats{
						{
							Protocol:                 "tcp",
							Port:                     1,
							Bind:                     2,
							Connect:                  3,
							ConnectIdleTime:          4,
							IdleTimeSeconds:          5,
							Sends:                    6,
							Receives:                 7,
							BadTransactionIDs:        8,
							CumulativeActiveRequests: 9,
							CumulativeBacklog:        10,
							MaximumRPCSlotsUsed:      11,
							CumulativeSendingQueue:   12,
							CumulativePendingQueue:   13,
						},
					},
				},
			}},
		},
		{
			name: "NFSv3 device using UDP with transport stats version 1.1 OK",
			s:    "device 192.168.1.1:/srv mounted on /mnt/nfs with fstype nfs statvers=1.1\nxprt: udp 1 2 3 4 5 6 7 8 9 10",
			mounts: []*Mount{{
				Device: "192.168.1.1:/srv",
				Mount:  "/mnt/nfs",
				Type:   "nfs",
				Stats: &MountStatsNFS{
					StatVersion: "1.1",
					Transport: []NFSTransportStats{
						{
							Protocol:                 "udp",
							Port:                     1,
							Bind:                     2,
							Connect:                  0, // these three are not
							ConnectIdleTime:          0, // present for UDP
							IdleTimeSeconds:          0, //
							Sends:                    3,
							Receives:                 4,
							BadTransactionIDs:        5,
							CumulativeActiveRequests: 6,
							CumulativeBacklog:        7,
							MaximumRPCSlotsUsed:      8,
							CumulativeSendingQueue:   9,
							CumulativePendingQueue:   10,
						},
					},
				},
			}},
		},
		{
			name: "NFSv3 device with mountaddr OK",
			s:    "device 192.168.1.1:/srv mounted on /mnt/nfs with fstype nfs statvers=1.1\nopts: rw,vers=3,mountaddr=192.168.1.1,proto=udp\n",
			mounts: []*Mount{{
				Device: "192.168.1.1:/srv",
				Mount:  "/mnt/nfs",
				Type:   "nfs",
				Stats: &MountStatsNFS{
					StatVersion: "1.1",
					Opts:        map[string]string{"rw": "", "vers": "3", "mountaddr": "192.168.1.1", "proto": "udp"},
				},
			}},
		},
		{
			name: "device rootfs OK",
			s:    `device rootfs mounted on / with fstype rootfs`,
			mounts: []*Mount{{
				Device: "rootfs",
				Mount:  "/",
				Type:   "rootfs",
			}},
		},
		{
			name: "NFSv3 device with minimal stats OK",
			s:    `device 192.168.1.1:/srv mounted on /mnt/nfs with fstype nfs statvers=1.1`,
			mounts: []*Mount{{
				Device: "192.168.1.1:/srv",
				Mount:  "/mnt/nfs",
				Type:   "nfs",
				Stats: &MountStatsNFS{
					StatVersion: "1.1",
				},
			}},
		},
		{
			name: "NFS4.1 device with multiline impl_id OK",
			s:    "device 192.168.0.1:/srv mounted on /mnt/nfs with fstype nfs4 statvers=1.1\nopts: rw,vers=4.1,rsize=131072,wsize=131072,namlen=255,acregmin=3,acregmax=60,acdirmin=30,acdirmax=60,hard,proto=tcp,timeo=600,retrans=2,sec=sys,clientaddr=192.168.0.2,fsc,local_lock=none\nage: 1234567\nimpl_id: name='FreeBSD 11.2-STABLE #0 r325575+c9231c7d6bd(HEAD): Mon Nov 18 22:46:47 UTC 2019\nuser@host:/path/to/something'\n,domain='something.org',date='1293840000,0'",
			mounts: []*Mount{{
				Device: "192.168.0.1:/srv",
				Mount:  "/mnt/nfs",
				Type:   "nfs4",
				Stats: &MountStatsNFS{
					StatVersion: "1.1",
					Opts: map[string]string{"rw": "", "vers": "4.1",
						"rsize": "131072", "wsize": "131072", "namlen": "255", "acregmin": "3",
						"acregmax": "60", "acdirmin": "30", "acdirmax": "60", "fsc": "", "hard": "",
						"proto": "tcp", "timeo": "600", "retrans": "2",
						"sec": "sys", "clientaddr": "192.168.0.2",
						"local_lock": "none",
					},
					Age: 1234567 * time.Second,
				},
			}},
		},
		{
			name: "fixtures/proc OK",
			mounts: []*Mount{
				{
					Device: "rootfs",
					Mount:  "/",
					Type:   "rootfs",
				},
				{
					Device: "sysfs",
					Mount:  "/sys",
					Type:   "sysfs",
				},
				{
					Device: "proc",
					Mount:  "/proc",
					Type:   "proc",
				},
				{
					Device: "/dev/sda1",
					Mount:  "/",
					Type:   "ext4",
				},
				{
					Device: "192.168.1.1:/srv/test",
					Mount:  "/mnt/nfs/test",
					Type:   "nfs4",
					Stats: &MountStatsNFS{
						StatVersion: "1.1",
						Opts: map[string]string{"rw": "", "vers": "4.0",
							"rsize": "1048576", "wsize": "1048576", "namlen": "255", "acregmin": "3",
							"acregmax": "60", "acdirmin": "30", "acdirmax": "60", "hard": "",
							"proto": "tcp", "port": "0", "timeo": "600", "retrans": "2",
							"sec": "sys", "mountaddr": "192.168.1.1", "clientaddr": "192.168.1.5",
							"local_lock": "none",
						},
						Age: 13968 * time.Second,
						Bytes: NFSBytesStats{
							Read:      1207640230,
							ReadTotal: 1210214218,
							ReadPages: 295483,
						},
						Events: NFSEventsStats{
							InodeRevalidate: 52,
							DnodeRevalidate: 226,
							VFSOpen:         1,
							VFSLookup:       13,
							VFSAccess:       398,
							VFSReadPages:    331,
							VFSWritePages:   47,
							VFSFlush:        77,
							VFSFileRelease:  77,
						},
						Operations: []NFSOperationStats{
							{
								Operation: "NULL",
							},
							{
								Operation:                           "READ",
								Requests:                            1298,
								Transmissions:                       1298,
								BytesSent:                           207680,
								BytesReceived:                       1210292152,
								CumulativeQueueMilliseconds:         6,
								CumulativeTotalResponseMilliseconds: 79386,
								CumulativeTotalRequestMilliseconds:  79407,
							},
							{
								Operation: "WRITE",
							},
							{
								Operation:                           "ACCESS",
								Requests:                            2927395007,
								Transmissions:                       2927394995,
								BytesSent:                           526931094212,
								BytesReceived:                       362996810236,
								CumulativeQueueMilliseconds:         18446743919241604546,
								CumulativeTotalResponseMilliseconds: 1667369447,
								CumulativeTotalRequestMilliseconds:  1953587717,
							},
						},
						Transport: []NFSTransportStats{
							{
								Protocol:                 "tcp",
								Port:                     832,
								Connect:                  1,
								IdleTimeSeconds:          11,
								Sends:                    6428,
								Receives:                 6428,
								CumulativeActiveRequests: 12154,
								MaximumRPCSlotsUsed:      24,
								CumulativeSendingQueue:   26,
								CumulativePendingQueue:   5726,
							},
						},
					},
				},
			},
		},
		{
			name: "NFS xprt over rdma proto",
			s: `device <nfsserver>:<nfsmount> mounted on <mountpoint> with fstype nfs statvers=1.1
        opts:   ro,vers=3,rsize=1048576,wsize=1048576,namlen=255,acregmin=120,acregmax=120,acdirmin=120,acdirmax=120,hard,nocto,forcerdirplus,proto=rdma,nconnect=16,port=20049,timeo=600,retrans=2,sec=sys,mountaddr=172.16.40.20,mountvers=3,mountport=0,mountproto=tcp,local_lock=none
        age:    1270876
        caps:   caps=0xf,wtmult=4096,dtsize=131072,bsize=0,namlen=255
        sec:    flavor=1,pseudoflavor=1
        events: 512052 36601115 0 68 1498583 16514 38815015 0 41584 2654459933 0 0 0 0 1527715 0 0 1498575 0 0 0 0 0 0 0 0 0
        bytes:  3104202770327296 0 0 0 2013200952170479 0 491504202537 0
        RPC iostats version: 1.1  p/v: 100003/3 (nfs)
        xprt:   rdma 0 0 5808 62 0 494490723 494490687 36 10032963746 1282789 107150285 1226637531 2673889 135120843409861 135119397156505 266368832 75716996 0 7853 0 0 0 0 119328 1336431717 0 96
        per-op statistics
                NULL: 16 16 0 640 384 320 11 331 0
`,
			mounts: []*Mount{{
				Device: "<nfsserver>:<nfsmount>",
				Mount:  "<mountpoint>",
				Type:   "nfs",
				Stats: &MountStatsNFS{
					StatVersion: "1.1",
					Opts: map[string]string{"acdirmax": "120", "acdirmin": "120", "acregmax": "120",
						"acregmin": "120", "forcerdirplus": "", "hard": "", "local_lock": "none",
						"mountaddr": "172.16.40.20", "mountport": "0", "mountproto": "tcp", "mountvers": "3",
						"namlen": "255", "nconnect": "16", "nocto": "", "port": "20049", "proto": "rdma",
						"retrans": "2", "ro": "", "rsize": "1048576", "sec": "sys", "timeo": "600",
						"vers": "3", "wsize": "1048576"},
					Age: 1270876 * time.Second,
					Bytes: NFSBytesStats{
						Read:      3104202770327296,
						ReadTotal: 2013200952170479,
						ReadPages: 491504202537,
					},
					Events: NFSEventsStats{
						InodeRevalidate:     512052,
						DnodeRevalidate:     36601115,
						AttributeInvalidate: 68,
						VFSOpen:             1498583,
						VFSLookup:           16514,
						VFSAccess:           38815015,
						VFSReadPage:         41584,
						VFSReadPages:        2654459933,
						VFSFlush:            1527715,
						VFSFileRelease:      1498575,
					},
					Operations: []NFSOperationStats{
						{
							Operation:                           "NULL",
							Requests:                            16,
							Transmissions:                       16,
							MajorTimeouts:                       0,
							BytesSent:                           640,
							BytesReceived:                       384,
							CumulativeQueueMilliseconds:         320,
							CumulativeTotalResponseMilliseconds: 11,
							CumulativeTotalRequestMilliseconds:  331,
							Errors:                              0,
						},
					},
					Transport: []NFSTransportStats{
						{
							Protocol:                 "rdma",
							Port:                     0,
							Bind:                     0,
							Connect:                  5808,
							ConnectIdleTime:          62,
							IdleTimeSeconds:          0,
							Sends:                    494490723,
							Receives:                 494490687,
							BadTransactionIDs:        36,
							CumulativeActiveRequests: 10032963746,
							CumulativeBacklog:        1282789,
							MaximumRPCSlotsUsed:      0,
							CumulativeSendingQueue:   0,
							CumulativePendingQueue:   0,
							ReadChunkCount:           107150285,
							WriteChunkCount:          1226637531,
							ReplyChunkCount:          2673889,
							TotalRdmaRequest:         135120843409861,
							PullupCopyCount:          135119397156505,
							HardwayRegisterCount:     266368832,
							FailedMarshalCount:       75716996,
							BadReplyCount:            0,
							MrsRecovered:             7853,
							MrsOrphaned:              0,
							MrsAllocated:             0,
							EmptySendctxQ:            0,
							TotalRdmaReply:           0,
							FixupCopyCount:           119328,
							ReplyWaitsForSend:        1336431717,
							LocalInvNeeded:           0,
							NomsgCallCount:           96,
							BcallCount:               0,
						},
					},
				},
			}},
		},
	}

	for i, tt := range tests {
		t.Logf("[%02d] test %q", i, tt.name)

		var mounts []*Mount
		var err error

		if tt.s != "" {
			mounts, err = parseMountStats(strings.NewReader(tt.s))
		} else {
			proc, e := getProcFixtures(t).Proc(26231)
			if e != nil {
				t.Fatalf("failed to create proc: %v", err)
			}

			mounts, err = proc.MountStats()
		}

		if tt.invalid && err == nil {
			t.Error("expected an error, but none occurred")
		}
		if !tt.invalid && err != nil {
			t.Errorf("unexpected error: %v", err)
		}

		if want, have := tt.mounts, mounts; !reflect.DeepEqual(want, have) {
			t.Errorf("mounts:\nwant:\n%v\nhave:\n%v", mountsStr(want), mountsStr(have))
		}
	}
}

func mountsStr(mounts []*Mount) string {
	var out string
	for i, m := range mounts {
		out += fmt.Sprintf("[%d] %q on %q (%q)", i, m.Device, m.Mount, m.Type)

		stats, ok := m.Stats.(*MountStatsNFS)
		if !ok {
			out += "\n"
			continue
		}

		out += fmt.Sprintf("\n\t- opts: %s", stats.Opts)
		out += fmt.Sprintf("\n\t- v%s, age: %s", stats.StatVersion, stats.Age)
		out += fmt.Sprintf("\n\t- bytes: %v", stats.Bytes)
		out += fmt.Sprintf("\n\t- events: %v", stats.Events)
		out += fmt.Sprintf("\n\t- transport: %v", stats.Transport)
		out += "\n\t- per-operation stats:"

		for _, o := range stats.Operations {
			out += fmt.Sprintf("\n\t\t- %v", o)
		}

		out += "\n"
	}

	return out
}

func TestMountStatsExtendedOperationStats(t *testing.T) {
	r := strings.NewReader(extendedOpsExampleMountstats)
	_, err := parseMountStats(r)
	if err != nil {
		t.Errorf("failed to parse mount stats with extended per-op statistics: %v", err)
	}
}

const (
	extendedOpsExampleMountstats = `
device fs.example.com:/volume4/apps/home-automation/node-red-data mounted on /var/lib/kubelet/pods/1c2215a7-0d92-4df5-83ce-a807bcc2f8c8/volumes/kubernetes.io~nfs/home-automation--node-red-data--pv0001 with fstype nfs4 statvers=1.1
	opts:   rw,vers=4.1,rsize=131072,wsize=131072,namlen=255,acregmin=3,acregmax=60,acdirmin=30,acdirmax=60,hard,proto=tcp,timeo=600,retrans=2,sec=sys,clientaddr=192.168.1.191,local_lock=none
	age:    83520
	impl_id:        name='',domain='',date='0,0'
	caps:   caps=0x3fff7,wtmult=512,dtsize=32768,bsize=0,namlen=255
	nfsv4:  bm0=0xfdffafff,bm1=0xf9be3e,bm2=0x800,acl=0x0,sessions,pnfs=not configured,lease_time=90,lease_expired=0
	sec:    flavor=1,pseudoflavor=1
	events: 52472 472680 16671 57552 2104 9565 749555 9568641 168 24103 1 267134 3350 20097 116581 18214 43757 111141 0 28 9563845 34 0 0 0 0 0 
	bytes:  2021340783 39056395530 0 0 1788561151 39087991255 442605 9557343 
	RPC iostats version: 1.1  p/v: 100003/4 (nfs)
	xprt:   tcp 940 0 2 0 1 938505 938504 0 12756069 0 32 254729 10823602
	per-op statistics
			NULL: 1 1 0 44 24 0 0 0 0
			READ: 34096 34096 0 7103096 1792122744 2272 464840 467945 0
			WRITE: 322308 322308 0 39161277084 56725504 401718334 10139998 411864389 0
			COMMIT: 12541 12541 0 2709896 1304264 342 7179 7819 0
			OPEN: 12637 12637 0 3923256 4659940 871 57185 58251 394
	OPEN_CONFIRM: 0 0 0 0 0 0 0 0 0
		OPEN_NOATTR: 98741 98741 0 25656212 31630800 3366 77710 82693 0
	OPEN_DOWNGRADE: 0 0 0 0 0 0 0 0 0
			CLOSE: 87075 87075 0 18778608 15308496 2026 49131 52399 116
			SETATTR: 24576 24576 0 5825876 6522260 643 34384 35650 0
			FSINFO: 1 1 0 168 152 0 0 0 0
			RENEW: 0 0 0 0 0 0 0 0 0
		SETCLIENTID: 0 0 0 0 0 0 0 0 0
	SETCLIENTID_CONFIRM: 0 0 0 0 0 0 0 0 0
			LOCK: 22512 22512 0 5417628 2521312 1088 17407 18794 2
			LOCKT: 0 0 0 0 0 0 0 0 0
			LOCKU: 21247 21247 0 4589352 2379664 315 8409 9003 0
			ACCESS: 1466 1466 0 298160 246288 22 1394 1492 0
			GETATTR: 52480 52480 0 10015464 12694076 2930 30069 34502 0
			LOOKUP: 11727 11727 0 2518200 2886376 272 16935 17662 3546
		LOOKUP_ROOT: 0 0 0 0 0 0 0 0 0
			REMOVE: 833 833 0 172236 95268 15 4566 4617 68
			RENAME: 11431 11431 0 3150708 1737512 211 52649 53091 0
			LINK: 1 1 0 288 292 0 0 0 0
			SYMLINK: 0 0 0 0 0 0 0 0 0
			CREATE: 77 77 0 18292 23496 0 363 371 11
		PATHCONF: 1 1 0 164 116 0 0 0 0
			STATFS: 7420 7420 0 1394960 1187200 144 4672 4975 0
		READLINK: 4 4 0 704 488 0 1 1 0
			READDIR: 1353 1353 0 304024 2902928 11 4326 4411 0
		SERVER_CAPS: 9 9 0 1548 1476 0 3 3 0
		DELEGRETURN: 232 232 0 48896 37120 811 300 1115 0
			GETACL: 0 0 0 0 0 0 0 0 0
			SETACL: 0 0 0 0 0 0 0 0 0
	FS_LOCATIONS: 0 0 0 0 0 0 0 0 0
	RELEASE_LOCKOWNER: 0 0 0 0 0 0 0 0 0
			SECINFO: 0 0 0 0 0 0 0 0 0
	FSID_PRESENT: 0 0 0 0 0 0 0 0 0
		EXCHANGE_ID: 2 2 0 464 200 0 0 0 0
	CREATE_SESSION: 1 1 0 192 124 0 0 0 0
	DESTROY_SESSION: 0 0 0 0 0 0 0 0 0
		SEQUENCE: 0 0 0 0 0 0 0 0 0
	GET_LEASE_TIME: 0 0 0 0 0 0 0 0 0
	RECLAIM_COMPLETE: 1 1 0 124 88 0 81 81 0
		LAYOUTGET: 0 0 0 0 0 0 0 0 0
	GETDEVICEINFO: 0 0 0 0 0 0 0 0 0
	LAYOUTCOMMIT: 0 0 0 0 0 0 0 0 0
	LAYOUTRETURN: 0 0 0 0 0 0 0 0 0
	SECINFO_NO_NAME: 0 0 0 0 0 0 0 0 0
	TEST_STATEID: 0 0 0 0 0 0 0 0 0
	FREE_STATEID: 10413 10413 0 1416168 916344 147 3518 3871 10413
	GETDEVICELIST: 0 0 0 0 0 0 0 0 0
	BIND_CONN_TO_SESSION: 0 0 0 0 0 0 0 0 0
	DESTROY_CLIENTID: 0 0 0 0 0 0 0 0 0
			SEEK: 0 0 0 0 0 0 0 0 0
		ALLOCATE: 0 0 0 0 0 0 0 0 0
		DEALLOCATE: 0 0 0 0 0 0 0 0 0
		LAYOUTSTATS: 0 0 0 0 0 0 0 0 0
			CLONE: 0 0 0 0 0 0 0 0 0
			COPY: 0 0 0 0 0 0 0 0 0
	OFFLOAD_CANCEL: 0 0 0 0 0 0 0 0 0
			LOOKUPP: 0 0 0 0 0 0 0 0 0
		LAYOUTERROR: 0 0 0 0 0 0 0 0 0
`

	extendedRDMAExampleALLMountstats = `device <nfsserver>:<nfsmount> mounted on <mountpoint> with fstype nfs statvers=1.1
        opts:   ro,vers=3,rsize=1048576,wsize=1048576,namlen=255,acregmin=120,acregmax=120,acdirmin=120,acdirmax=120,hard,nocto,forcerdirplus,proto=rdma,nconnect=16,port=20049,timeo=600,retrans=2,sec=sys,mountaddr=172.16.40.20,mountvers=3,mountport=0,mountproto=tcp,local_lock=none
        age:    1270876
        caps:   caps=0xf,wtmult=4096,dtsize=131072,bsize=0,namlen=255
        sec:    flavor=1,pseudoflavor=1
        events: 512052 36601115 0 68 1498583 16514 38815015 0 41584 2654459933 0 0 0 0 1527715 0 0 1498575 0 0 0 0 0 0 0 0 0
        bytes:  3104202770327296 0 0 0 2013200952170479 0 491504202537 0
        RPC iostats version: 1.1  p/v: 100003/3 (nfs)
        xprt:   rdma 0 0 5808 62 0 494490723 494490687 36 10032963746 1282789 107150285 1226637531 2673889 135120843409861 135119397156505 266368832 75716996 0 7853 0 0 0 0 119328 1336431717 0 96
        xprt:   rdma 0 0 14094 145 0 492392334 492392307 27 7078693624 2509627 105561370 1280878332 2659446 142218924010291 142217463504063 276368040 94761838 0 7610 0 0 0 0 207977 1389069860 0 103
        xprt:   rdma 0 0 16107 156 0 522755125 522755092 33 9119562599 1147699 109077860 1491898147 2566003 167152062826463 167149287506014 284931680 83011025 0 6229 0 0 0 0 221408 1603518232 0 82
        xprt:   rdma 0 0 7808 82 0 441542046 441542010 36 7226132207 2519174 111096004 955223347 2676765 105741904708009 105740125663595 275613584 80373159 0 8893 0 0 0 0 149479 1068962768 0 76
        xprt:   rdma 0 0 15018 167 0 508091827 508091764 63 19817677255 36702583 108265928 1258185459 2438516 138247436686102 138246196289594 270162080 74962306 0 13328 0 0 0 0 268433 1368837472 0 66
        xprt:   rdma 0 0 14321 149 0 530246310 530246275 35 9723190432 2392024 111099700 1494204555 2589805 166691166581904 166689567426908 289995492 85067377 0 8010 0 0 0 0 214511 1607864447 0 100
        xprt:   rdma 0 0 7863 84 0 459019689 459019642 47 11809253102 1716688 111825219 1032758664 2564226 114416685286438 114414936423706 290494252 73702102 0 6927 0 0 0 0 134453 1147121864 0 79
        xprt:   rdma 0 0 7702 84 3 497598986 497598931 55 11816221496 3924722 106922130 1382063307 2506108 153967067193941 153965665472218 286222584 84094006 0 5875 0 0 0 0 127347 1491469045 0 66
        xprt:   rdma 0 0 18341 202 0 477721151 477721073 78 15204400959 40562626 106645745 1291616653 3091375 144533696686651 144529688231163 278135800 73821525 0 6795 0 0 0 0 251097 1401327563 0 64
        xprt:   rdma 0 0 8228 90 4 453155092 453155063 29 7884786894 1591225 112197590 1026006338 2742688 114591819605673 114590175821191 275541944 85857259 0 7487 0 0 0 0 143044 1140917892 0 76
        xprt:   rdma 0 0 7843 83 0 446480377 446480324 53 12267986428 2958997 111971246 963162784 2693433 107176282309753 107174637802555 290269096 101100410 0 7825 0 0 0 0 141735 1077797328 0 83
        xprt:   rdma 0 0 7582 86 0 423315608 423315567 41 10197484604 2076993 109207538 785978455 2650354 86090211449474 86088475571312 279912524 87676008 0 7491 0 0 0 0 137533 897807641 0 101
        xprt:   rdma 0 0 7767 84 0 482538465 482538424 41 8935200479 1344778 112200583 1192341640 2644896 132860698423762 132858881459050 273354060 75337030 0 5941 0 0 0 0 127842 1307164736 0 97
        xprt:   rdma 0 0 14526 148 2 537745063 537745007 56 20756072620 3970332320 109539564 1363647371 2503250 148793734936250 148791264145401 291888720 90344151 0 7471 0 0 0 0 211057 1475661285 0 82
        xprt:   rdma 0 0 14300 151 0 495357347 495357316 31 8703101643 1451809 112315311 1303804607 2620502 145680743007170 145678880292235 288046696 98018259 0 7241 0 0 0 0 209396 1418712657 0 139
        xprt:   rdma 0 0 7700 82 0 466611083 466611050 33 8540498291 4082864 114740300 1059770596 2523155 117376668239921 117375375683167 260927576 78437075 0 6691 0 0 0 0 130878 1177008175 1 76
        per-op statistics
                NULL: 16 16 0 640 384 320 11 331 0
             GETATTR: 512052 512052 0 79823516 57349824 107131 612667 751847 0
             SETATTR: 0 0 0 0 0 0 0 0 0
              LOOKUP: 16713 16713 0 3040536 3706344 560 17488 20232 346
              ACCESS: 211705 211705 0 33860920 25404600 37059 229754 283822 0
            READLINK: 0 0 0 0 0 0 0 0 0
                READ: 2654501510 2654501510 0 445911966900 2013540728551504 6347457114 31407021389 37927280438 0
               WRITE: 0 0 0 0 0 0 0 0 0
              CREATE: 0 0 0 0 0 0 0 0 0
               MKDIR: 0 0 0 0 0 0 0 0 0
             SYMLINK: 0 0 0 0 0 0 0 0 0
               MKNOD: 0 0 0 0 0 0 0 0 0
              REMOVE: 0 0 0 0 0 0 0 0 0
               RMDIR: 0 0 0 0 0 0 0 0 0
              RENAME: 0 0 0 0 0 0 0 0 0
                LINK: 0 0 0 0 0 0 0 0 0
             READDIR: 0 0 0 0 0 0 0 0 0
         READDIRPLUS: 0 0 0 0 0 0 0 0 0
              FSSTAT: 56356 56356 0 6243572 9467808 82068 74356 159001 0
              FSINFO: 2 2 0 184 328 0 0 0 0
            PATHCONF: 1 1 0 92 140 0 0 0 0
              COMMIT: 0 0 0 0 0 0 0 0 0
`
)

func TestMountStatsExtendedRDMAStats(t *testing.T) {
	r := strings.NewReader(extendedRDMAExampleALLMountstats)
	_, err := parseMountStats(r)
	if err != nil {
		t.Errorf("failed to parse mount stats with extended RDMA statistics: %v", err)
	}
}
