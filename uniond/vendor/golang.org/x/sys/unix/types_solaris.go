// Copyright 2009 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build ignore

/*
Input to cgo -godefs.  See README.md
*/

// +godefs map struct_in_addr [4]byte /* in_addr */
// +godefs map struct_in6_addr [16]byte /* in6_addr */

package unix

/*
#define KERNEL
// These defines ensure that builds done on newer versions of Solaris are
// backwards-compatible with older versions of Solaris and
// OpenSolaris-based derivatives.
#define __USE_SUNOS_SOCKETS__          // msghdr
#define __USE_LEGACY_PROTOTYPES__      // iovec
#include <dirent.h>
#include <fcntl.h>
#include <netdb.h>
#include <limits.h>
#include <poll.h>
#include <signal.h>
#include <termios.h>
#include <termio.h>
#include <stdio.h>
#include <unistd.h>
#include <sys/mman.h>
#include <sys/mount.h>
#include <sys/param.h>
#include <sys/port.h>
#include <sys/resource.h>
#include <sys/select.h>
#include <sys/signal.h>
#include <sys/socket.h>
#include <sys/sockio.h>
#include <sys/stat.h>
#include <sys/statvfs.h>
#include <sys/stropts.h>
#include <sys/time.h>
#include <sys/times.h>
#include <sys/types.h>
#include <sys/utsname.h>
#include <sys/un.h>
#include <sys/wait.h>
#include <net/bpf.h>
#include <net/if.h>
#include <net/if_dl.h>
#include <net/route.h>
#include <netinet/in.h>
#include <netinet/icmp6.h>
#include <netinet/tcp.h>
#include <ustat.h>
#include <utime.h>

enum {
	sizeofPtr = sizeof(void*),
};

union sockaddr_all {
	struct sockaddr s1;	// this one gets used for fields
	struct sockaddr_in s2;	// these pad it out
	struct sockaddr_in6 s3;
	struct sockaddr_un s4;
	struct sockaddr_dl s5;
};

struct sockaddr_any {
	struct sockaddr addr;
	char pad[sizeof(union sockaddr_all) - sizeof(struct sockaddr)];
};

// Solaris and the major illumos distributions ship a 3rd party tun/tap driver
// from https://github.com/kaizawa/tuntap
// It supports a pair of IOCTLs defined at
// https://github.com/kaizawa/tuntap/blob/master/if_tun.h#L91-L93
#define TUNNEWPPA	(('T'<<16) | 0x0001)
#define TUNSETPPA	(('T'<<16) | 0x0002)
*/
import "C"

// Machine characteristics

const (
	SizeofPtr      = C.sizeofPtr
	SizeofShort    = C.sizeof_short
	SizeofInt      = C.sizeof_int
	SizeofLong     = C.sizeof_long
	SizeofLongLong = C.sizeof_longlong
	PathMax        = C.PATH_MAX
	MaxHostNameLen = C.MAXHOSTNAMELEN
)

// Basic types

type (
	_C_short     C.short
	_C_int       C.int
	_C_long      C.long
	_C_long_long C.longlong
)

// Time

type Timespec C.struct_timespec

type Timeval C.struct_timeval

type Timeval32 C.struct_timeval32

type Tms C.struct_tms

type Utimbuf C.struct_utimbuf

// Processes

type Rusage C.struct_rusage

type Rlimit C.struct_rlimit

type _Gid_t C.gid_t

// Files

type Stat_t C.struct_stat

type Flock_t C.struct_flock

type Dirent C.struct_dirent

// Filesystems

type _Fsblkcnt_t C.fsblkcnt_t

type Statvfs_t C.struct_statvfs

// Sockets

type RawSockaddrInet4 C.struct_sockaddr_in

type RawSockaddrInet6 C.struct_sockaddr_in6

type RawSockaddrUnix C.struct_sockaddr_un

type RawSockaddrDatalink C.struct_sockaddr_dl

type RawSockaddr C.struct_sockaddr

type RawSockaddrAny C.struct_sockaddr_any

type _Socklen C.socklen_t

type Linger C.struct_linger

type Iovec C.struct_iovec

type IPMreq C.struct_ip_mreq

type IPv6Mreq C.struct_ipv6_mreq

type Msghdr C.struct_msghdr

type Cmsghdr C.struct_cmsghdr

type Inet4Pktinfo C.struct_in_pktinfo

type Inet6Pktinfo C.struct_in6_pktinfo

type IPv6MTUInfo C.struct_ip6_mtuinfo

type ICMPv6Filter C.struct_icmp6_filter

const (
	SizeofSockaddrInet4    = C.sizeof_struct_sockaddr_in
	SizeofSockaddrInet6    = C.sizeof_struct_sockaddr_in6
	SizeofSockaddrAny      = C.sizeof_struct_sockaddr_any
	SizeofSockaddrUnix     = C.sizeof_struct_sockaddr_un
	SizeofSockaddrDatalink = C.sizeof_struct_sockaddr_dl
	SizeofLinger           = C.sizeof_struct_linger
	SizeofIovec            = C.sizeof_struct_iovec
	SizeofIPMreq           = C.sizeof_struct_ip_mreq
	SizeofIPv6Mreq         = C.sizeof_struct_ipv6_mreq
	SizeofMsghdr           = C.sizeof_struct_msghdr
	SizeofCmsghdr          = C.sizeof_struct_cmsghdr
	SizeofInet4Pktinfo     = C.sizeof_struct_in_pktinfo
	SizeofInet6Pktinfo     = C.sizeof_struct_in6_pktinfo
	SizeofIPv6MTUInfo      = C.sizeof_struct_ip6_mtuinfo
	SizeofICMPv6Filter     = C.sizeof_struct_icmp6_filter
)

