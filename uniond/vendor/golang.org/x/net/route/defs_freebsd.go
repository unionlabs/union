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

struct if_data_freebsd7 {
	u_char ifi_type;
	u_char ifi_physical;
	u_char ifi_addrlen;
	u_char ifi_hdrlen;
	u_char ifi_link_state;
	u_char ifi_spare_char1;
	u_char ifi_spare_char2;
	u_char ifi_datalen;
	u_long ifi_mtu;
	u_long ifi_metric;
	u_long ifi_baudrate;
	u_long ifi_ipackets;
	u_long ifi_ierrors;
	u_long ifi_opackets;
	u_long ifi_oerrors;
	u_long ifi_collisions;
	u_long ifi_ibytes;
	u_long ifi_obytes;
	u_long ifi_imcasts;
	u_long ifi_omcasts;
	u_long ifi_iqdrops;
	u_long ifi_noproto;
	u_long ifi_hwassist;
	time_t __ifi_epoch;
	struct timeval __ifi_lastchange;
};

struct if_data_freebsd8 {
	u_char ifi_type;
	u_char ifi_physical;
	u_char ifi_addrlen;
	u_char ifi_hdrlen;
	u_char ifi_link_state;
	u_char ifi_spare_char1;
	u_char ifi_spare_char2;
	u_char ifi_datalen;
	u_long ifi_mtu;
	u_long ifi_metric;
	u_long ifi_baudrate;
	u_long ifi_ipackets;
	u_long ifi_ierrors;
	u_long ifi_opackets;
	u_long ifi_oerrors;
	u_long ifi_collisions;
	u_long ifi_ibytes;
	u_long ifi_obytes;
	u_long ifi_imcasts;
	u_long ifi_omcasts;
	u_long ifi_iqdrops;
	u_long ifi_noproto;
	u_long ifi_hwassist;
	time_t __ifi_epoch;
	struct timeval __ifi_lastchange;
};

struct if_data_freebsd9 {
	u_char ifi_type;
	u_char ifi_physical;
	u_char ifi_addrlen;
	u_char ifi_hdrlen;
	u_char ifi_link_state;
	u_char ifi_spare_char1;
	u_char ifi_spare_char2;
	u_char ifi_datalen;
	u_long ifi_mtu;
	u_long ifi_metric;
	u_long ifi_baudrate;
	u_long ifi_ipackets;
	u_long ifi_ierrors;
	u_long ifi_opackets;
	u_long ifi_oerrors;
	u_long ifi_collisions;
	u_long ifi_ibytes;
	u_long ifi_obytes;
	u_long ifi_imcasts;
	u_long ifi_omcasts;
	u_long ifi_iqdrops;
	u_long ifi_noproto;
	u_long ifi_hwassist;
	time_t __ifi_epoch;
	struct timeval __ifi_lastchange;
};

struct if_data_freebsd10 {
	u_char ifi_type;
	u_char ifi_physical;
	u_char ifi_addrlen;
	u_char ifi_hdrlen;
	u_char ifi_link_state;
	u_char ifi_vhid;
	u_char ifi_baudrate_pf;
	u_char ifi_datalen;
	u_long ifi_mtu;
	u_long ifi_metric;
	u_long ifi_baudrate;
	u_long ifi_ipackets;
	u_long ifi_ierrors;
	u_long ifi_opackets;
	u_long ifi_oerrors;
	u_long ifi_collisions;
	u_long ifi_ibytes;
	u_long ifi_obytes;
	u_long ifi_imcasts;
	u_long ifi_omcasts;
	u_long ifi_iqdrops;
	u_long ifi_noproto;
	uint64_t ifi_hwassist;
	time_t __ifi_epoch;
	struct timeval __ifi_lastchange;
};

struct if_data_freebsd11 {
	uint8_t ifi_type;
	uint8_t ifi_physical;
	uint8_t ifi_addrlen;
	uint8_t ifi_hdrlen;
	uint8_t ifi_link_state;
	uint8_t ifi_vhid;
	uint16_t ifi_datalen;
	uint32_t ifi_mtu;
	uint32_t ifi_metric;
	uint64_t ifi_baudrate;
	uint64_t ifi_ipackets;
	uint64_t ifi_ierrors;
	uint64_t ifi_opackets;
	uint64_t ifi_oerrors;
	uint64_t ifi_collisions;
	uint64_t ifi_ibytes;
	uint64_t ifi_obytes;
	uint64_t ifi_imcasts;
	uint64_t ifi_omcasts;
	uint64_t ifi_iqdrops;
	uint64_t ifi_oqdrops;
	uint64_t ifi_noproto;
	uint64_t ifi_hwassist;
	union {
		time_t tt;
		uint64_t ph;
	} __ifi_epoch;
	union {
		struct timeval tv;
		struct {
			uint64_t ph1;
			uint64_t ph2;
		} ph;
	} __ifi_lastchange;
};

struct if_msghdr_freebsd7 {
	u_short ifm_msglen;
	u_char ifm_version;
	u_char ifm_type;
	int ifm_addrs;
	int ifm_flags;
	u_short ifm_index;
	struct if_data_freebsd7 ifm_data;
};

