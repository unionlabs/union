// Copyright 2016 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build ignore

// +godefs map struct_in_addr [4]byte /* in_addr */
// +godefs map struct_in6_addr [16]byte /* in6_addr */

package lif

/*
#include <sys/socket.h>
#include <sys/sockio.h>

#include <net/if.h>
#include <net/if_types.h>
*/
import "C"

type sockaddrStorage C.struct_sockaddr_storage

const (
	sysLIFC_NOXMIT          = C.LIFC_NOXMIT
	sysLIFC_EXTERNAL_SOURCE = C.LIFC_EXTERNAL_SOURCE
	sysLIFC_TEMPORARY       = C.LIFC_TEMPORARY
	sysLIFC_ALLZONES        = C.LIFC_ALLZONES
	sysLIFC_UNDER_IPMP      = C.LIFC_UNDER_IPMP
	sysLIFC_ENABLED         = C.LIFC_ENABLED
)

const (
	sizeofLifnum       = C.sizeof_struct_lifnum
	sizeofLifreq       = C.sizeof_struct_lifreq
	sizeofLifconf      = C.sizeof_struct_lifconf
	sizeofLifIfinfoReq = C.sizeof_struct_lif_ifinfo_req
)

type lifnum C.struct_lifnum

type lifreq C.struct_lifreq

type lifconf C.struct_lifconf

type lifIfinfoReq C.struct_lif_ifinfo_req
