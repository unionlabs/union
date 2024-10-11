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
	sizeofIfMsghdrDarwin15    = C.sizeof_struct_if_msghdr
	sizeofIfaMsghdrDarwin15   = C.sizeof_struct_ifa_msghdr
	sizeofIfmaMsghdrDarwin15  = C.sizeof_struct_ifma_msghdr
	sizeofIfMsghdr2Darwin15   = C.sizeof_struct_if_msghdr2
	sizeofIfmaMsghdr2Darwin15 = C.sizeof_struct_ifma_msghdr2
	sizeofIfDataDarwin15      = C.sizeof_struct_if_data
	sizeofIfData64Darwin15    = C.sizeof_struct_if_data64

	sizeofRtMsghdrDarwin15  = C.sizeof_struct_rt_msghdr
	sizeofRtMsghdr2Darwin15 = C.sizeof_struct_rt_msghdr2
	sizeofRtMetricsDarwin15 = C.sizeof_struct_rt_metrics

	sizeofSockaddrStorage = C.sizeof_struct_sockaddr_storage
	sizeofSockaddrInet    = C.sizeof_struct_sockaddr_in
	sizeofSockaddrInet6   = C.sizeof_struct_sockaddr_in6
)
