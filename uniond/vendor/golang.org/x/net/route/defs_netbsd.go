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
*/
import "C"

const (
	sizeofIfMsghdrNetBSD7         = C.sizeof_struct_if_msghdr
	sizeofIfaMsghdrNetBSD7        = C.sizeof_struct_ifa_msghdr
	sizeofIfAnnouncemsghdrNetBSD7 = C.sizeof_struct_if_announcemsghdr

	sizeofRtMsghdrNetBSD7  = C.sizeof_struct_rt_msghdr
	sizeofRtMetricsNetBSD7 = C.sizeof_struct_rt_metrics

	sizeofSockaddrStorage = C.sizeof_struct_sockaddr_storage
	sizeofSockaddrInet    = C.sizeof_struct_sockaddr_in
	sizeofSockaddrInet6   = C.sizeof_struct_sockaddr_in6
)