// Select

type FdSet C.fd_set

// Misc

type Utsname C.struct_utsname

type Ustat_t C.struct_ustat

const (
	AT_FDCWD            = C.AT_FDCWD
	AT_SYMLINK_NOFOLLOW = C.AT_SYMLINK_NOFOLLOW
	AT_SYMLINK_FOLLOW   = C.AT_SYMLINK_FOLLOW
	AT_REMOVEDIR        = C.AT_REMOVEDIR
	AT_EACCESS          = C.AT_EACCESS
)

// Routing and interface messages

const (
	SizeofIfMsghdr  = C.sizeof_struct_if_msghdr
	SizeofIfData    = C.sizeof_struct_if_data
	SizeofIfaMsghdr = C.sizeof_struct_ifa_msghdr
	SizeofRtMsghdr  = C.sizeof_struct_rt_msghdr
	SizeofRtMetrics = C.sizeof_struct_rt_metrics
)

type IfMsghdr C.struct_if_msghdr

type IfData C.struct_if_data

type IfaMsghdr C.struct_ifa_msghdr

type RtMsghdr C.struct_rt_msghdr

type RtMetrics C.struct_rt_metrics

// Berkeley packet filter

const (
	SizeofBpfVersion = C.sizeof_struct_bpf_version
	SizeofBpfStat    = C.sizeof_struct_bpf_stat
	SizeofBpfProgram = C.sizeof_struct_bpf_program
	SizeofBpfInsn    = C.sizeof_struct_bpf_insn
	SizeofBpfHdr     = C.sizeof_struct_bpf_hdr
)

type BpfVersion C.struct_bpf_version

type BpfStat C.struct_bpf_stat

type BpfProgram C.struct_bpf_program

type BpfInsn C.struct_bpf_insn

type BpfTimeval C.struct_bpf_timeval

type BpfHdr C.struct_bpf_hdr

// Terminal handling

type Termios C.struct_termios

type Termio C.struct_termio

type Winsize C.struct_winsize

// poll

type PollFd C.struct_pollfd

const (
	POLLERR    = C.POLLERR
	POLLHUP    = C.POLLHUP
	POLLIN     = C.POLLIN
	POLLNVAL   = C.POLLNVAL
	POLLOUT    = C.POLLOUT
	POLLPRI    = C.POLLPRI
	POLLRDBAND = C.POLLRDBAND
	POLLRDNORM = C.POLLRDNORM
	POLLWRBAND = C.POLLWRBAND
	POLLWRNORM = C.POLLWRNORM
)

// Event Ports

type fileObj C.struct_file_obj

type portEvent C.struct_port_event

const (
	PORT_SOURCE_AIO    = C.PORT_SOURCE_AIO
	PORT_SOURCE_TIMER  = C.PORT_SOURCE_TIMER
	PORT_SOURCE_USER   = C.PORT_SOURCE_USER
	PORT_SOURCE_FD     = C.PORT_SOURCE_FD
	PORT_SOURCE_ALERT  = C.PORT_SOURCE_ALERT
	PORT_SOURCE_MQ     = C.PORT_SOURCE_MQ
	PORT_SOURCE_FILE   = C.PORT_SOURCE_FILE
	PORT_ALERT_SET     = C.PORT_ALERT_SET
	PORT_ALERT_UPDATE  = C.PORT_ALERT_UPDATE
	PORT_ALERT_INVALID = C.PORT_ALERT_INVALID
	FILE_ACCESS        = C.FILE_ACCESS
	FILE_MODIFIED      = C.FILE_MODIFIED
	FILE_ATTRIB        = C.FILE_ATTRIB
	FILE_TRUNC         = C.FILE_TRUNC
	FILE_NOFOLLOW      = C.FILE_NOFOLLOW
	FILE_DELETE        = C.FILE_DELETE
	FILE_RENAME_TO     = C.FILE_RENAME_TO
	FILE_RENAME_FROM   = C.FILE_RENAME_FROM
	UNMOUNTED          = C.UNMOUNTED
	MOUNTEDOVER        = C.MOUNTEDOVER
	FILE_EXCEPTION     = C.FILE_EXCEPTION
)

// STREAMS and Tun

const (
	TUNNEWPPA = C.TUNNEWPPA
	TUNSETPPA = C.TUNSETPPA

	// sys/stropts.h:
	I_STR     = C.I_STR
	I_POP     = C.I_POP
	I_PUSH    = C.I_PUSH
	I_LINK    = C.I_LINK
	I_UNLINK  = C.I_UNLINK
	I_PLINK   = C.I_PLINK
	I_PUNLINK = C.I_PUNLINK

	// sys/sockio.h:
	IF_UNITSEL = C.IF_UNITSEL
)

type strbuf C.struct_strbuf

type Strioctl C.struct_strioctl

type Lifreq C.struct_lifreq
