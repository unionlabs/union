// Copyright 2016 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build ignore

package route

/*
#include <sys/socket.h>
#include <sys/sysctl.h>

#include <net/if.h>
#include <net/if_dl.h>
#include <net/route.h>

#include <netinet/in.h>

struct ifa_msghdr_dfly4 {
	u_short	ifam_msglen;
	u_char	ifam_version;
	u_char	ifam_type;
	int	ifam_addrs;
	int	ifam_flags;
	u_short	ifam_index;
	int	ifam_metric;
};

struct ifa_msghdr_dfly58 {
	u_short	ifam_msglen;
	u_char	ifam_version;
	u_char	ifam_type;
	u_short	ifam_index;
	int	ifam_flags;
	int	ifam_addrs;
	int	ifam_addrflags;
	int	ifam_metric;
};
*/
import "C"

const (
	sizeofIfMsghdrDragonFlyBSD4         = C.sizeof_struct_if_msghdr
	sizeofIfaMsghdrDragonFlyBSD4        = C.sizeof_struct_ifa_msghdr_dfly4
	sizeofIfmaMsghdrDragonFlyBSD4       = C.sizeof_struct_ifma_msghdr
	sizeofIfAnnouncemsghdrDragonFlyBSD4 = C.sizeof_struct_if_announcemsghdr

	sizeofIfaMsghdrDragonFlyBSD58 = C.sizeof_struct_ifa_msghdr_dfly58

	sizeofRtMsghdrDragonFlyBSD4  = C.sizeof_struct_rt_msghdr
	sizeofRtMetricsDragonFlyBSD4 = C.sizeof_struct_rt_metrics

	sizeofSockaddrStorage = C.sizeof_struct_sockaddr_storage
	sizeofSockaddrInet    = C.sizeof_struct_sockaddr_in
	sizeofSockaddrInet6   = C.sizeof_struct_sockaddr_in6
)