struct if_msghdr_freebsd8 {
	u_short ifm_msglen;
	u_char ifm_version;
	u_char ifm_type;
	int ifm_addrs;
	int ifm_flags;
	u_short ifm_index;
	struct if_data_freebsd8 ifm_data;
};

struct if_msghdr_freebsd9 {
	u_short ifm_msglen;
	u_char ifm_version;
	u_char ifm_type;
	int ifm_addrs;
	int ifm_flags;
	u_short ifm_index;
	struct if_data_freebsd9 ifm_data;
};

struct if_msghdr_freebsd10 {
	u_short ifm_msglen;
	u_char ifm_version;
	u_char ifm_type;
	int ifm_addrs;
	int ifm_flags;
	u_short ifm_index;
	struct if_data_freebsd10 ifm_data;
};

struct if_msghdr_freebsd11 {
	u_short ifm_msglen;
	u_char ifm_version;
	u_char ifm_type;
	int ifm_addrs;
	int ifm_flags;
	u_short ifm_index;
	struct if_data_freebsd11 ifm_data;
};
*/
import "C"

const (
	sizeofIfMsghdrlFreeBSD10        = C.sizeof_struct_if_msghdrl
	sizeofIfaMsghdrFreeBSD10        = C.sizeof_struct_ifa_msghdr
	sizeofIfaMsghdrlFreeBSD10       = C.sizeof_struct_ifa_msghdrl
	sizeofIfmaMsghdrFreeBSD10       = C.sizeof_struct_ifma_msghdr
	sizeofIfAnnouncemsghdrFreeBSD10 = C.sizeof_struct_if_announcemsghdr

	sizeofRtMsghdrFreeBSD10  = C.sizeof_struct_rt_msghdr
	sizeofRtMetricsFreeBSD10 = C.sizeof_struct_rt_metrics

	sizeofIfMsghdrFreeBSD7  = C.sizeof_struct_if_msghdr_freebsd7
	sizeofIfMsghdrFreeBSD8  = C.sizeof_struct_if_msghdr_freebsd8
	sizeofIfMsghdrFreeBSD9  = C.sizeof_struct_if_msghdr_freebsd9
	sizeofIfMsghdrFreeBSD10 = C.sizeof_struct_if_msghdr_freebsd10
	sizeofIfMsghdrFreeBSD11 = C.sizeof_struct_if_msghdr_freebsd11

	sizeofIfDataFreeBSD7  = C.sizeof_struct_if_data_freebsd7
	sizeofIfDataFreeBSD8  = C.sizeof_struct_if_data_freebsd8
	sizeofIfDataFreeBSD9  = C.sizeof_struct_if_data_freebsd9
	sizeofIfDataFreeBSD10 = C.sizeof_struct_if_data_freebsd10
	sizeofIfDataFreeBSD11 = C.sizeof_struct_if_data_freebsd11

	sizeofIfMsghdrlFreeBSD10Emu        = C.sizeof_struct_if_msghdrl
	sizeofIfaMsghdrFreeBSD10Emu        = C.sizeof_struct_ifa_msghdr
	sizeofIfaMsghdrlFreeBSD10Emu       = C.sizeof_struct_ifa_msghdrl
	sizeofIfmaMsghdrFreeBSD10Emu       = C.sizeof_struct_ifma_msghdr
	sizeofIfAnnouncemsghdrFreeBSD10Emu = C.sizeof_struct_if_announcemsghdr

	sizeofRtMsghdrFreeBSD10Emu  = C.sizeof_struct_rt_msghdr
	sizeofRtMetricsFreeBSD10Emu = C.sizeof_struct_rt_metrics

	sizeofIfMsghdrFreeBSD7Emu  = C.sizeof_struct_if_msghdr_freebsd7
	sizeofIfMsghdrFreeBSD8Emu  = C.sizeof_struct_if_msghdr_freebsd8
	sizeofIfMsghdrFreeBSD9Emu  = C.sizeof_struct_if_msghdr_freebsd9
	sizeofIfMsghdrFreeBSD10Emu = C.sizeof_struct_if_msghdr_freebsd10
	sizeofIfMsghdrFreeBSD11Emu = C.sizeof_struct_if_msghdr_freebsd11

	sizeofIfDataFreeBSD7Emu  = C.sizeof_struct_if_data_freebsd7
	sizeofIfDataFreeBSD8Emu  = C.sizeof_struct_if_data_freebsd8
	sizeofIfDataFreeBSD9Emu  = C.sizeof_struct_if_data_freebsd9
	sizeofIfDataFreeBSD10Emu = C.sizeof_struct_if_data_freebsd10
	sizeofIfDataFreeBSD11Emu = C.sizeof_struct_if_data_freebsd11

	sizeofSockaddrStorage = C.sizeof_struct_sockaddr_storage
	sizeofSockaddrInet    = C.sizeof_struct_sockaddr_in
	sizeofSockaddrInet6   = C.sizeof_struct_sockaddr_in6
)
