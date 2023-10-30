//! Linux-specific definitions for linux-like values

pub type useconds_t = u32;
pub type dev_t = u64;
pub type socklen_t = u32;
pub type mode_t = u32;
pub type ino64_t = u64;
pub type off64_t = i64;
pub type blkcnt64_t = i64;
pub type rlim64_t = u64;
pub type mqd_t = ::c_int;
pub type nfds_t = ::c_ulong;
pub type nl_item = ::c_int;
pub type idtype_t = ::c_uint;
pub type loff_t = ::c_longlong;
pub type pthread_key_t = ::c_uint;
pub type pthread_spinlock_t = ::c_int;

pub type __u8 = ::c_uchar;
pub type __u16 = ::c_ushort;
pub type __s16 = ::c_short;
pub type __u32 = ::c_uint;
pub type __s32 = ::c_int;

pub type Elf32_Half = u16;
pub type Elf32_Word = u32;
pub type Elf32_Off = u32;
pub type Elf32_Addr = u32;

pub type Elf64_Half = u16;
pub type Elf64_Word = u32;
pub type Elf64_Off = u64;
pub type Elf64_Addr = u64;
pub type Elf64_Xword = u64;
pub type Elf64_Sxword = i64;

pub type Elf32_Section = u16;
pub type Elf64_Section = u16;

// linux/can.h
pub type canid_t = u32;

// linux/can/j1939.h
pub type can_err_mask_t = u32;
pub type pgn_t = u32;
pub type priority_t = u8;
pub type name_t = u64;

pub type iconv_t = *mut ::c_void;

// linux/sctp.h
pub type sctp_assoc_t = ::__s32;

#[cfg_attr(feature = "extra_traits", derive(Debug))]
pub enum fpos64_t {} // FIXME: fill this out with a struct
impl ::Copy for fpos64_t {}
impl ::Clone for fpos64_t {
    fn clone(&self) -> fpos64_t {
        *self
    }
}

s! {
    pub struct glob_t {
        pub gl_pathc: ::size_t,
        pub gl_pathv: *mut *mut c_char,
        pub gl_offs: ::size_t,
        pub gl_flags: ::c_int,

        __unused1: *mut ::c_void,
        __unused2: *mut ::c_void,
        __unused3: *mut ::c_void,
        __unused4: *mut ::c_void,
        __unused5: *mut ::c_void,
    }

    pub struct passwd {
        pub pw_name: *mut ::c_char,
        pub pw_passwd: *mut ::c_char,
        pub pw_uid: ::uid_t,
        pub pw_gid: ::gid_t,
        pub pw_gecos: *mut ::c_char,
        pub pw_dir: *mut ::c_char,
        pub pw_shell: *mut ::c_char,
    }

    pub struct spwd {
        pub sp_namp: *mut ::c_char,
        pub sp_pwdp: *mut ::c_char,
        pub sp_lstchg: ::c_long,
        pub sp_min: ::c_long,
        pub sp_max: ::c_long,
        pub sp_warn: ::c_long,
        pub sp_inact: ::c_long,
        pub sp_expire: ::c_long,
        pub sp_flag: ::c_ulong,
    }

    pub struct dqblk {
        pub dqb_bhardlimit: u64,
        pub dqb_bsoftlimit: u64,
        pub dqb_curspace: u64,
        pub dqb_ihardlimit: u64,
        pub dqb_isoftlimit: u64,
        pub dqb_curinodes: u64,
        pub dqb_btime: u64,
        pub dqb_itime: u64,
        pub dqb_valid: u32,
    }

    pub struct signalfd_siginfo {
        pub ssi_signo: u32,
        pub ssi_errno: i32,
        pub ssi_code: i32,
        pub ssi_pid: u32,
        pub ssi_uid: u32,
        pub ssi_fd: i32,
        pub ssi_tid: u32,
        pub ssi_band: u32,
        pub ssi_overrun: u32,
        pub ssi_trapno: u32,
        pub ssi_status: i32,
        pub ssi_int: i32,
        pub ssi_ptr: u64,
        pub ssi_utime: u64,
        pub ssi_stime: u64,
        pub ssi_addr: u64,
        pub ssi_addr_lsb: u16,
        _pad2: u16,
        pub ssi_syscall: i32,
        pub ssi_call_addr: u64,
        pub ssi_arch: u32,
        _pad: [u8; 28],
    }

    pub struct itimerspec {
        pub it_interval: ::timespec,
        pub it_value: ::timespec,
    }

    pub struct fsid_t {
        __val: [::c_int; 2],
    }

    pub struct packet_mreq {
        pub mr_ifindex: ::c_int,
        pub mr_type: ::c_ushort,
        pub mr_alen: ::c_ushort,
        pub mr_address: [::c_uchar; 8],
    }

    pub struct cpu_set_t {
        #[cfg(all(target_pointer_width = "32",
                  not(target_arch = "x86_64")))]
        bits: [u32; 32],
        #[cfg(not(all(target_pointer_width = "32",
                      not(target_arch = "x86_64"))))]
        bits: [u64; 16],
    }

    pub struct if_nameindex {
        pub if_index: ::c_uint,
        pub if_name: *mut ::c_char,
    }

    // System V IPC
    pub struct msginfo {
        pub msgpool: ::c_int,
        pub msgmap: ::c_int,
        pub msgmax: ::c_int,
        pub msgmnb: ::c_int,
        pub msgmni: ::c_int,
        pub msgssz: ::c_int,
        pub msgtql: ::c_int,
        pub msgseg: ::c_ushort,
    }

    pub struct sembuf {
        pub sem_num: ::c_ushort,
        pub sem_op: ::c_short,
        pub sem_flg: ::c_short,
    }

    pub struct input_event {
        pub time: ::timeval,
        pub type_: ::__u16,
        pub code: ::__u16,
        pub value: ::__s32,
    }

    pub struct input_id {
        pub bustype: ::__u16,
        pub vendor: ::__u16,
        pub product: ::__u16,
        pub version: ::__u16,
    }

    pub struct input_absinfo {
        pub value: ::__s32,
        pub minimum: ::__s32,
        pub maximum: ::__s32,
        pub fuzz: ::__s32,
        pub flat: ::__s32,
        pub resolution: ::__s32,
    }

    pub struct input_keymap_entry {
        pub flags: ::__u8,
        pub len: ::__u8,
        pub index: ::__u16,
        pub keycode: ::__u32,
        pub scancode: [::__u8; 32],
    }

    pub struct input_mask {
        pub type_: ::__u32,
        pub codes_size: ::__u32,
        pub codes_ptr: ::__u64,
    }

    pub struct ff_replay {
        pub length: ::__u16,
        pub delay: ::__u16,
    }

    pub struct ff_trigger {
        pub button: ::__u16,
        pub interval: ::__u16,
    }

    pub struct ff_envelope {
        pub attack_length: ::__u16,
        pub attack_level: ::__u16,
        pub fade_length: ::__u16,
        pub fade_level: ::__u16,
    }

    pub struct ff_constant_effect {
        pub level: ::__s16,
        pub envelope: ff_envelope,
    }

    pub struct ff_ramp_effect {
        pub start_level: ::__s16,
        pub end_level: ::__s16,
        pub envelope: ff_envelope,
    }

    pub struct ff_condition_effect {
        pub right_saturation: ::__u16,
        pub left_saturation: ::__u16,

        pub right_coeff: ::__s16,
        pub left_coeff: ::__s16,

        pub deadband: ::__u16,
        pub center: ::__s16,
    }

    pub struct ff_periodic_effect {
        pub waveform: ::__u16,
        pub period: ::__u16,
        pub magnitude: ::__s16,
        pub offset: ::__s16,
        pub phase: ::__u16,

        pub envelope: ff_envelope,

        pub custom_len: ::__u32,
        pub custom_data: *mut ::__s16,
    }

    pub struct ff_rumble_effect {
        pub strong_magnitude: ::__u16,
        pub weak_magnitude: ::__u16,
    }

    pub struct ff_effect {
        pub type_: ::__u16,
        pub id: ::__s16,
        pub direction: ::__u16,
        pub trigger: ff_trigger,
        pub replay: ff_replay,
        // FIXME this is actually a union
        #[cfg(target_pointer_width = "64")]
        pub u: [u64; 4],
        #[cfg(target_pointer_width = "32")]
        pub u: [u32; 7],
    }

    pub struct uinput_ff_upload {
        pub request_id: ::__u32,
        pub retval: ::__s32,
        pub effect: ff_effect,
        pub old: ff_effect,
    }

    pub struct uinput_ff_erase {
        pub request_id: ::__u32,
        pub retval: ::__s32,
        pub effect_id: ::__u32,
    }

    pub struct uinput_abs_setup {
        pub code: ::__u16,
        pub absinfo: input_absinfo,
    }

    pub struct dl_phdr_info {
        #[cfg(target_pointer_width = "64")]
        pub dlpi_addr: Elf64_Addr,
        #[cfg(target_pointer_width = "32")]
        pub dlpi_addr: Elf32_Addr,

        pub dlpi_name: *const ::c_char,

        #[cfg(target_pointer_width = "64")]
        pub dlpi_phdr: *const Elf64_Phdr,
        #[cfg(target_pointer_width = "32")]
        pub dlpi_phdr: *const Elf32_Phdr,

        #[cfg(target_pointer_width = "64")]
        pub dlpi_phnum: Elf64_Half,
        #[cfg(target_pointer_width = "32")]
        pub dlpi_phnum: Elf32_Half,

        // As of uClibc 1.0.36, the following fields are
        // gated behind a "#if 0" block which always evaluates
        // to false. So I'm just removing these, and if uClibc changes
        // the #if block in the future to include the following fields, these
        // will probably need including here. tsidea, skrap
        #[cfg(not(target_env = "uclibc"))]
        pub dlpi_adds: ::c_ulonglong,
        #[cfg(not(target_env = "uclibc"))]
        pub dlpi_subs: ::c_ulonglong,
        #[cfg(not(target_env = "uclibc"))]
        pub dlpi_tls_modid: ::size_t,
        #[cfg(not(target_env = "uclibc"))]
        pub dlpi_tls_data: *mut ::c_void,
    }

    pub struct Elf32_Ehdr {
        pub e_ident: [::c_uchar; 16],
        pub e_type: Elf32_Half,
        pub e_machine: Elf32_Half,
        pub e_version: Elf32_Word,
        pub e_entry: Elf32_Addr,
        pub e_phoff: Elf32_Off,
        pub e_shoff: Elf32_Off,
        pub e_flags: Elf32_Word,
        pub e_ehsize: Elf32_Half,
        pub e_phentsize: Elf32_Half,
        pub e_phnum: Elf32_Half,
        pub e_shentsize: Elf32_Half,
        pub e_shnum: Elf32_Half,
        pub e_shstrndx: Elf32_Half,
    }

    pub struct Elf64_Ehdr {
        pub e_ident: [::c_uchar; 16],
        pub e_type: Elf64_Half,
        pub e_machine: Elf64_Half,
        pub e_version: Elf64_Word,
        pub e_entry: Elf64_Addr,
        pub e_phoff: Elf64_Off,
        pub e_shoff: Elf64_Off,
        pub e_flags: Elf64_Word,
        pub e_ehsize: Elf64_Half,
        pub e_phentsize: Elf64_Half,
        pub e_phnum: Elf64_Half,
        pub e_shentsize: Elf64_Half,
        pub e_shnum: Elf64_Half,
        pub e_shstrndx: Elf64_Half,
    }

    pub struct Elf32_Sym {
        pub st_name: Elf32_Word,
        pub st_value: Elf32_Addr,
        pub st_size: Elf32_Word,
        pub st_info: ::c_uchar,
        pub st_other: ::c_uchar,
        pub st_shndx: Elf32_Section,
    }

    pub struct Elf64_Sym {
        pub st_name: Elf64_Word,
        pub st_info: ::c_uchar,
        pub st_other: ::c_uchar,
        pub st_shndx: Elf64_Section,
        pub st_value: Elf64_Addr,
        pub st_size: Elf64_Xword,
    }

    pub struct Elf32_Phdr {
        pub p_type: Elf32_Word,
        pub p_offset: Elf32_Off,
        pub p_vaddr: Elf32_Addr,
        pub p_paddr: Elf32_Addr,
        pub p_filesz: Elf32_Word,
        pub p_memsz: Elf32_Word,
        pub p_flags: Elf32_Word,
        pub p_align: Elf32_Word,
    }

    pub struct Elf64_Phdr {
        pub p_type: Elf64_Word,
        pub p_flags: Elf64_Word,
        pub p_offset: Elf64_Off,
        pub p_vaddr: Elf64_Addr,
        pub p_paddr: Elf64_Addr,
        pub p_filesz: Elf64_Xword,
        pub p_memsz: Elf64_Xword,
        pub p_align: Elf64_Xword,
    }

    pub struct Elf32_Shdr {
        pub sh_name: Elf32_Word,
        pub sh_type: Elf32_Word,
        pub sh_flags: Elf32_Word,
        pub sh_addr: Elf32_Addr,
        pub sh_offset: Elf32_Off,
        pub sh_size: Elf32_Word,
        pub sh_link: Elf32_Word,
        pub sh_info: Elf32_Word,
        pub sh_addralign: Elf32_Word,
        pub sh_entsize: Elf32_Word,
    }

    pub struct Elf64_Shdr {
        pub sh_name: Elf64_Word,
        pub sh_type: Elf64_Word,
        pub sh_flags: Elf64_Xword,
        pub sh_addr: Elf64_Addr,
        pub sh_offset: Elf64_Off,
        pub sh_size: Elf64_Xword,
        pub sh_link: Elf64_Word,
        pub sh_info: Elf64_Word,
        pub sh_addralign: Elf64_Xword,
        pub sh_entsize: Elf64_Xword,
    }

    pub struct ucred {
        pub pid: ::pid_t,
        pub uid: ::uid_t,
        pub gid: ::gid_t,
    }

    pub struct mntent {
        pub mnt_fsname: *mut ::c_char,
        pub mnt_dir: *mut ::c_char,
        pub mnt_type: *mut ::c_char,
        pub mnt_opts: *mut ::c_char,
        pub mnt_freq: ::c_int,
        pub mnt_passno: ::c_int,
    }

    pub struct posix_spawn_file_actions_t {
        __allocated: ::c_int,
        __used: ::c_int,
        __actions: *mut ::c_int,
        __pad: [::c_int; 16],
    }

    pub struct posix_spawnattr_t {
        __flags: ::c_short,
        __pgrp: ::pid_t,
        __sd: ::sigset_t,
        __ss: ::sigset_t,
        #[cfg(any(target_env = "musl", target_env = "ohos"))]
        __prio: ::c_int,
        #[cfg(not(any(target_env = "musl", target_env = "ohos")))]
        __sp: ::sched_param,
        __policy: ::c_int,
        __pad: [::c_int; 16],
    }

    pub struct genlmsghdr {
        pub cmd: u8,
        pub version: u8,
        pub reserved: u16,
    }

    pub struct in6_pktinfo {
        pub ipi6_addr: ::in6_addr,
        pub ipi6_ifindex: ::c_uint,
    }

    pub struct arpd_request {
        pub req: ::c_ushort,
        pub ip: u32,
        pub dev: ::c_ulong,
        pub stamp: ::c_ulong,
        pub updated: ::c_ulong,
        pub ha: [::c_uchar; ::MAX_ADDR_LEN],
    }

    pub struct inotify_event {
        pub wd: ::c_int,
        pub mask: u32,
        pub cookie: u32,
        pub len: u32
    }

    pub struct fanotify_response {
        pub fd: ::c_int,
        pub response: __u32,
    }

    pub struct sockaddr_vm {
        pub svm_family: ::sa_family_t,
        pub svm_reserved1: ::c_ushort,
        pub svm_port: ::c_uint,
        pub svm_cid: ::c_uint,
        pub svm_zero: [u8; 4]
    }

    pub struct regmatch_t {
        pub rm_so: regoff_t,
        pub rm_eo: regoff_t,
    }

    pub struct sock_extended_err {
        pub ee_errno: u32,
        pub ee_origin: u8,
        pub ee_type: u8,
        pub ee_code: u8,
        pub ee_pad: u8,
        pub ee_info: u32,
        pub ee_data: u32,
    }

    // linux/can.h
    pub struct __c_anonymous_sockaddr_can_tp {
        pub rx_id: canid_t,
        pub tx_id: canid_t,
    }

    pub struct __c_anonymous_sockaddr_can_j1939 {
        pub name: u64,
        pub pgn: u32,
        pub addr: u8,
    }

    pub struct can_filter {
        pub can_id: canid_t,
        pub can_mask: canid_t,
    }

    // linux/can/j1939.h
    pub struct j1939_filter {
        pub name: name_t,
        pub name_mask: name_t,
        pub pgn: pgn_t,
        pub pgn_mask: pgn_t,
        pub addr: u8,
        pub addr_mask: u8,
    }

    // linux/filter.h
    pub struct sock_filter {
        pub code: ::__u16,
        pub jt: ::__u8,
        pub jf: ::__u8,
        pub k: ::__u32,
    }

    pub struct sock_fprog {
        pub len: ::c_ushort,
        pub filter: *mut sock_filter,
    }

    // linux/seccomp.h
    pub struct seccomp_data {
        pub nr: ::c_int,
        pub arch: ::__u32,
        pub instruction_pointer: ::__u64,
        pub args: [::__u64; 6],
    }

    pub struct nlmsghdr {
        pub nlmsg_len: u32,
        pub nlmsg_type: u16,
        pub nlmsg_flags: u16,
        pub nlmsg_seq: u32,
        pub nlmsg_pid: u32,
    }

    pub struct nlmsgerr {
        pub error: ::c_int,
        pub msg: nlmsghdr,
    }

    pub struct nlattr {
        pub nla_len: u16,
        pub nla_type: u16,
    }

    pub struct file_clone_range {
        pub src_fd: ::__s64,
        pub src_offset: ::__u64,
        pub src_length: ::__u64,
        pub dest_offset: ::__u64,
    }

    pub struct __c_anonymous_ifru_map {
        pub mem_start: ::c_ulong,
        pub mem_end: ::c_ulong,
        pub base_addr: ::c_ushort,
        pub irq: ::c_uchar,
        pub dma: ::c_uchar,
        pub port: ::c_uchar,
    }

   pub struct in6_ifreq {
       pub ifr6_addr: ::in6_addr,
       pub ifr6_prefixlen: u32,
       pub ifr6_ifindex: ::c_int,
   }

    pub struct option {
        pub name: *const ::c_char,
        pub has_arg: ::c_int,
        pub flag: *mut ::c_int,
        pub val: ::c_int,
    }

    // linux/sctp.h

    pub struct sctp_initmsg {
        pub sinit_num_ostreams: ::__u16,
        pub sinit_max_instreams: ::__u16,
        pub sinit_max_attempts: ::__u16,
        pub sinit_max_init_timeo: ::__u16,
    }

    pub struct sctp_sndrcvinfo {
        pub sinfo_stream: ::__u16,
        pub sinfo_ssn: ::__u16,
        pub sinfo_flags: ::__u16,
        pub sinfo_ppid: ::__u32,
        pub sinfo_context: ::__u32,
        pub sinfo_timetolive: ::__u32,
        pub sinfo_tsn: ::__u32,
        pub sinfo_cumtsn: ::__u32,
        pub sinfo_assoc_id: ::sctp_assoc_t,
    }

    pub struct sctp_sndinfo {
        pub snd_sid: ::__u16,
        pub snd_flags: ::__u16,
        pub snd_ppid: ::__u32,
        pub snd_context: ::__u32,
        pub snd_assoc_id: ::sctp_assoc_t,
    }

    pub struct sctp_rcvinfo {
        pub rcv_sid: ::__u16,
        pub rcv_ssn: ::__u16,
        pub rcv_flags: ::__u16,
        pub rcv_ppid: ::__u32,
        pub rcv_tsn: ::__u32,
        pub rcv_cumtsn: ::__u32,
        pub rcv_context: ::__u32,
        pub rcv_assoc_id: ::sctp_assoc_t,
    }

    pub struct sctp_nxtinfo {
        pub nxt_sid: ::__u16,
        pub nxt_flags: ::__u16,
        pub nxt_ppid: ::__u32,
        pub nxt_length: ::__u32,
        pub nxt_assoc_id: ::sctp_assoc_t,
    }

    pub struct sctp_prinfo {
        pub pr_policy: ::__u16,
        pub pr_value: ::__u32,
    }

    pub struct sctp_authinfo {
        pub auth_keynumber: ::__u16,
    }

    pub struct rlimit64 {
        pub rlim_cur: rlim64_t,
        pub rlim_max: rlim64_t,
    }
}

s_no_extra_traits! {
    pub struct sockaddr_nl {
        pub nl_family: ::sa_family_t,
        nl_pad: ::c_ushort,
        pub nl_pid: u32,
        pub nl_groups: u32
    }

    pub struct dirent {
        pub d_ino: ::ino_t,
        pub d_off: ::off_t,
        pub d_reclen: ::c_ushort,
        pub d_type: ::c_uchar,
        pub d_name: [::c_char; 256],
    }

    pub struct sockaddr_alg {
        pub salg_family: ::sa_family_t,
        pub salg_type: [::c_uchar; 14],
        pub salg_feat: u32,
        pub salg_mask: u32,
        pub salg_name: [::c_uchar; 64],
    }

    pub struct uinput_setup {
        pub id: input_id,
        pub name: [::c_char; UINPUT_MAX_NAME_SIZE],
        pub ff_effects_max: ::__u32,
    }

    pub struct uinput_user_dev {
        pub name: [::c_char; UINPUT_MAX_NAME_SIZE],
        pub id: input_id,
        pub ff_effects_max: ::__u32,
        pub absmax: [::__s32; ABS_CNT],
        pub absmin: [::__s32; ABS_CNT],
        pub absfuzz: [::__s32; ABS_CNT],
        pub absflat: [::__s32; ABS_CNT],
    }

    /// WARNING: The `PartialEq`, `Eq` and `Hash` implementations of this
    /// type are unsound and will be removed in the future.
    #[deprecated(
        note = "this struct has unsafe trait implementations that will be \
                removed in the future",
        since = "0.2.80"
    )]
    pub struct af_alg_iv {
        pub ivlen: u32,
        pub iv: [::c_uchar; 0],
    }

    // x32 compatibility
    // See https://sourceware.org/bugzilla/show_bug.cgi?id=21279
    pub struct mq_attr {
        #[cfg(all(target_arch = "x86_64", target_pointer_width = "32"))]
        pub mq_flags: i64,
        #[cfg(all(target_arch = "x86_64", target_pointer_width = "32"))]
        pub mq_maxmsg: i64,
        #[cfg(all(target_arch = "x86_64", target_pointer_width = "32"))]
        pub mq_msgsize: i64,
        #[cfg(all(target_arch = "x86_64", target_pointer_width = "32"))]
        pub mq_curmsgs: i64,
        #[cfg(all(target_arch = "x86_64", target_pointer_width = "32"))]
        pad: [i64; 4],

        #[cfg(not(all(target_arch = "x86_64", target_pointer_width = "32")))]
        pub mq_flags: ::c_long,
        #[cfg(not(all(target_arch = "x86_64", target_pointer_width = "32")))]
        pub mq_maxmsg: ::c_long,
        #[cfg(not(all(target_arch = "x86_64", target_pointer_width = "32")))]
        pub mq_msgsize: ::c_long,
        #[cfg(not(all(target_arch = "x86_64", target_pointer_width = "32")))]
        pub mq_curmsgs: ::c_long,
        #[cfg(not(all(target_arch = "x86_64", target_pointer_width = "32")))]
        pad: [::c_long; 4],
    }

    #[cfg(libc_union)]
    pub union __c_anonymous_ifr_ifru {
        pub ifru_addr: ::sockaddr,
        pub ifru_dstaddr: ::sockaddr,
        pub ifru_broadaddr: ::sockaddr,
        pub ifru_netmask: ::sockaddr,
        pub ifru_hwaddr: ::sockaddr,
        pub ifru_flags: ::c_short,
        pub ifru_ifindex: ::c_int,
        pub ifru_metric: ::c_int,
        pub ifru_mtu: ::c_int,
        pub ifru_map: __c_anonymous_ifru_map,
        pub ifru_slave: [::c_char; ::IFNAMSIZ],
        pub ifru_newname: [::c_char; ::IFNAMSIZ],
        pub ifru_data: *mut ::c_char,
    }

    pub struct ifreq {
        /// interface name, e.g. "en0"
        pub ifr_name: [::c_char; ::IFNAMSIZ],
        #[cfg(libc_union)]
        pub ifr_ifru: __c_anonymous_ifr_ifru,
        #[cfg(not(libc_union))]
        pub ifr_ifru: ::sockaddr,
    }

    pub struct hwtstamp_config {
        pub flags: ::c_int,
        pub tx_type: ::c_int,
        pub rx_filter: ::c_int,
    }

    pub struct dirent64 {
        pub d_ino: ::ino64_t,
        pub d_off: ::off64_t,
        pub d_reclen: ::c_ushort,
        pub d_type: ::c_uchar,
        pub d_name: [::c_char; 256],
    }
}

s_no_extra_traits! {
    // linux/net_tstamp.h
    #[allow(missing_debug_implementations)]
    pub struct sock_txtime {
        pub clockid: ::clockid_t,
        pub flags: ::__u32,
    }
}

cfg_if! {
    if #[cfg(libc_union)] {
        s_no_extra_traits! {
            // linux/can.h
            #[allow(missing_debug_implementations)]
            pub union __c_anonymous_sockaddr_can_can_addr {
                pub tp: __c_anonymous_sockaddr_can_tp,
                pub j1939: __c_anonymous_sockaddr_can_j1939,
            }

            #[allow(missing_debug_implementations)]
            pub struct sockaddr_can {
                pub can_family: ::sa_family_t,
                pub can_ifindex: ::c_int,
                pub can_addr: __c_anonymous_sockaddr_can_can_addr,
            }
        }
    }
}

cfg_if! {
    if #[cfg(feature = "extra_traits")] {
        impl PartialEq for sockaddr_nl {
            fn eq(&self, other: &sockaddr_nl) -> bool {
                self.nl_family == other.nl_family &&
                    self.nl_pid == other.nl_pid &&
                    self.nl_groups == other.nl_groups
            }
        }
        impl Eq for sockaddr_nl {}
        impl ::fmt::Debug for sockaddr_nl {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("sockaddr_nl")
                    .field("nl_family", &self.nl_family)
                    .field("nl_pid", &self.nl_pid)
                    .field("nl_groups", &self.nl_groups)
                    .finish()
            }
        }
        impl ::hash::Hash for sockaddr_nl {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.nl_family.hash(state);
                self.nl_pid.hash(state);
                self.nl_groups.hash(state);
            }
        }

        impl PartialEq for dirent {
            fn eq(&self, other: &dirent) -> bool {
                self.d_ino == other.d_ino
                    && self.d_off == other.d_off
                    && self.d_reclen == other.d_reclen
                    && self.d_type == other.d_type
                    && self
                    .d_name
                    .iter()
                    .zip(other.d_name.iter())
                    .all(|(a,b)| a == b)
            }
        }

        impl Eq for dirent {}

        impl ::fmt::Debug for dirent {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("dirent")
                    .field("d_ino", &self.d_ino)
                    .field("d_off", &self.d_off)
                    .field("d_reclen", &self.d_reclen)
                    .field("d_type", &self.d_type)
                // FIXME: .field("d_name", &self.d_name)
                    .finish()
            }
        }

        impl ::hash::Hash for dirent {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.d_ino.hash(state);
                self.d_off.hash(state);
                self.d_reclen.hash(state);
                self.d_type.hash(state);
                self.d_name.hash(state);
            }
        }

        impl PartialEq for dirent64 {
            fn eq(&self, other: &dirent64) -> bool {
                self.d_ino == other.d_ino
                    && self.d_off == other.d_off
                    && self.d_reclen == other.d_reclen
                    && self.d_type == other.d_type
                    && self
                    .d_name
                    .iter()
                    .zip(other.d_name.iter())
                    .all(|(a,b)| a == b)
            }
        }

        impl Eq for dirent64 {}

        impl ::fmt::Debug for dirent64 {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("dirent64")
                    .field("d_ino", &self.d_ino)
                    .field("d_off", &self.d_off)
                    .field("d_reclen", &self.d_reclen)
                    .field("d_type", &self.d_type)
                // FIXME: .field("d_name", &self.d_name)
                    .finish()
            }
        }

        impl ::hash::Hash for dirent64 {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.d_ino.hash(state);
                self.d_off.hash(state);
                self.d_reclen.hash(state);
                self.d_type.hash(state);
                self.d_name.hash(state);
            }
        }

        impl PartialEq for pthread_cond_t {
            fn eq(&self, other: &pthread_cond_t) -> bool {
                self.size.iter().zip(other.size.iter()).all(|(a,b)| a == b)
            }
        }

        impl Eq for pthread_cond_t {}

        impl ::fmt::Debug for pthread_cond_t {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("pthread_cond_t")
                // FIXME: .field("size", &self.size)
                    .finish()
            }
        }

        impl ::hash::Hash for pthread_cond_t {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.size.hash(state);
            }
        }

        impl PartialEq for pthread_mutex_t {
            fn eq(&self, other: &pthread_mutex_t) -> bool {
                self.size.iter().zip(other.size.iter()).all(|(a,b)| a == b)
            }
        }

        impl Eq for pthread_mutex_t {}

        impl ::fmt::Debug for pthread_mutex_t {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("pthread_mutex_t")
                // FIXME: .field("size", &self.size)
                    .finish()
            }
        }

        impl ::hash::Hash for pthread_mutex_t {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.size.hash(state);
            }
        }

        impl PartialEq for pthread_rwlock_t {
            fn eq(&self, other: &pthread_rwlock_t) -> bool {
                self.size.iter().zip(other.size.iter()).all(|(a,b)| a == b)
            }
        }

        impl Eq for pthread_rwlock_t {}

        impl ::fmt::Debug for pthread_rwlock_t {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("pthread_rwlock_t")
                // FIXME: .field("size", &self.size)
                    .finish()
            }
        }

        impl ::hash::Hash for pthread_rwlock_t {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.size.hash(state);
            }
        }

        impl PartialEq for pthread_barrier_t {
            fn eq(&self, other: &pthread_barrier_t) -> bool {
                self.size.iter().zip(other.size.iter()).all(|(a,b)| a == b)
            }
        }

        impl Eq for pthread_barrier_t {}

        impl ::fmt::Debug for pthread_barrier_t {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("pthread_barrier_t")
                    .field("size", &self.size)
                    .finish()
            }
        }

        impl ::hash::Hash for pthread_barrier_t {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.size.hash(state);
            }
        }

        impl PartialEq for sockaddr_alg {
            fn eq(&self, other: &sockaddr_alg) -> bool {
                self.salg_family == other.salg_family
                    && self
                    .salg_type
                    .iter()
                    .zip(other.salg_type.iter())
                    .all(|(a, b)| a == b)
                    && self.salg_feat == other.salg_feat
                    && self.salg_mask == other.salg_mask
                    && self
                    .salg_name
                    .iter()
                    .zip(other.salg_name.iter())
                    .all(|(a, b)| a == b)
           }
        }

        impl Eq for sockaddr_alg {}

        impl ::fmt::Debug for sockaddr_alg {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("sockaddr_alg")
                    .field("salg_family", &self.salg_family)
                    .field("salg_type", &self.salg_type)
                    .field("salg_feat", &self.salg_feat)
                    .field("salg_mask", &self.salg_mask)
                    .field("salg_name", &&self.salg_name[..])
                    .finish()
            }
        }

        impl ::hash::Hash for sockaddr_alg {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.salg_family.hash(state);
                self.salg_type.hash(state);
                self.salg_feat.hash(state);
                self.salg_mask.hash(state);
                self.salg_name.hash(state);
            }
        }

        impl PartialEq for uinput_setup {
            fn eq(&self, other: &uinput_setup) -> bool {
                self.id == other.id
                    && self.name[..] == other.name[..]
                    && self.ff_effects_max == other.ff_effects_max
           }
        }
        impl Eq for uinput_setup {}

        impl ::fmt::Debug for uinput_setup {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("uinput_setup")
                    .field("id", &self.id)
                    .field("name", &&self.name[..])
                    .field("ff_effects_max", &self.ff_effects_max)
                    .finish()
            }
        }

        impl ::hash::Hash for uinput_setup {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.id.hash(state);
                self.name.hash(state);
                self.ff_effects_max.hash(state);
            }
        }

        impl PartialEq for uinput_user_dev {
            fn eq(&self, other: &uinput_user_dev) -> bool {
                 self.name[..] == other.name[..]
                    && self.id == other.id
                    && self.ff_effects_max == other.ff_effects_max
                    && self.absmax[..] == other.absmax[..]
                    && self.absmin[..] == other.absmin[..]
                    && self.absfuzz[..] == other.absfuzz[..]
                    && self.absflat[..] == other.absflat[..]
           }
        }
        impl Eq for uinput_user_dev {}

        impl ::fmt::Debug for uinput_user_dev {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("uinput_setup")
                    .field("name", &&self.name[..])
                    .field("id", &self.id)
                    .field("ff_effects_max", &self.ff_effects_max)
                    .field("absmax", &&self.absmax[..])
                    .field("absmin", &&self.absmin[..])
                    .field("absfuzz", &&self.absfuzz[..])
                    .field("absflat", &&self.absflat[..])
                    .finish()
            }
        }

        impl ::hash::Hash for uinput_user_dev {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.name.hash(state);
                self.id.hash(state);
                self.ff_effects_max.hash(state);
                self.absmax.hash(state);
                self.absmin.hash(state);
                self.absfuzz.hash(state);
                self.absflat.hash(state);
            }
        }

        #[allow(deprecated)]
        impl af_alg_iv {
            fn as_slice(&self) -> &[u8] {
                unsafe {
                    ::core::slice::from_raw_parts(
                        self.iv.as_ptr(),
                        self.ivlen as usize
                    )
                }
            }
        }

        #[allow(deprecated)]
        impl PartialEq for af_alg_iv {
            fn eq(&self, other: &af_alg_iv) -> bool {
                *self.as_slice() == *other.as_slice()
           }
        }

        #[allow(deprecated)]
        impl Eq for af_alg_iv {}

        #[allow(deprecated)]
        impl ::fmt::Debug for af_alg_iv {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("af_alg_iv")
                    .field("ivlen", &self.ivlen)
                    .finish()
            }
        }

        #[allow(deprecated)]
        impl ::hash::Hash for af_alg_iv {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.as_slice().hash(state);
            }
        }

        impl PartialEq for mq_attr {
            fn eq(&self, other: &mq_attr) -> bool {
                self.mq_flags == other.mq_flags &&
                self.mq_maxmsg == other.mq_maxmsg &&
                self.mq_msgsize == other.mq_msgsize &&
                self.mq_curmsgs == other.mq_curmsgs
            }
        }
        impl Eq for mq_attr {}
        impl ::fmt::Debug for mq_attr {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("mq_attr")
                    .field("mq_flags", &self.mq_flags)
                    .field("mq_maxmsg", &self.mq_maxmsg)
                    .field("mq_msgsize", &self.mq_msgsize)
                    .field("mq_curmsgs", &self.mq_curmsgs)
                    .finish()
            }
        }
        impl ::hash::Hash for mq_attr {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.mq_flags.hash(state);
                self.mq_maxmsg.hash(state);
                self.mq_msgsize.hash(state);
                self.mq_curmsgs.hash(state);
            }
        }
        #[cfg(libc_union)]
        impl ::fmt::Debug for __c_anonymous_ifr_ifru {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("ifr_ifru")
                    .field("ifru_addr", unsafe { &self.ifru_addr })
                    .field("ifru_dstaddr", unsafe { &self.ifru_dstaddr })
                    .field("ifru_broadaddr", unsafe { &self.ifru_broadaddr })
                    .field("ifru_netmask", unsafe { &self.ifru_netmask })
                    .field("ifru_hwaddr", unsafe { &self.ifru_hwaddr })
                    .field("ifru_flags", unsafe { &self.ifru_flags })
                    .field("ifru_ifindex", unsafe { &self.ifru_ifindex })
                    .field("ifru_metric", unsafe { &self.ifru_metric })
                    .field("ifru_mtu", unsafe { &self.ifru_mtu })
                    .field("ifru_map", unsafe { &self.ifru_map })
                    .field("ifru_slave", unsafe { &self.ifru_slave })
                    .field("ifru_newname", unsafe { &self.ifru_newname })
                    .field("ifru_data", unsafe { &self.ifru_data })
                    .finish()
            }
        }
        impl ::fmt::Debug for ifreq {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("ifreq")
                    .field("ifr_name", &self.ifr_name)
                    .field("ifr_ifru", &self.ifr_ifru)
                    .finish()
            }
        }

        impl ::fmt::Debug for hwtstamp_config {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("hwtstamp_config")
                    .field("flags", &self.flags)
                    .field("tx_type", &self.tx_type)
                    .field("rx_filter", &self.rx_filter)
                    .finish()
            }
        }
        impl PartialEq for hwtstamp_config {
            fn eq(&self, other: &hwtstamp_config) -> bool {
                self.flags == other.flags &&
                self.tx_type == other.tx_type &&
                self.rx_filter == other.rx_filter
            }
        }
        impl Eq for hwtstamp_config {}
        impl ::hash::Hash for hwtstamp_config {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.flags.hash(state);
                self.tx_type.hash(state);
                self.rx_filter.hash(state);
            }
        }
    }
}

cfg_if! {
    if #[cfg(any(target_env = "gnu", target_env = "musl", target_env = "ohos"))] {
        pub const ABDAY_1: ::nl_item = 0x20000;
        pub const ABDAY_2: ::nl_item = 0x20001;
        pub const ABDAY_3: ::nl_item = 0x20002;
        pub const ABDAY_4: ::nl_item = 0x20003;
        pub const ABDAY_5: ::nl_item = 0x20004;
        pub const ABDAY_6: ::nl_item = 0x20005;
        pub const ABDAY_7: ::nl_item = 0x20006;

        pub const DAY_1: ::nl_item = 0x20007;
        pub const DAY_2: ::nl_item = 0x20008;
        pub const DAY_3: ::nl_item = 0x20009;
        pub const DAY_4: ::nl_item = 0x2000A;
        pub const DAY_5: ::nl_item = 0x2000B;
        pub const DAY_6: ::nl_item = 0x2000C;
        pub const DAY_7: ::nl_item = 0x2000D;

        pub const ABMON_1: ::nl_item = 0x2000E;
        pub const ABMON_2: ::nl_item = 0x2000F;
        pub const ABMON_3: ::nl_item = 0x20010;
        pub const ABMON_4: ::nl_item = 0x20011;
        pub const ABMON_5: ::nl_item = 0x20012;
        pub const ABMON_6: ::nl_item = 0x20013;
        pub const ABMON_7: ::nl_item = 0x20014;
        pub const ABMON_8: ::nl_item = 0x20015;
        pub const ABMON_9: ::nl_item = 0x20016;
        pub const ABMON_10: ::nl_item = 0x20017;
        pub const ABMON_11: ::nl_item = 0x20018;
        pub const ABMON_12: ::nl_item = 0x20019;

        pub const MON_1: ::nl_item = 0x2001A;
        pub const MON_2: ::nl_item = 0x2001B;
        pub const MON_3: ::nl_item = 0x2001C;
        pub const MON_4: ::nl_item = 0x2001D;
        pub const MON_5: ::nl_item = 0x2001E;
        pub const MON_6: ::nl_item = 0x2001F;
        pub const MON_7: ::nl_item = 0x20020;
        pub const MON_8: ::nl_item = 0x20021;
        pub const MON_9: ::nl_item = 0x20022;
        pub const MON_10: ::nl_item = 0x20023;
        pub const MON_11: ::nl_item = 0x20024;
        pub const MON_12: ::nl_item = 0x20025;

        pub const AM_STR: ::nl_item = 0x20026;
        pub const PM_STR: ::nl_item = 0x20027;

        pub const D_T_FMT: ::nl_item = 0x20028;
        pub const D_FMT: ::nl_item = 0x20029;
        pub const T_FMT: ::nl_item = 0x2002A;
        pub const T_FMT_AMPM: ::nl_item = 0x2002B;

        pub const ERA: ::nl_item = 0x2002C;
        pub const ERA_D_FMT: ::nl_item = 0x2002E;
        pub const ALT_DIGITS: ::nl_item = 0x2002F;
        pub const ERA_D_T_FMT: ::nl_item = 0x20030;
        pub const ERA_T_FMT: ::nl_item = 0x20031;

        pub const CODESET: ::nl_item = 14;
        pub const CRNCYSTR: ::nl_item = 0x4000F;
        pub const RADIXCHAR: ::nl_item = 0x10000;
        pub const THOUSEP: ::nl_item = 0x10001;
        pub const YESEXPR: ::nl_item = 0x50000;
        pub const NOEXPR: ::nl_item = 0x50001;
        pub const YESSTR: ::nl_item = 0x50002;
        pub const NOSTR: ::nl_item = 0x50003;
    }
}

pub const RUSAGE_CHILDREN: ::c_int = -1;
pub const L_tmpnam: ::c_uint = 20;
pub const _PC_LINK_MAX: ::c_int = 0;
pub const _PC_MAX_CANON: ::c_int = 1;
pub const _PC_MAX_INPUT: ::c_int = 2;
pub const _PC_NAME_MAX: ::c_int = 3;
pub const _PC_PATH_MAX: ::c_int = 4;
pub const _PC_PIPE_BUF: ::c_int = 5;
pub const _PC_CHOWN_RESTRICTED: ::c_int = 6;
pub const _PC_NO_TRUNC: ::c_int = 7;
pub const _PC_VDISABLE: ::c_int = 8;
pub const _PC_SYNC_IO: ::c_int = 9;
pub const _PC_ASYNC_IO: ::c_int = 10;
pub const _PC_PRIO_IO: ::c_int = 11;
pub const _PC_SOCK_MAXBUF: ::c_int = 12;
pub const _PC_FILESIZEBITS: ::c_int = 13;
pub const _PC_REC_INCR_XFER_SIZE: ::c_int = 14;
pub const _PC_REC_MAX_XFER_SIZE: ::c_int = 15;
pub const _PC_REC_MIN_XFER_SIZE: ::c_int = 16;
pub const _PC_REC_XFER_ALIGN: ::c_int = 17;
pub const _PC_ALLOC_SIZE_MIN: ::c_int = 18;
pub const _PC_SYMLINK_MAX: ::c_int = 19;
pub const _PC_2_SYMLINKS: ::c_int = 20;

pub const MS_NOUSER: ::c_ulong = 0xffffffff80000000;

pub const _SC_ARG_MAX: ::c_int = 0;
pub const _SC_CHILD_MAX: ::c_int = 1;
pub const _SC_CLK_TCK: ::c_int = 2;
pub const _SC_NGROUPS_MAX: ::c_int = 3;
pub const _SC_OPEN_MAX: ::c_int = 4;
pub const _SC_STREAM_MAX: ::c_int = 5;
pub const _SC_TZNAME_MAX: ::c_int = 6;
pub const _SC_JOB_CONTROL: ::c_int = 7;
pub const _SC_SAVED_IDS: ::c_int = 8;
pub const _SC_REALTIME_SIGNALS: ::c_int = 9;
pub const _SC_PRIORITY_SCHEDULING: ::c_int = 10;
pub const _SC_TIMERS: ::c_int = 11;
pub const _SC_ASYNCHRONOUS_IO: ::c_int = 12;
pub const _SC_PRIORITIZED_IO: ::c_int = 13;
pub const _SC_SYNCHRONIZED_IO: ::c_int = 14;
pub const _SC_FSYNC: ::c_int = 15;
pub const _SC_MAPPED_FILES: ::c_int = 16;
pub const _SC_MEMLOCK: ::c_int = 17;
pub const _SC_MEMLOCK_RANGE: ::c_int = 18;
pub const _SC_MEMORY_PROTECTION: ::c_int = 19;
pub const _SC_MESSAGE_PASSING: ::c_int = 20;
pub const _SC_SEMAPHORES: ::c_int = 21;
pub const _SC_SHARED_MEMORY_OBJECTS: ::c_int = 22;
pub const _SC_AIO_LISTIO_MAX: ::c_int = 23;
pub const _SC_AIO_MAX: ::c_int = 24;
pub const _SC_AIO_PRIO_DELTA_MAX: ::c_int = 25;
pub const _SC_DELAYTIMER_MAX: ::c_int = 26;
pub const _SC_MQ_OPEN_MAX: ::c_int = 27;
pub const _SC_MQ_PRIO_MAX: ::c_int = 28;
pub const _SC_VERSION: ::c_int = 29;
pub const _SC_PAGESIZE: ::c_int = 30;
pub const _SC_PAGE_SIZE: ::c_int = _SC_PAGESIZE;
pub const _SC_RTSIG_MAX: ::c_int = 31;
pub const _SC_SEM_NSEMS_MAX: ::c_int = 32;
pub const _SC_SEM_VALUE_MAX: ::c_int = 33;
pub const _SC_SIGQUEUE_MAX: ::c_int = 34;
pub const _SC_TIMER_MAX: ::c_int = 35;
pub const _SC_BC_BASE_MAX: ::c_int = 36;
pub const _SC_BC_DIM_MAX: ::c_int = 37;
pub const _SC_BC_SCALE_MAX: ::c_int = 38;
pub const _SC_BC_STRING_MAX: ::c_int = 39;
pub const _SC_COLL_WEIGHTS_MAX: ::c_int = 40;
pub const _SC_EXPR_NEST_MAX: ::c_int = 42;
pub const _SC_LINE_MAX: ::c_int = 43;
pub const _SC_RE_DUP_MAX: ::c_int = 44;
pub const _SC_2_VERSION: ::c_int = 46;
pub const _SC_2_C_BIND: ::c_int = 47;
pub const _SC_2_C_DEV: ::c_int = 48;
pub const _SC_2_FORT_DEV: ::c_int = 49;
pub const _SC_2_FORT_RUN: ::c_int = 50;
pub const _SC_2_SW_DEV: ::c_int = 51;
pub const _SC_2_LOCALEDEF: ::c_int = 52;
pub const _SC_UIO_MAXIOV: ::c_int = 60;
pub const _SC_IOV_MAX: ::c_int = 60;
pub const _SC_THREADS: ::c_int = 67;
pub const _SC_THREAD_SAFE_FUNCTIONS: ::c_int = 68;
pub const _SC_GETGR_R_SIZE_MAX: ::c_int = 69;
pub const _SC_GETPW_R_SIZE_MAX: ::c_int = 70;
pub const _SC_LOGIN_NAME_MAX: ::c_int = 71;
pub const _SC_TTY_NAME_MAX: ::c_int = 72;
pub const _SC_THREAD_DESTRUCTOR_ITERATIONS: ::c_int = 73;
pub const _SC_THREAD_KEYS_MAX: ::c_int = 74;
pub const _SC_THREAD_STACK_MIN: ::c_int = 75;
pub const _SC_THREAD_THREADS_MAX: ::c_int = 76;
pub const _SC_THREAD_ATTR_STACKADDR: ::c_int = 77;
pub const _SC_THREAD_ATTR_STACKSIZE: ::c_int = 78;
pub const _SC_THREAD_PRIORITY_SCHEDULING: ::c_int = 79;
pub const _SC_THREAD_PRIO_INHERIT: ::c_int = 80;
pub const _SC_THREAD_PRIO_PROTECT: ::c_int = 81;
pub const _SC_THREAD_PROCESS_SHARED: ::c_int = 82;
pub const _SC_NPROCESSORS_CONF: ::c_int = 83;
pub const _SC_NPROCESSORS_ONLN: ::c_int = 84;
pub const _SC_PHYS_PAGES: ::c_int = 85;
pub const _SC_AVPHYS_PAGES: ::c_int = 86;
pub const _SC_ATEXIT_MAX: ::c_int = 87;
pub const _SC_PASS_MAX: ::c_int = 88;
pub const _SC_XOPEN_VERSION: ::c_int = 89;
pub const _SC_XOPEN_XCU_VERSION: ::c_int = 90;
pub const _SC_XOPEN_UNIX: ::c_int = 91;
pub const _SC_XOPEN_CRYPT: ::c_int = 92;
pub const _SC_XOPEN_ENH_I18N: ::c_int = 93;
pub const _SC_XOPEN_SHM: ::c_int = 94;
pub const _SC_2_CHAR_TERM: ::c_int = 95;
pub const _SC_2_UPE: ::c_int = 97;
pub const _SC_XOPEN_XPG2: ::c_int = 98;
pub const _SC_XOPEN_XPG3: ::c_int = 99;
pub const _SC_XOPEN_XPG4: ::c_int = 100;
pub const _SC_NZERO: ::c_int = 109;
pub const _SC_XBS5_ILP32_OFF32: ::c_int = 125;
pub const _SC_XBS5_ILP32_OFFBIG: ::c_int = 126;
pub const _SC_XBS5_LP64_OFF64: ::c_int = 127;
pub const _SC_XBS5_LPBIG_OFFBIG: ::c_int = 128;
pub const _SC_XOPEN_LEGACY: ::c_int = 129;
pub const _SC_XOPEN_REALTIME: ::c_int = 130;
pub const _SC_XOPEN_REALTIME_THREADS: ::c_int = 131;
pub const _SC_ADVISORY_INFO: ::c_int = 132;
pub const _SC_BARRIERS: ::c_int = 133;
pub const _SC_CLOCK_SELECTION: ::c_int = 137;
pub const _SC_CPUTIME: ::c_int = 138;
pub const _SC_THREAD_CPUTIME: ::c_int = 139;
pub const _SC_MONOTONIC_CLOCK: ::c_int = 149;
pub const _SC_READER_WRITER_LOCKS: ::c_int = 153;
pub const _SC_SPIN_LOCKS: ::c_int = 154;
pub const _SC_REGEXP: ::c_int = 155;
pub const _SC_SHELL: ::c_int = 157;
pub const _SC_SPAWN: ::c_int = 159;
pub const _SC_SPORADIC_SERVER: ::c_int = 160;
pub const _SC_THREAD_SPORADIC_SERVER: ::c_int = 161;
pub const _SC_TIMEOUTS: ::c_int = 164;
pub const _SC_TYPED_MEMORY_OBJECTS: ::c_int = 165;
pub const _SC_2_PBS: ::c_int = 168;
pub const _SC_2_PBS_ACCOUNTING: ::c_int = 169;
pub const _SC_2_PBS_LOCATE: ::c_int = 170;
pub const _SC_2_PBS_MESSAGE: ::c_int = 171;
pub const _SC_2_PBS_TRACK: ::c_int = 172;
pub const _SC_SYMLOOP_MAX: ::c_int = 173;
pub const _SC_STREAMS: ::c_int = 174;
pub const _SC_2_PBS_CHECKPOINT: ::c_int = 175;
pub const _SC_V6_ILP32_OFF32: ::c_int = 176;
pub const _SC_V6_ILP32_OFFBIG: ::c_int = 177;
pub const _SC_V6_LP64_OFF64: ::c_int = 178;
pub const _SC_V6_LPBIG_OFFBIG: ::c_int = 179;
pub const _SC_HOST_NAME_MAX: ::c_int = 180;
pub const _SC_TRACE: ::c_int = 181;
pub const _SC_TRACE_EVENT_FILTER: ::c_int = 182;
pub const _SC_TRACE_INHERIT: ::c_int = 183;
pub const _SC_TRACE_LOG: ::c_int = 184;
pub const _SC_IPV6: ::c_int = 235;
pub const _SC_RAW_SOCKETS: ::c_int = 236;
pub const _SC_V7_ILP32_OFF32: ::c_int = 237;
pub const _SC_V7_ILP32_OFFBIG: ::c_int = 238;
pub const _SC_V7_LP64_OFF64: ::c_int = 239;
pub const _SC_V7_LPBIG_OFFBIG: ::c_int = 240;
pub const _SC_SS_REPL_MAX: ::c_int = 241;
pub const _SC_TRACE_EVENT_NAME_MAX: ::c_int = 242;
pub const _SC_TRACE_NAME_MAX: ::c_int = 243;
pub const _SC_TRACE_SYS_MAX: ::c_int = 244;
pub const _SC_TRACE_USER_EVENT_MAX: ::c_int = 245;
pub const _SC_XOPEN_STREAMS: ::c_int = 246;
pub const _SC_THREAD_ROBUST_PRIO_INHERIT: ::c_int = 247;
pub const _SC_THREAD_ROBUST_PRIO_PROTECT: ::c_int = 248;

pub const RLIM_SAVED_MAX: ::rlim_t = RLIM_INFINITY;
pub const RLIM_SAVED_CUR: ::rlim_t = RLIM_INFINITY;

// elf.h - Fields in the e_ident array.
pub const EI_NIDENT: usize = 16;

pub const EI_MAG0: usize = 0;
pub const ELFMAG0: u8 = 0x7f;
pub const EI_MAG1: usize = 1;
pub const ELFMAG1: u8 = b'E';
pub const EI_MAG2: usize = 2;
pub const ELFMAG2: u8 = b'L';
pub const EI_MAG3: usize = 3;
pub const ELFMAG3: u8 = b'F';
pub const SELFMAG: usize = 4;

pub const EI_CLASS: usize = 4;
pub const ELFCLASSNONE: u8 = 0;
pub const ELFCLASS32: u8 = 1;
pub const ELFCLASS64: u8 = 2;
pub const ELFCLASSNUM: usize = 3;

pub const EI_DATA: usize = 5;
pub const ELFDATANONE: u8 = 0;
pub const ELFDATA2LSB: u8 = 1;
pub const ELFDATA2MSB: u8 = 2;
pub const ELFDATANUM: usize = 3;

pub const EI_VERSION: usize = 6;

pub const EI_OSABI: usize = 7;
pub const ELFOSABI_NONE: u8 = 0;
pub const ELFOSABI_SYSV: u8 = 0;
pub const ELFOSABI_HPUX: u8 = 1;
pub const ELFOSABI_NETBSD: u8 = 2;
pub const ELFOSABI_GNU: u8 = 3;
pub const ELFOSABI_LINUX: u8 = ELFOSABI_GNU;
pub const ELFOSABI_SOLARIS: u8 = 6;
pub const ELFOSABI_AIX: u8 = 7;
pub const ELFOSABI_IRIX: u8 = 8;
pub const ELFOSABI_FREEBSD: u8 = 9;
pub const ELFOSABI_TRU64: u8 = 10;
pub const ELFOSABI_MODESTO: u8 = 11;
pub const ELFOSABI_OPENBSD: u8 = 12;
pub const ELFOSABI_ARM: u8 = 97;
pub const ELFOSABI_STANDALONE: u8 = 255;

pub const EI_ABIVERSION: usize = 8;

pub const EI_PAD: usize = 9;

// elf.h - Legal values for e_type (object file type).
pub const ET_NONE: u16 = 0;
pub const ET_REL: u16 = 1;
pub const ET_EXEC: u16 = 2;
pub const ET_DYN: u16 = 3;
pub const ET_CORE: u16 = 4;
pub const ET_NUM: u16 = 5;
pub const ET_LOOS: u16 = 0xfe00;
pub const ET_HIOS: u16 = 0xfeff;
pub const ET_LOPROC: u16 = 0xff00;
pub const ET_HIPROC: u16 = 0xffff;

// elf.h - Legal values for e_machine (architecture).
pub const EM_NONE: u16 = 0;
pub const EM_M32: u16 = 1;
pub const EM_SPARC: u16 = 2;
pub const EM_386: u16 = 3;
pub const EM_68K: u16 = 4;
pub const EM_88K: u16 = 5;
pub const EM_860: u16 = 7;
pub const EM_MIPS: u16 = 8;
pub const EM_S370: u16 = 9;
pub const EM_MIPS_RS3_LE: u16 = 10;
pub const EM_PARISC: u16 = 15;
pub const EM_VPP500: u16 = 17;
pub const EM_SPARC32PLUS: u16 = 18;
pub const EM_960: u16 = 19;
pub const EM_PPC: u16 = 20;
pub const EM_PPC64: u16 = 21;
pub const EM_S390: u16 = 22;
pub const EM_V800: u16 = 36;
pub const EM_FR20: u16 = 37;
pub const EM_RH32: u16 = 38;
pub const EM_RCE: u16 = 39;
pub const EM_ARM: u16 = 40;
pub const EM_FAKE_ALPHA: u16 = 41;
pub const EM_SH: u16 = 42;
pub const EM_SPARCV9: u16 = 43;
pub const EM_TRICORE: u16 = 44;
pub const EM_ARC: u16 = 45;
pub const EM_H8_300: u16 = 46;
pub const EM_H8_300H: u16 = 47;
pub const EM_H8S: u16 = 48;
pub const EM_H8_500: u16 = 49;
pub const EM_IA_64: u16 = 50;
pub const EM_MIPS_X: u16 = 51;
pub const EM_COLDFIRE: u16 = 52;
pub const EM_68HC12: u16 = 53;
pub const EM_MMA: u16 = 54;
pub const EM_PCP: u16 = 55;
pub const EM_NCPU: u16 = 56;
pub const EM_NDR1: u16 = 57;
pub const EM_STARCORE: u16 = 58;
pub const EM_ME16: u16 = 59;
pub const EM_ST100: u16 = 60;
pub const EM_TINYJ: u16 = 61;
pub const EM_X86_64: u16 = 62;
pub const EM_PDSP: u16 = 63;
pub const EM_FX66: u16 = 66;
pub const EM_ST9PLUS: u16 = 67;
pub const EM_ST7: u16 = 68;
pub const EM_68HC16: u16 = 69;
pub const EM_68HC11: u16 = 70;
pub const EM_68HC08: u16 = 71;
pub const EM_68HC05: u16 = 72;
pub const EM_SVX: u16 = 73;
pub const EM_ST19: u16 = 74;
pub const EM_VAX: u16 = 75;
pub const EM_CRIS: u16 = 76;
pub const EM_JAVELIN: u16 = 77;
pub const EM_FIREPATH: u16 = 78;
pub const EM_ZSP: u16 = 79;
pub const EM_MMIX: u16 = 80;
pub const EM_HUANY: u16 = 81;
pub const EM_PRISM: u16 = 82;
pub const EM_AVR: u16 = 83;
pub const EM_FR30: u16 = 84;
pub const EM_D10V: u16 = 85;
pub const EM_D30V: u16 = 86;
pub const EM_V850: u16 = 87;
pub const EM_M32R: u16 = 88;
pub const EM_MN10300: u16 = 89;
pub const EM_MN10200: u16 = 90;
pub const EM_PJ: u16 = 91;
pub const EM_OPENRISC: u16 = 92;
pub const EM_ARC_A5: u16 = 93;
pub const EM_XTENSA: u16 = 94;
pub const EM_AARCH64: u16 = 183;
pub const EM_TILEPRO: u16 = 188;
pub const EM_TILEGX: u16 = 191;
pub const EM_ALPHA: u16 = 0x9026;

// elf.h - Legal values for e_version (version).
pub const EV_NONE: u32 = 0;
pub const EV_CURRENT: u32 = 1;
pub const EV_NUM: u32 = 2;

// elf.h - Legal values for p_type (segment type).
pub const PT_NULL: u32 = 0;
pub const PT_LOAD: u32 = 1;
pub const PT_DYNAMIC: u32 = 2;
pub const PT_INTERP: u32 = 3;
pub const PT_NOTE: u32 = 4;
pub const PT_SHLIB: u32 = 5;
pub const PT_PHDR: u32 = 6;
pub const PT_TLS: u32 = 7;
pub const PT_NUM: u32 = 8;
pub const PT_LOOS: u32 = 0x60000000;
pub const PT_GNU_EH_FRAME: u32 = 0x6474e550;
pub const PT_GNU_STACK: u32 = 0x6474e551;
pub const PT_GNU_RELRO: u32 = 0x6474e552;
pub const PT_LOSUNW: u32 = 0x6ffffffa;
pub const PT_SUNWBSS: u32 = 0x6ffffffa;
pub const PT_SUNWSTACK: u32 = 0x6ffffffb;
pub const PT_HISUNW: u32 = 0x6fffffff;
pub const PT_HIOS: u32 = 0x6fffffff;
pub const PT_LOPROC: u32 = 0x70000000;
pub const PT_HIPROC: u32 = 0x7fffffff;

// Legal values for p_flags (segment flags).
pub const PF_X: u32 = 1 << 0;
pub const PF_W: u32 = 1 << 1;
pub const PF_R: u32 = 1 << 2;
pub const PF_MASKOS: u32 = 0x0ff00000;
pub const PF_MASKPROC: u32 = 0xf0000000;

// elf.h - Legal values for a_type (entry type).
pub const AT_NULL: ::c_ulong = 0;
pub const AT_IGNORE: ::c_ulong = 1;
pub const AT_EXECFD: ::c_ulong = 2;
pub const AT_PHDR: ::c_ulong = 3;
pub const AT_PHENT: ::c_ulong = 4;
pub const AT_PHNUM: ::c_ulong = 5;
pub const AT_PAGESZ: ::c_ulong = 6;
pub const AT_BASE: ::c_ulong = 7;
pub const AT_FLAGS: ::c_ulong = 8;
pub const AT_ENTRY: ::c_ulong = 9;
pub const AT_NOTELF: ::c_ulong = 10;
pub const AT_UID: ::c_ulong = 11;
pub const AT_EUID: ::c_ulong = 12;
pub const AT_GID: ::c_ulong = 13;
pub const AT_EGID: ::c_ulong = 14;
pub const AT_PLATFORM: ::c_ulong = 15;
pub const AT_HWCAP: ::c_ulong = 16;
pub const AT_CLKTCK: ::c_ulong = 17;

pub const AT_SECURE: ::c_ulong = 23;
pub const AT_BASE_PLATFORM: ::c_ulong = 24;
pub const AT_RANDOM: ::c_ulong = 25;
pub const AT_HWCAP2: ::c_ulong = 26;

pub const AT_EXECFN: ::c_ulong = 31;

// defined in arch/<arch>/include/uapi/asm/auxvec.h but has the same value
// wherever it is defined.
pub const AT_SYSINFO_EHDR: ::c_ulong = 33;

pub const GLOB_ERR: ::c_int = 1 << 0;
pub const GLOB_MARK: ::c_int = 1 << 1;
pub const GLOB_NOSORT: ::c_int = 1 << 2;
pub const GLOB_DOOFFS: ::c_int = 1 << 3;
pub const GLOB_NOCHECK: ::c_int = 1 << 4;
pub const GLOB_APPEND: ::c_int = 1 << 5;
pub const GLOB_NOESCAPE: ::c_int = 1 << 6;

pub const GLOB_NOSPACE: ::c_int = 1;
pub const GLOB_ABORTED: ::c_int = 2;
pub const GLOB_NOMATCH: ::c_int = 3;

pub const POSIX_MADV_NORMAL: ::c_int = 0;
pub const POSIX_MADV_RANDOM: ::c_int = 1;
pub const POSIX_MADV_SEQUENTIAL: ::c_int = 2;
pub const POSIX_MADV_WILLNEED: ::c_int = 3;
pub const POSIX_SPAWN_USEVFORK: ::c_int = 64;
pub const POSIX_SPAWN_SETSID: ::c_int = 128;

pub const S_IEXEC: mode_t = 64;
pub const S_IWRITE: mode_t = 128;
pub const S_IREAD: mode_t = 256;

pub const F_LOCK: ::c_int = 1;
pub const F_TEST: ::c_int = 3;
pub const F_TLOCK: ::c_int = 2;
pub const F_ULOCK: ::c_int = 0;

pub const F_SEAL_FUTURE_WRITE: ::c_int = 0x0010;

pub const IFF_LOWER_UP: ::c_int = 0x10000;
pub const IFF_DORMANT: ::c_int = 0x20000;
pub const IFF_ECHO: ::c_int = 0x40000;

// linux/if_addr.h
pub const IFA_UNSPEC: ::c_ushort = 0;
pub const IFA_ADDRESS: ::c_ushort = 1;
pub const IFA_LOCAL: ::c_ushort = 2;
pub const IFA_LABEL: ::c_ushort = 3;
pub const IFA_BROADCAST: ::c_ushort = 4;
pub const IFA_ANYCAST: ::c_ushort = 5;
pub const IFA_CACHEINFO: ::c_ushort = 6;
pub const IFA_MULTICAST: ::c_ushort = 7;

pub const IFA_F_SECONDARY: u32 = 0x01;
pub const IFA_F_TEMPORARY: u32 = 0x01;
pub const IFA_F_NODAD: u32 = 0x02;
pub const IFA_F_OPTIMISTIC: u32 = 0x04;
pub const IFA_F_DADFAILED: u32 = 0x08;
pub const IFA_F_HOMEADDRESS: u32 = 0x10;
pub const IFA_F_DEPRECATED: u32 = 0x20;
pub const IFA_F_TENTATIVE: u32 = 0x40;
pub const IFA_F_PERMANENT: u32 = 0x80;

// linux/if_link.h
pub const IFLA_UNSPEC: ::c_ushort = 0;
pub const IFLA_ADDRESS: ::c_ushort = 1;
pub const IFLA_BROADCAST: ::c_ushort = 2;
pub const IFLA_IFNAME: ::c_ushort = 3;
pub const IFLA_MTU: ::c_ushort = 4;
pub const IFLA_LINK: ::c_ushort = 5;
pub const IFLA_QDISC: ::c_ushort = 6;
pub const IFLA_STATS: ::c_ushort = 7;
pub const IFLA_COST: ::c_ushort = 8;
pub const IFLA_PRIORITY: ::c_ushort = 9;
pub const IFLA_MASTER: ::c_ushort = 10;
pub const IFLA_WIRELESS: ::c_ushort = 11;
pub const IFLA_PROTINFO: ::c_ushort = 12;
pub const IFLA_TXQLEN: ::c_ushort = 13;
pub const IFLA_MAP: ::c_ushort = 14;
pub const IFLA_WEIGHT: ::c_ushort = 15;
pub const IFLA_OPERSTATE: ::c_ushort = 16;
pub const IFLA_LINKMODE: ::c_ushort = 17;
pub const IFLA_LINKINFO: ::c_ushort = 18;
pub const IFLA_NET_NS_PID: ::c_ushort = 19;
pub const IFLA_IFALIAS: ::c_ushort = 20;
pub const IFLA_NUM_VF: ::c_ushort = 21;
pub const IFLA_VFINFO_LIST: ::c_ushort = 22;
pub const IFLA_STATS64: ::c_ushort = 23;
pub const IFLA_VF_PORTS: ::c_ushort = 24;
pub const IFLA_PORT_SELF: ::c_ushort = 25;
pub const IFLA_AF_SPEC: ::c_ushort = 26;
pub const IFLA_GROUP: ::c_ushort = 27;
pub const IFLA_NET_NS_FD: ::c_ushort = 28;
pub const IFLA_EXT_MASK: ::c_ushort = 29;
pub const IFLA_PROMISCUITY: ::c_ushort = 30;
pub const IFLA_NUM_TX_QUEUES: ::c_ushort = 31;
pub const IFLA_NUM_RX_QUEUES: ::c_ushort = 32;
pub const IFLA_CARRIER: ::c_ushort = 33;
pub const IFLA_PHYS_PORT_ID: ::c_ushort = 34;
pub const IFLA_CARRIER_CHANGES: ::c_ushort = 35;
pub const IFLA_PHYS_SWITCH_ID: ::c_ushort = 36;
pub const IFLA_LINK_NETNSID: ::c_ushort = 37;
pub const IFLA_PHYS_PORT_NAME: ::c_ushort = 38;
pub const IFLA_PROTO_DOWN: ::c_ushort = 39;
pub const IFLA_GSO_MAX_SEGS: ::c_ushort = 40;
pub const IFLA_GSO_MAX_SIZE: ::c_ushort = 41;
pub const IFLA_PAD: ::c_ushort = 42;
pub const IFLA_XDP: ::c_ushort = 43;
pub const IFLA_EVENT: ::c_ushort = 44;
pub const IFLA_NEW_NETNSID: ::c_ushort = 45;
pub const IFLA_IF_NETNSID: ::c_ushort = 46;
pub const IFLA_TARGET_NETNSID: ::c_ushort = IFLA_IF_NETNSID;
pub const IFLA_CARRIER_UP_COUNT: ::c_ushort = 47;
pub const IFLA_CARRIER_DOWN_COUNT: ::c_ushort = 48;
pub const IFLA_NEW_IFINDEX: ::c_ushort = 49;
pub const IFLA_MIN_MTU: ::c_ushort = 50;
pub const IFLA_MAX_MTU: ::c_ushort = 51;
pub const IFLA_PROP_LIST: ::c_ushort = 52;
pub const IFLA_ALT_IFNAME: ::c_ushort = 53;
pub const IFLA_PERM_ADDRESS: ::c_ushort = 54;
pub const IFLA_PROTO_DOWN_REASON: ::c_ushort = 55;
pub const IFLA_PARENT_DEV_NAME: ::c_ushort = 56;
pub const IFLA_PARENT_DEV_BUS_NAME: ::c_ushort = 57;
pub const IFLA_GRO_MAX_SIZE: ::c_ushort = 58;
pub const IFLA_TSO_MAX_SIZE: ::c_ushort = 59;
pub const IFLA_TSO_MAX_SEGS: ::c_ushort = 60;
pub const IFLA_ALLMULTI: ::c_ushort = 61;

pub const IFLA_INFO_UNSPEC: ::c_ushort = 0;
pub const IFLA_INFO_KIND: ::c_ushort = 1;
pub const IFLA_INFO_DATA: ::c_ushort = 2;
pub const IFLA_INFO_XSTATS: ::c_ushort = 3;
pub const IFLA_INFO_SLAVE_KIND: ::c_ushort = 4;
pub const IFLA_INFO_SLAVE_DATA: ::c_ushort = 5;

// linux/if_tun.h
pub const IFF_TUN: ::c_int = 0x0001;
pub const IFF_TAP: ::c_int = 0x0002;
pub const IFF_NO_PI: ::c_int = 0x1000;
// Read queue size
pub const TUN_READQ_SIZE: ::c_short = 500;
// TUN device type flags: deprecated. Use IFF_TUN/IFF_TAP instead.
pub const TUN_TUN_DEV: ::c_short = ::IFF_TUN as ::c_short;
pub const TUN_TAP_DEV: ::c_short = ::IFF_TAP as ::c_short;
pub const TUN_TYPE_MASK: ::c_short = 0x000f;
// This flag has no real effect
pub const IFF_ONE_QUEUE: ::c_int = 0x2000;
pub const IFF_VNET_HDR: ::c_int = 0x4000;
pub const IFF_TUN_EXCL: ::c_int = 0x8000;
pub const IFF_MULTI_QUEUE: ::c_int = 0x0100;
pub const IFF_ATTACH_QUEUE: ::c_int = 0x0200;
pub const IFF_DETACH_QUEUE: ::c_int = 0x0400;
// read-only flag
pub const IFF_PERSIST: ::c_int = 0x0800;
pub const IFF_NOFILTER: ::c_int = 0x1000;

// Since Linux 3.1
pub const SEEK_DATA: ::c_int = 3;
pub const SEEK_HOLE: ::c_int = 4;

pub const ST_RDONLY: ::c_ulong = 1;
pub const ST_NOSUID: ::c_ulong = 2;
pub const ST_NODEV: ::c_ulong = 4;
pub const ST_NOEXEC: ::c_ulong = 8;
pub const ST_SYNCHRONOUS: ::c_ulong = 16;
pub const ST_MANDLOCK: ::c_ulong = 64;
pub const ST_WRITE: ::c_ulong = 128;
pub const ST_APPEND: ::c_ulong = 256;
pub const ST_IMMUTABLE: ::c_ulong = 512;
pub const ST_NOATIME: ::c_ulong = 1024;
pub const ST_NODIRATIME: ::c_ulong = 2048;

pub const RTLD_NEXT: *mut ::c_void = -1i64 as *mut ::c_void;
pub const RTLD_DEFAULT: *mut ::c_void = 0i64 as *mut ::c_void;
pub const RTLD_NODELETE: ::c_int = 0x1000;
pub const RTLD_NOW: ::c_int = 0x2;

pub const AT_EACCESS: ::c_int = 0x200;

// linux/mempolicy.h
pub const MPOL_DEFAULT: ::c_int = 0;
pub const MPOL_PREFERRED: ::c_int = 1;
pub const MPOL_BIND: ::c_int = 2;
pub const MPOL_INTERLEAVE: ::c_int = 3;
pub const MPOL_LOCAL: ::c_int = 4;
pub const MPOL_F_NUMA_BALANCING: ::c_int = 1 << 13;
pub const MPOL_F_RELATIVE_NODES: ::c_int = 1 << 14;
pub const MPOL_F_STATIC_NODES: ::c_int = 1 << 15;

// linux/membarrier.h
pub const MEMBARRIER_CMD_QUERY: ::c_int = 0;
pub const MEMBARRIER_CMD_GLOBAL: ::c_int = 1 << 0;
pub const MEMBARRIER_CMD_GLOBAL_EXPEDITED: ::c_int = 1 << 1;
pub const MEMBARRIER_CMD_REGISTER_GLOBAL_EXPEDITED: ::c_int = 1 << 2;
pub const MEMBARRIER_CMD_PRIVATE_EXPEDITED: ::c_int = 1 << 3;
pub const MEMBARRIER_CMD_REGISTER_PRIVATE_EXPEDITED: ::c_int = 1 << 4;
pub const MEMBARRIER_CMD_PRIVATE_EXPEDITED_SYNC_CORE: ::c_int = 1 << 5;
pub const MEMBARRIER_CMD_REGISTER_PRIVATE_EXPEDITED_SYNC_CORE: ::c_int = 1 << 6;
pub const MEMBARRIER_CMD_PRIVATE_EXPEDITED_RSEQ: ::c_int = 1 << 7;
pub const MEMBARRIER_CMD_REGISTER_PRIVATE_EXPEDITED_RSEQ: ::c_int = 1 << 8;

align_const! {
    pub const PTHREAD_MUTEX_INITIALIZER: pthread_mutex_t = pthread_mutex_t {
        size: [0; __SIZEOF_PTHREAD_MUTEX_T],
    };
    pub const PTHREAD_COND_INITIALIZER: pthread_cond_t = pthread_cond_t {
        size: [0; __SIZEOF_PTHREAD_COND_T],
    };
    pub const PTHREAD_RWLOCK_INITIALIZER: pthread_rwlock_t = pthread_rwlock_t {
        size: [0; __SIZEOF_PTHREAD_RWLOCK_T],
    };
}
pub const PTHREAD_MUTEX_NORMAL: ::c_int = 0;
pub const PTHREAD_MUTEX_RECURSIVE: ::c_int = 1;
pub const PTHREAD_MUTEX_ERRORCHECK: ::c_int = 2;
pub const PTHREAD_MUTEX_DEFAULT: ::c_int = PTHREAD_MUTEX_NORMAL;
pub const PTHREAD_MUTEX_STALLED: ::c_int = 0;
pub const PTHREAD_MUTEX_ROBUST: ::c_int = 1;
pub const PTHREAD_PRIO_NONE: ::c_int = 0;
pub const PTHREAD_PRIO_INHERIT: ::c_int = 1;
pub const PTHREAD_PRIO_PROTECT: ::c_int = 2;
pub const PTHREAD_PROCESS_PRIVATE: ::c_int = 0;
pub const PTHREAD_PROCESS_SHARED: ::c_int = 1;
pub const __SIZEOF_PTHREAD_COND_T: usize = 48;

pub const RENAME_NOREPLACE: ::c_uint = 1;
pub const RENAME_EXCHANGE: ::c_uint = 2;
pub const RENAME_WHITEOUT: ::c_uint = 4;

pub const SCHED_OTHER: ::c_int = 0;
pub const SCHED_FIFO: ::c_int = 1;
pub const SCHED_RR: ::c_int = 2;
pub const SCHED_BATCH: ::c_int = 3;
pub const SCHED_IDLE: ::c_int = 5;

pub const SCHED_RESET_ON_FORK: ::c_int = 0x40000000;

pub const CLONE_PIDFD: ::c_int = 0x1000;

// netinet/in.h
// NOTE: These are in addition to the constants defined in src/unix/mod.rs

#[deprecated(
    since = "0.2.80",
    note = "This value was increased in the newer kernel \
            and we'll change this following upstream in the future release. \
            See #1896 for more info."
)]
pub const IPPROTO_MAX: ::c_int = 256;

// System V IPC
pub const IPC_PRIVATE: ::key_t = 0;

pub const IPC_CREAT: ::c_int = 0o1000;
pub const IPC_EXCL: ::c_int = 0o2000;
pub const IPC_NOWAIT: ::c_int = 0o4000;

pub const IPC_RMID: ::c_int = 0;
pub const IPC_SET: ::c_int = 1;
pub const IPC_STAT: ::c_int = 2;
pub const IPC_INFO: ::c_int = 3;
pub const MSG_STAT: ::c_int = 11;
pub const MSG_INFO: ::c_int = 12;
pub const MSG_NOTIFICATION: ::c_int = 0x8000;

pub const MSG_NOERROR: ::c_int = 0o10000;
pub const MSG_EXCEPT: ::c_int = 0o20000;
pub const MSG_ZEROCOPY: ::c_int = 0x4000000;

pub const SHM_R: ::c_int = 0o400;
pub const SHM_W: ::c_int = 0o200;

pub const SHM_RDONLY: ::c_int = 0o10000;
pub const SHM_RND: ::c_int = 0o20000;
pub const SHM_REMAP: ::c_int = 0o40000;

pub const SHM_LOCK: ::c_int = 11;
pub const SHM_UNLOCK: ::c_int = 12;

pub const SHM_HUGETLB: ::c_int = 0o4000;
#[cfg(not(all(target_env = "uclibc", target_arch = "mips")))]
pub const SHM_NORESERVE: ::c_int = 0o10000;

pub const QFMT_VFS_OLD: ::c_int = 1;
pub const QFMT_VFS_V0: ::c_int = 2;
pub const QFMT_VFS_V1: ::c_int = 4;

pub const EFD_SEMAPHORE: ::c_int = 0x1;

pub const LOG_NFACILITIES: ::c_int = 24;

pub const SEM_FAILED: *mut ::sem_t = 0 as *mut sem_t;

pub const RB_AUTOBOOT: ::c_int = 0x01234567u32 as i32;
pub const RB_HALT_SYSTEM: ::c_int = 0xcdef0123u32 as i32;
pub const RB_ENABLE_CAD: ::c_int = 0x89abcdefu32 as i32;
pub const RB_DISABLE_CAD: ::c_int = 0x00000000u32 as i32;
pub const RB_POWER_OFF: ::c_int = 0x4321fedcu32 as i32;
pub const RB_SW_SUSPEND: ::c_int = 0xd000fce2u32 as i32;
pub const RB_KEXEC: ::c_int = 0x45584543u32 as i32;

pub const AI_PASSIVE: ::c_int = 0x0001;
pub const AI_CANONNAME: ::c_int = 0x0002;
pub const AI_NUMERICHOST: ::c_int = 0x0004;
pub const AI_V4MAPPED: ::c_int = 0x0008;
pub const AI_ALL: ::c_int = 0x0010;
pub const AI_ADDRCONFIG: ::c_int = 0x0020;

pub const AI_NUMERICSERV: ::c_int = 0x0400;

pub const EAI_BADFLAGS: ::c_int = -1;
pub const EAI_NONAME: ::c_int = -2;
pub const EAI_AGAIN: ::c_int = -3;
pub const EAI_FAIL: ::c_int = -4;
pub const EAI_NODATA: ::c_int = -5;
pub const EAI_FAMILY: ::c_int = -6;
pub const EAI_SOCKTYPE: ::c_int = -7;
pub const EAI_SERVICE: ::c_int = -8;
pub const EAI_MEMORY: ::c_int = -10;
pub const EAI_SYSTEM: ::c_int = -11;
pub const EAI_OVERFLOW: ::c_int = -12;

pub const NI_NUMERICHOST: ::c_int = 1;
pub const NI_NUMERICSERV: ::c_int = 2;
pub const NI_NOFQDN: ::c_int = 4;
pub const NI_NAMEREQD: ::c_int = 8;
pub const NI_DGRAM: ::c_int = 16;

pub const SYNC_FILE_RANGE_WAIT_BEFORE: ::c_uint = 1;
pub const SYNC_FILE_RANGE_WRITE: ::c_uint = 2;
pub const SYNC_FILE_RANGE_WAIT_AFTER: ::c_uint = 4;

cfg_if! {
    if #[cfg(not(target_env = "uclibc"))] {
        pub const AIO_CANCELED: ::c_int = 0;
        pub const AIO_NOTCANCELED: ::c_int = 1;
        pub const AIO_ALLDONE: ::c_int = 2;
        pub const LIO_READ: ::c_int = 0;
        pub const LIO_WRITE: ::c_int = 1;
        pub const LIO_NOP: ::c_int = 2;
        pub const LIO_WAIT: ::c_int = 0;
        pub const LIO_NOWAIT: ::c_int = 1;
        pub const RUSAGE_THREAD: ::c_int = 1;
        pub const MSG_COPY: ::c_int = 0o40000;
        pub const SHM_EXEC: ::c_int = 0o100000;
        pub const IPV6_MULTICAST_ALL: ::c_int = 29;
        pub const IPV6_ROUTER_ALERT_ISOLATE: ::c_int = 30;
        pub const PACKET_MR_UNICAST: ::c_int = 3;
        pub const PTRACE_EVENT_STOP: ::c_int = 128;
        pub const UDP_SEGMENT: ::c_int = 103;
        pub const UDP_GRO: ::c_int = 104;
    }
}

pub const MREMAP_MAYMOVE: ::c_int = 1;
pub const MREMAP_FIXED: ::c_int = 2;
pub const MREMAP_DONTUNMAP: ::c_int = 4;

pub const PR_SET_PDEATHSIG: ::c_int = 1;
pub const PR_GET_PDEATHSIG: ::c_int = 2;

pub const PR_GET_DUMPABLE: ::c_int = 3;
pub const PR_SET_DUMPABLE: ::c_int = 4;

pub const PR_GET_UNALIGN: ::c_int = 5;
pub const PR_SET_UNALIGN: ::c_int = 6;
pub const PR_UNALIGN_NOPRINT: ::c_int = 1;
pub const PR_UNALIGN_SIGBUS: ::c_int = 2;

pub const PR_GET_KEEPCAPS: ::c_int = 7;
pub const PR_SET_KEEPCAPS: ::c_int = 8;

pub const PR_GET_FPEMU: ::c_int = 9;
pub const PR_SET_FPEMU: ::c_int = 10;
pub const PR_FPEMU_NOPRINT: ::c_int = 1;
pub const PR_FPEMU_SIGFPE: ::c_int = 2;

pub const PR_GET_FPEXC: ::c_int = 11;
pub const PR_SET_FPEXC: ::c_int = 12;
pub const PR_FP_EXC_SW_ENABLE: ::c_int = 0x80;
pub const PR_FP_EXC_DIV: ::c_int = 0x010000;
pub const PR_FP_EXC_OVF: ::c_int = 0x020000;
pub const PR_FP_EXC_UND: ::c_int = 0x040000;
pub const PR_FP_EXC_RES: ::c_int = 0x080000;
pub const PR_FP_EXC_INV: ::c_int = 0x100000;
pub const PR_FP_EXC_DISABLED: ::c_int = 0;
pub const PR_FP_EXC_NONRECOV: ::c_int = 1;
pub const PR_FP_EXC_ASYNC: ::c_int = 2;
pub const PR_FP_EXC_PRECISE: ::c_int = 3;

pub const PR_GET_TIMING: ::c_int = 13;
pub const PR_SET_TIMING: ::c_int = 14;
pub const PR_TIMING_STATISTICAL: ::c_int = 0;
pub const PR_TIMING_TIMESTAMP: ::c_int = 1;

pub const PR_SET_NAME: ::c_int = 15;
pub const PR_GET_NAME: ::c_int = 16;

pub const PR_GET_ENDIAN: ::c_int = 19;
pub const PR_SET_ENDIAN: ::c_int = 20;
pub const PR_ENDIAN_BIG: ::c_int = 0;
pub const PR_ENDIAN_LITTLE: ::c_int = 1;
pub const PR_ENDIAN_PPC_LITTLE: ::c_int = 2;

pub const PR_GET_SECCOMP: ::c_int = 21;
pub const PR_SET_SECCOMP: ::c_int = 22;

pub const PR_CAPBSET_READ: ::c_int = 23;
pub const PR_CAPBSET_DROP: ::c_int = 24;

pub const PR_GET_TSC: ::c_int = 25;
pub const PR_SET_TSC: ::c_int = 26;
pub const PR_TSC_ENABLE: ::c_int = 1;
pub const PR_TSC_SIGSEGV: ::c_int = 2;

pub const PR_GET_SECUREBITS: ::c_int = 27;
pub const PR_SET_SECUREBITS: ::c_int = 28;

pub const PR_SET_TIMERSLACK: ::c_int = 29;
pub const PR_GET_TIMERSLACK: ::c_int = 30;

pub const PR_TASK_PERF_EVENTS_DISABLE: ::c_int = 31;
pub const PR_TASK_PERF_EVENTS_ENABLE: ::c_int = 32;

pub const PR_MCE_KILL: ::c_int = 33;
pub const PR_MCE_KILL_CLEAR: ::c_int = 0;
pub const PR_MCE_KILL_SET: ::c_int = 1;

pub const PR_MCE_KILL_LATE: ::c_int = 0;
pub const PR_MCE_KILL_EARLY: ::c_int = 1;
pub const PR_MCE_KILL_DEFAULT: ::c_int = 2;

pub const PR_MCE_KILL_GET: ::c_int = 34;

pub const PR_SET_MM: ::c_int = 35;
pub const PR_SET_MM_START_CODE: ::c_int = 1;
pub const PR_SET_MM_END_CODE: ::c_int = 2;
pub const PR_SET_MM_START_DATA: ::c_int = 3;
pub const PR_SET_MM_END_DATA: ::c_int = 4;
pub const PR_SET_MM_START_STACK: ::c_int = 5;
pub const PR_SET_MM_START_BRK: ::c_int = 6;
pub const PR_SET_MM_BRK: ::c_int = 7;
pub const PR_SET_MM_ARG_START: ::c_int = 8;
pub const PR_SET_MM_ARG_END: ::c_int = 9;
pub const PR_SET_MM_ENV_START: ::c_int = 10;
pub const PR_SET_MM_ENV_END: ::c_int = 11;
pub const PR_SET_MM_AUXV: ::c_int = 12;
pub const PR_SET_MM_EXE_FILE: ::c_int = 13;
pub const PR_SET_MM_MAP: ::c_int = 14;
pub const PR_SET_MM_MAP_SIZE: ::c_int = 15;

pub const PR_SET_PTRACER: ::c_int = 0x59616d61;
pub const PR_SET_PTRACER_ANY: ::c_ulong = 0xffffffffffffffff;

pub const PR_SET_CHILD_SUBREAPER: ::c_int = 36;
pub const PR_GET_CHILD_SUBREAPER: ::c_int = 37;

pub const PR_SET_NO_NEW_PRIVS: ::c_int = 38;
pub const PR_GET_NO_NEW_PRIVS: ::c_int = 39;

pub const PR_GET_TID_ADDRESS: ::c_int = 40;

pub const PR_SET_THP_DISABLE: ::c_int = 41;
pub const PR_GET_THP_DISABLE: ::c_int = 42;

pub const PR_MPX_ENABLE_MANAGEMENT: ::c_int = 43;
pub const PR_MPX_DISABLE_MANAGEMENT: ::c_int = 44;

pub const PR_SET_FP_MODE: ::c_int = 45;
pub const PR_GET_FP_MODE: ::c_int = 46;
pub const PR_FP_MODE_FR: ::c_int = 1 << 0;
pub const PR_FP_MODE_FRE: ::c_int = 1 << 1;

pub const PR_CAP_AMBIENT: ::c_int = 47;
pub const PR_CAP_AMBIENT_IS_SET: ::c_int = 1;
pub const PR_CAP_AMBIENT_RAISE: ::c_int = 2;
pub const PR_CAP_AMBIENT_LOWER: ::c_int = 3;
pub const PR_CAP_AMBIENT_CLEAR_ALL: ::c_int = 4;

pub const PR_SET_VMA: ::c_int = 0x53564d41;
pub const PR_SET_VMA_ANON_NAME: ::c_int = 0;

pub const PR_SCHED_CORE: ::c_int = 62;
pub const PR_SCHED_CORE_GET: ::c_int = 0;
pub const PR_SCHED_CORE_CREATE: ::c_int = 1;
pub const PR_SCHED_CORE_SHARE_TO: ::c_int = 2;
pub const PR_SCHED_CORE_SHARE_FROM: ::c_int = 3;
pub const PR_SCHED_CORE_MAX: ::c_int = 4;
pub const PR_SCHED_CORE_SCOPE_THREAD: ::c_int = 0;
pub const PR_SCHED_CORE_SCOPE_THREAD_GROUP: ::c_int = 1;
pub const PR_SCHED_CORE_SCOPE_PROCESS_GROUP: ::c_int = 2;

pub const GRND_NONBLOCK: ::c_uint = 0x0001;
pub const GRND_RANDOM: ::c_uint = 0x0002;
pub const GRND_INSECURE: ::c_uint = 0x0004;

pub const SECCOMP_MODE_DISABLED: ::c_uint = 0;
pub const SECCOMP_MODE_STRICT: ::c_uint = 1;
pub const SECCOMP_MODE_FILTER: ::c_uint = 2;

pub const SECCOMP_FILTER_FLAG_TSYNC: ::c_ulong = 1;
pub const SECCOMP_FILTER_FLAG_LOG: ::c_ulong = 2;
pub const SECCOMP_FILTER_FLAG_SPEC_ALLOW: ::c_ulong = 4;

pub const SECCOMP_RET_KILL_PROCESS: ::c_uint = 0x80000000;
pub const SECCOMP_RET_KILL_THREAD: ::c_uint = 0x00000000;
pub const SECCOMP_RET_KILL: ::c_uint = SECCOMP_RET_KILL_THREAD;
pub const SECCOMP_RET_TRAP: ::c_uint = 0x00030000;
pub const SECCOMP_RET_ERRNO: ::c_uint = 0x00050000;
pub const SECCOMP_RET_TRACE: ::c_uint = 0x7ff00000;
pub const SECCOMP_RET_LOG: ::c_uint = 0x7ffc0000;
pub const SECCOMP_RET_ALLOW: ::c_uint = 0x7fff0000;

pub const SECCOMP_RET_ACTION_FULL: ::c_uint = 0xffff0000;
pub const SECCOMP_RET_ACTION: ::c_uint = 0x7fff0000;
pub const SECCOMP_RET_DATA: ::c_uint = 0x0000ffff;

pub const ITIMER_REAL: ::c_int = 0;
pub const ITIMER_VIRTUAL: ::c_int = 1;
pub const ITIMER_PROF: ::c_int = 2;

pub const TFD_CLOEXEC: ::c_int = O_CLOEXEC;
pub const TFD_NONBLOCK: ::c_int = O_NONBLOCK;
pub const TFD_TIMER_ABSTIME: ::c_int = 1;
pub const TFD_TIMER_CANCEL_ON_SET: ::c_int = 2;

pub const _POSIX_VDISABLE: ::cc_t = 0;

pub const FALLOC_FL_KEEP_SIZE: ::c_int = 0x01;
pub const FALLOC_FL_PUNCH_HOLE: ::c_int = 0x02;
pub const FALLOC_FL_COLLAPSE_RANGE: ::c_int = 0x08;
pub const FALLOC_FL_ZERO_RANGE: ::c_int = 0x10;
pub const FALLOC_FL_INSERT_RANGE: ::c_int = 0x20;
pub const FALLOC_FL_UNSHARE_RANGE: ::c_int = 0x40;

#[deprecated(
    since = "0.2.55",
    note = "ENOATTR is not available on Linux; use ENODATA instead"
)]
pub const ENOATTR: ::c_int = ::ENODATA;

pub const SO_ORIGINAL_DST: ::c_int = 80;

pub const IP_RECVFRAGSIZE: ::c_int = 25;

pub const IPV6_FLOWINFO: ::c_int = 11;
pub const IPV6_FLOWLABEL_MGR: ::c_int = 32;
pub const IPV6_FLOWINFO_SEND: ::c_int = 33;
pub const IPV6_RECVFRAGSIZE: ::c_int = 77;
pub const IPV6_FREEBIND: ::c_int = 78;
pub const IPV6_FLOWINFO_FLOWLABEL: ::c_int = 0x000fffff;
pub const IPV6_FLOWINFO_PRIORITY: ::c_int = 0x0ff00000;

pub const IPV6_RTHDR_LOOSE: ::c_int = 0;
pub const IPV6_RTHDR_STRICT: ::c_int = 1;

// SO_MEMINFO offsets
pub const SK_MEMINFO_RMEM_ALLOC: ::c_int = 0;
pub const SK_MEMINFO_RCVBUF: ::c_int = 1;
pub const SK_MEMINFO_WMEM_ALLOC: ::c_int = 2;
pub const SK_MEMINFO_SNDBUF: ::c_int = 3;
pub const SK_MEMINFO_FWD_ALLOC: ::c_int = 4;
pub const SK_MEMINFO_WMEM_QUEUED: ::c_int = 5;
pub const SK_MEMINFO_OPTMEM: ::c_int = 6;
pub const SK_MEMINFO_BACKLOG: ::c_int = 7;
pub const SK_MEMINFO_DROPS: ::c_int = 8;

pub const IUTF8: ::tcflag_t = 0x00004000;
#[cfg(not(all(target_env = "uclibc", target_arch = "mips")))]
pub const CMSPAR: ::tcflag_t = 0o10000000000;

pub const MFD_CLOEXEC: ::c_uint = 0x0001;
pub const MFD_ALLOW_SEALING: ::c_uint = 0x0002;
pub const MFD_HUGETLB: ::c_uint = 0x0004;
pub const MFD_HUGE_64KB: ::c_uint = 0x40000000;
pub const MFD_HUGE_512KB: ::c_uint = 0x4c000000;
pub const MFD_HUGE_1MB: ::c_uint = 0x50000000;
pub const MFD_HUGE_2MB: ::c_uint = 0x54000000;
pub const MFD_HUGE_8MB: ::c_uint = 0x5c000000;
pub const MFD_HUGE_16MB: ::c_uint = 0x60000000;
pub const MFD_HUGE_32MB: ::c_uint = 0x64000000;
pub const MFD_HUGE_256MB: ::c_uint = 0x70000000;
pub const MFD_HUGE_512MB: ::c_uint = 0x74000000;
pub const MFD_HUGE_1GB: ::c_uint = 0x78000000;
pub const MFD_HUGE_2GB: ::c_uint = 0x7c000000;
pub const MFD_HUGE_16GB: ::c_uint = 0x88000000;
pub const MFD_HUGE_MASK: ::c_uint = 63;
pub const MFD_HUGE_SHIFT: ::c_uint = 26;

// linux/close_range.h
pub const CLOSE_RANGE_UNSHARE: ::c_uint = 1 << 1;
pub const CLOSE_RANGE_CLOEXEC: ::c_uint = 1 << 2;

// linux/filter.h
pub const SKF_AD_OFF: ::c_int = -0x1000;
pub const SKF_AD_PROTOCOL: ::c_int = 0;
pub const SKF_AD_PKTTYPE: ::c_int = 4;
pub const SKF_AD_IFINDEX: ::c_int = 8;
pub const SKF_AD_NLATTR: ::c_int = 12;
pub const SKF_AD_NLATTR_NEST: ::c_int = 16;
pub const SKF_AD_MARK: ::c_int = 20;
pub const SKF_AD_QUEUE: ::c_int = 24;
pub const SKF_AD_HATYPE: ::c_int = 28;
pub const SKF_AD_RXHASH: ::c_int = 32;
pub const SKF_AD_CPU: ::c_int = 36;
pub const SKF_AD_ALU_XOR_X: ::c_int = 40;
pub const SKF_AD_VLAN_TAG: ::c_int = 44;
pub const SKF_AD_VLAN_TAG_PRESENT: ::c_int = 48;
pub const SKF_AD_PAY_OFFSET: ::c_int = 52;
pub const SKF_AD_RANDOM: ::c_int = 56;
pub const SKF_AD_VLAN_TPID: ::c_int = 60;
pub const SKF_AD_MAX: ::c_int = 64;
pub const SKF_NET_OFF: ::c_int = -0x100000;
pub const SKF_LL_OFF: ::c_int = -0x200000;
pub const BPF_NET_OFF: ::c_int = SKF_NET_OFF;
pub const BPF_LL_OFF: ::c_int = SKF_LL_OFF;
pub const BPF_MEMWORDS: ::c_int = 16;
pub const BPF_MAXINSNS: ::c_int = 4096;

// linux/bpf_common.h
pub const BPF_LD: ::__u32 = 0x00;
pub const BPF_LDX: ::__u32 = 0x01;
pub const BPF_ST: ::__u32 = 0x02;
pub const BPF_STX: ::__u32 = 0x03;
pub const BPF_ALU: ::__u32 = 0x04;
pub const BPF_JMP: ::__u32 = 0x05;
pub const BPF_RET: ::__u32 = 0x06;
pub const BPF_MISC: ::__u32 = 0x07;
pub const BPF_W: ::__u32 = 0x00;
pub const BPF_H: ::__u32 = 0x08;
pub const BPF_B: ::__u32 = 0x10;
pub const BPF_IMM: ::__u32 = 0x00;
pub const BPF_ABS: ::__u32 = 0x20;
pub const BPF_IND: ::__u32 = 0x40;
pub const BPF_MEM: ::__u32 = 0x60;
pub const BPF_LEN: ::__u32 = 0x80;
pub const BPF_MSH: ::__u32 = 0xa0;
pub const BPF_ADD: ::__u32 = 0x00;
pub const BPF_SUB: ::__u32 = 0x10;
pub const BPF_MUL: ::__u32 = 0x20;
pub const BPF_DIV: ::__u32 = 0x30;
pub const BPF_OR: ::__u32 = 0x40;
pub const BPF_AND: ::__u32 = 0x50;
pub const BPF_LSH: ::__u32 = 0x60;
pub const BPF_RSH: ::__u32 = 0x70;
pub const BPF_NEG: ::__u32 = 0x80;
pub const BPF_MOD: ::__u32 = 0x90;
pub const BPF_XOR: ::__u32 = 0xa0;
pub const BPF_JA: ::__u32 = 0x00;
pub const BPF_JEQ: ::__u32 = 0x10;
pub const BPF_JGT: ::__u32 = 0x20;
pub const BPF_JGE: ::__u32 = 0x30;
pub const BPF_JSET: ::__u32 = 0x40;
pub const BPF_K: ::__u32 = 0x00;
pub const BPF_X: ::__u32 = 0x08;

// linux/openat2.h
pub const RESOLVE_NO_XDEV: ::__u64 = 0x01;
pub const RESOLVE_NO_MAGICLINKS: ::__u64 = 0x02;
pub const RESOLVE_NO_SYMLINKS: ::__u64 = 0x04;
pub const RESOLVE_BENEATH: ::__u64 = 0x08;
pub const RESOLVE_IN_ROOT: ::__u64 = 0x10;
pub const RESOLVE_CACHED: ::__u64 = 0x20;

// linux/if_ether.h
pub const ETH_ALEN: ::c_int = 6;
pub const ETH_HLEN: ::c_int = 14;
pub const ETH_ZLEN: ::c_int = 60;
pub const ETH_DATA_LEN: ::c_int = 1500;
pub const ETH_FRAME_LEN: ::c_int = 1514;
pub const ETH_FCS_LEN: ::c_int = 4;

// These are the defined Ethernet Protocol ID's.
pub const ETH_P_LOOP: ::c_int = 0x0060;
pub const ETH_P_PUP: ::c_int = 0x0200;
pub const ETH_P_PUPAT: ::c_int = 0x0201;
pub const ETH_P_IP: ::c_int = 0x0800;
pub const ETH_P_X25: ::c_int = 0x0805;
pub const ETH_P_ARP: ::c_int = 0x0806;
pub const ETH_P_BPQ: ::c_int = 0x08FF;
pub const ETH_P_IEEEPUP: ::c_int = 0x0a00;
pub const ETH_P_IEEEPUPAT: ::c_int = 0x0a01;
pub const ETH_P_BATMAN: ::c_int = 0x4305;
pub const ETH_P_DEC: ::c_int = 0x6000;
pub const ETH_P_DNA_DL: ::c_int = 0x6001;
pub const ETH_P_DNA_RC: ::c_int = 0x6002;
pub const ETH_P_DNA_RT: ::c_int = 0x6003;
pub const ETH_P_LAT: ::c_int = 0x6004;
pub const ETH_P_DIAG: ::c_int = 0x6005;
pub const ETH_P_CUST: ::c_int = 0x6006;
pub const ETH_P_SCA: ::c_int = 0x6007;
pub const ETH_P_TEB: ::c_int = 0x6558;
pub const ETH_P_RARP: ::c_int = 0x8035;
pub const ETH_P_ATALK: ::c_int = 0x809B;
pub const ETH_P_AARP: ::c_int = 0x80F3;
pub const ETH_P_8021Q: ::c_int = 0x8100;
pub const ETH_P_IPX: ::c_int = 0x8137;
pub const ETH_P_IPV6: ::c_int = 0x86DD;
pub const ETH_P_PAUSE: ::c_int = 0x8808;
pub const ETH_P_SLOW: ::c_int = 0x8809;
pub const ETH_P_WCCP: ::c_int = 0x883E;
pub const ETH_P_MPLS_UC: ::c_int = 0x8847;
pub const ETH_P_MPLS_MC: ::c_int = 0x8848;
pub const ETH_P_ATMMPOA: ::c_int = 0x884c;
pub const ETH_P_PPP_DISC: ::c_int = 0x8863;
pub const ETH_P_PPP_SES: ::c_int = 0x8864;
pub const ETH_P_LINK_CTL: ::c_int = 0x886c;
pub const ETH_P_ATMFATE: ::c_int = 0x8884;
pub const ETH_P_PAE: ::c_int = 0x888E;
pub const ETH_P_AOE: ::c_int = 0x88A2;
pub const ETH_P_8021AD: ::c_int = 0x88A8;
pub const ETH_P_802_EX1: ::c_int = 0x88B5;
pub const ETH_P_TIPC: ::c_int = 0x88CA;
pub const ETH_P_MACSEC: ::c_int = 0x88E5;
pub const ETH_P_8021AH: ::c_int = 0x88E7;
pub const ETH_P_MVRP: ::c_int = 0x88F5;
pub const ETH_P_1588: ::c_int = 0x88F7;
pub const ETH_P_PRP: ::c_int = 0x88FB;
pub const ETH_P_FCOE: ::c_int = 0x8906;
pub const ETH_P_TDLS: ::c_int = 0x890D;
pub const ETH_P_FIP: ::c_int = 0x8914;
pub const ETH_P_80221: ::c_int = 0x8917;
pub const ETH_P_LOOPBACK: ::c_int = 0x9000;
pub const ETH_P_QINQ1: ::c_int = 0x9100;
pub const ETH_P_QINQ2: ::c_int = 0x9200;
pub const ETH_P_QINQ3: ::c_int = 0x9300;
pub const ETH_P_EDSA: ::c_int = 0xDADA;
pub const ETH_P_AF_IUCV: ::c_int = 0xFBFB;

pub const ETH_P_802_3_MIN: ::c_int = 0x0600;

// Non DIX types. Won't clash for 1500 types.
pub const ETH_P_802_3: ::c_int = 0x0001;
pub const ETH_P_AX25: ::c_int = 0x0002;
pub const ETH_P_ALL: ::c_int = 0x0003;
pub const ETH_P_802_2: ::c_int = 0x0004;
pub const ETH_P_SNAP: ::c_int = 0x0005;
pub const ETH_P_DDCMP: ::c_int = 0x0006;
pub const ETH_P_WAN_PPP: ::c_int = 0x0007;
pub const ETH_P_PPP_MP: ::c_int = 0x0008;
pub const ETH_P_LOCALTALK: ::c_int = 0x0009;
pub const ETH_P_CANFD: ::c_int = 0x000D;
pub const ETH_P_PPPTALK: ::c_int = 0x0010;
pub const ETH_P_TR_802_2: ::c_int = 0x0011;
pub const ETH_P_MOBITEX: ::c_int = 0x0015;
pub const ETH_P_CONTROL: ::c_int = 0x0016;
pub const ETH_P_IRDA: ::c_int = 0x0017;
pub const ETH_P_ECONET: ::c_int = 0x0018;
pub const ETH_P_HDLC: ::c_int = 0x0019;
pub const ETH_P_ARCNET: ::c_int = 0x001A;
pub const ETH_P_DSA: ::c_int = 0x001B;
pub const ETH_P_TRAILER: ::c_int = 0x001C;
pub const ETH_P_PHONET: ::c_int = 0x00F5;
pub const ETH_P_IEEE802154: ::c_int = 0x00F6;
pub const ETH_P_CAIF: ::c_int = 0x00F7;

pub const POSIX_SPAWN_RESETIDS: ::c_int = 0x01;
pub const POSIX_SPAWN_SETPGROUP: ::c_int = 0x02;
pub const POSIX_SPAWN_SETSIGDEF: ::c_int = 0x04;
pub const POSIX_SPAWN_SETSIGMASK: ::c_int = 0x08;
pub const POSIX_SPAWN_SETSCHEDPARAM: ::c_int = 0x10;
pub const POSIX_SPAWN_SETSCHEDULER: ::c_int = 0x20;

pub const NLMSG_NOOP: ::c_int = 0x1;
pub const NLMSG_ERROR: ::c_int = 0x2;
pub const NLMSG_DONE: ::c_int = 0x3;
pub const NLMSG_OVERRUN: ::c_int = 0x4;
pub const NLMSG_MIN_TYPE: ::c_int = 0x10;

// linux/netfilter/nfnetlink.h
pub const NFNLGRP_NONE: ::c_int = 0;
pub const NFNLGRP_CONNTRACK_NEW: ::c_int = 1;
pub const NFNLGRP_CONNTRACK_UPDATE: ::c_int = 2;
pub const NFNLGRP_CONNTRACK_DESTROY: ::c_int = 3;
pub const NFNLGRP_CONNTRACK_EXP_NEW: ::c_int = 4;
pub const NFNLGRP_CONNTRACK_EXP_UPDATE: ::c_int = 5;
pub const NFNLGRP_CONNTRACK_EXP_DESTROY: ::c_int = 6;
pub const NFNLGRP_NFTABLES: ::c_int = 7;
pub const NFNLGRP_ACCT_QUOTA: ::c_int = 8;
pub const NFNLGRP_NFTRACE: ::c_int = 9;

pub const NFNETLINK_V0: ::c_int = 0;

pub const NFNL_SUBSYS_NONE: ::c_int = 0;
pub const NFNL_SUBSYS_CTNETLINK: ::c_int = 1;
pub const NFNL_SUBSYS_CTNETLINK_EXP: ::c_int = 2;
pub const NFNL_SUBSYS_QUEUE: ::c_int = 3;
pub const NFNL_SUBSYS_ULOG: ::c_int = 4;
pub const NFNL_SUBSYS_OSF: ::c_int = 5;
pub const NFNL_SUBSYS_IPSET: ::c_int = 6;
pub const NFNL_SUBSYS_ACCT: ::c_int = 7;
pub const NFNL_SUBSYS_CTNETLINK_TIMEOUT: ::c_int = 8;
pub const NFNL_SUBSYS_CTHELPER: ::c_int = 9;
pub const NFNL_SUBSYS_NFTABLES: ::c_int = 10;
pub const NFNL_SUBSYS_NFT_COMPAT: ::c_int = 11;
pub const NFNL_SUBSYS_HOOK: ::c_int = 12;
pub const NFNL_SUBSYS_COUNT: ::c_int = 13;

pub const NFNL_MSG_BATCH_BEGIN: ::c_int = NLMSG_MIN_TYPE;
pub const NFNL_MSG_BATCH_END: ::c_int = NLMSG_MIN_TYPE + 1;

pub const NFNL_BATCH_UNSPEC: ::c_int = 0;
pub const NFNL_BATCH_GENID: ::c_int = 1;

// linux/netfilter/nfnetlink_log.h
pub const NFULNL_MSG_PACKET: ::c_int = 0;
pub const NFULNL_MSG_CONFIG: ::c_int = 1;

pub const NFULA_VLAN_UNSPEC: ::c_int = 0;
pub const NFULA_VLAN_PROTO: ::c_int = 1;
pub const NFULA_VLAN_TCI: ::c_int = 2;

pub const NFULA_UNSPEC: ::c_int = 0;
pub const NFULA_PACKET_HDR: ::c_int = 1;
pub const NFULA_MARK: ::c_int = 2;
pub const NFULA_TIMESTAMP: ::c_int = 3;
pub const NFULA_IFINDEX_INDEV: ::c_int = 4;
pub const NFULA_IFINDEX_OUTDEV: ::c_int = 5;
pub const NFULA_IFINDEX_PHYSINDEV: ::c_int = 6;
pub const NFULA_IFINDEX_PHYSOUTDEV: ::c_int = 7;
pub const NFULA_HWADDR: ::c_int = 8;
pub const NFULA_PAYLOAD: ::c_int = 9;
pub const NFULA_PREFIX: ::c_int = 10;
pub const NFULA_UID: ::c_int = 11;
pub const NFULA_SEQ: ::c_int = 12;
pub const NFULA_SEQ_GLOBAL: ::c_int = 13;
pub const NFULA_GID: ::c_int = 14;
pub const NFULA_HWTYPE: ::c_int = 15;
pub const NFULA_HWHEADER: ::c_int = 16;
pub const NFULA_HWLEN: ::c_int = 17;
pub const NFULA_CT: ::c_int = 18;
pub const NFULA_CT_INFO: ::c_int = 19;
pub const NFULA_VLAN: ::c_int = 20;
pub const NFULA_L2HDR: ::c_int = 21;

pub const NFULNL_CFG_CMD_NONE: ::c_int = 0;
pub const NFULNL_CFG_CMD_BIND: ::c_int = 1;
pub const NFULNL_CFG_CMD_UNBIND: ::c_int = 2;
pub const NFULNL_CFG_CMD_PF_BIND: ::c_int = 3;
pub const NFULNL_CFG_CMD_PF_UNBIND: ::c_int = 4;

pub const NFULA_CFG_UNSPEC: ::c_int = 0;
pub const NFULA_CFG_CMD: ::c_int = 1;
pub const NFULA_CFG_MODE: ::c_int = 2;
pub const NFULA_CFG_NLBUFSIZ: ::c_int = 3;
pub const NFULA_CFG_TIMEOUT: ::c_int = 4;
pub const NFULA_CFG_QTHRESH: ::c_int = 5;
pub const NFULA_CFG_FLAGS: ::c_int = 6;

pub const NFULNL_COPY_NONE: ::c_int = 0x00;
pub const NFULNL_COPY_META: ::c_int = 0x01;
pub const NFULNL_COPY_PACKET: ::c_int = 0x02;

pub const NFULNL_CFG_F_SEQ: ::c_int = 0x0001;
pub const NFULNL_CFG_F_SEQ_GLOBAL: ::c_int = 0x0002;
pub const NFULNL_CFG_F_CONNTRACK: ::c_int = 0x0004;

// linux/netfilter/nfnetlink_queue.h
pub const NFQNL_MSG_PACKET: ::c_int = 0;
pub const NFQNL_MSG_VERDICT: ::c_int = 1;
pub const NFQNL_MSG_CONFIG: ::c_int = 2;
pub const NFQNL_MSG_VERDICT_BATCH: ::c_int = 3;

pub const NFQA_UNSPEC: ::c_int = 0;
pub const NFQA_PACKET_HDR: ::c_int = 1;
pub const NFQA_VERDICT_HDR: ::c_int = 2;
pub const NFQA_MARK: ::c_int = 3;
pub const NFQA_TIMESTAMP: ::c_int = 4;
pub const NFQA_IFINDEX_INDEV: ::c_int = 5;
pub const NFQA_IFINDEX_OUTDEV: ::c_int = 6;
pub const NFQA_IFINDEX_PHYSINDEV: ::c_int = 7;
pub const NFQA_IFINDEX_PHYSOUTDEV: ::c_int = 8;
pub const NFQA_HWADDR: ::c_int = 9;
pub const NFQA_PAYLOAD: ::c_int = 10;
pub const NFQA_CT: ::c_int = 11;
pub const NFQA_CT_INFO: ::c_int = 12;
pub const NFQA_CAP_LEN: ::c_int = 13;
pub const NFQA_SKB_INFO: ::c_int = 14;
pub const NFQA_EXP: ::c_int = 15;
pub const NFQA_UID: ::c_int = 16;
pub const NFQA_GID: ::c_int = 17;
pub const NFQA_SECCTX: ::c_int = 18;
pub const NFQA_VLAN: ::c_int = 19;
pub const NFQA_L2HDR: ::c_int = 20;
pub const NFQA_PRIORITY: ::c_int = 21;

pub const NFQA_VLAN_UNSPEC: ::c_int = 0;
pub const NFQA_VLAN_PROTO: ::c_int = 1;
pub const NFQA_VLAN_TCI: ::c_int = 2;

pub const NFQNL_CFG_CMD_NONE: ::c_int = 0;
pub const NFQNL_CFG_CMD_BIND: ::c_int = 1;
pub const NFQNL_CFG_CMD_UNBIND: ::c_int = 2;
pub const NFQNL_CFG_CMD_PF_BIND: ::c_int = 3;
pub const NFQNL_CFG_CMD_PF_UNBIND: ::c_int = 4;

pub const NFQNL_COPY_NONE: ::c_int = 0;
pub const NFQNL_COPY_META: ::c_int = 1;
pub const NFQNL_COPY_PACKET: ::c_int = 2;

pub const NFQA_CFG_UNSPEC: ::c_int = 0;
pub const NFQA_CFG_CMD: ::c_int = 1;
pub const NFQA_CFG_PARAMS: ::c_int = 2;
pub const NFQA_CFG_QUEUE_MAXLEN: ::c_int = 3;
pub const NFQA_CFG_MASK: ::c_int = 4;
pub const NFQA_CFG_FLAGS: ::c_int = 5;

pub const NFQA_CFG_F_FAIL_OPEN: ::c_int = 0x0001;
pub const NFQA_CFG_F_CONNTRACK: ::c_int = 0x0002;
pub const NFQA_CFG_F_GSO: ::c_int = 0x0004;
pub const NFQA_CFG_F_UID_GID: ::c_int = 0x0008;
pub const NFQA_CFG_F_SECCTX: ::c_int = 0x0010;
pub const NFQA_CFG_F_MAX: ::c_int = 0x0020;

pub const NFQA_SKB_CSUMNOTREADY: ::c_int = 0x0001;
pub const NFQA_SKB_GSO: ::c_int = 0x0002;
pub const NFQA_SKB_CSUM_NOTVERIFIED: ::c_int = 0x0004;

// linux/genetlink.h

pub const GENL_NAMSIZ: ::c_int = 16;

pub const GENL_MIN_ID: ::c_int = NLMSG_MIN_TYPE;
pub const GENL_MAX_ID: ::c_int = 1023;

pub const GENL_ADMIN_PERM: ::c_int = 0x01;
pub const GENL_CMD_CAP_DO: ::c_int = 0x02;
pub const GENL_CMD_CAP_DUMP: ::c_int = 0x04;
pub const GENL_CMD_CAP_HASPOL: ::c_int = 0x08;

pub const GENL_ID_CTRL: ::c_int = NLMSG_MIN_TYPE;

pub const CTRL_CMD_UNSPEC: ::c_int = 0;
pub const CTRL_CMD_NEWFAMILY: ::c_int = 1;
pub const CTRL_CMD_DELFAMILY: ::c_int = 2;
pub const CTRL_CMD_GETFAMILY: ::c_int = 3;
pub const CTRL_CMD_NEWOPS: ::c_int = 4;
pub const CTRL_CMD_DELOPS: ::c_int = 5;
pub const CTRL_CMD_GETOPS: ::c_int = 6;
pub const CTRL_CMD_NEWMCAST_GRP: ::c_int = 7;
pub const CTRL_CMD_DELMCAST_GRP: ::c_int = 8;
pub const CTRL_CMD_GETMCAST_GRP: ::c_int = 9;

pub const CTRL_ATTR_UNSPEC: ::c_int = 0;
pub const CTRL_ATTR_FAMILY_ID: ::c_int = 1;
pub const CTRL_ATTR_FAMILY_NAME: ::c_int = 2;
pub const CTRL_ATTR_VERSION: ::c_int = 3;
pub const CTRL_ATTR_HDRSIZE: ::c_int = 4;
pub const CTRL_ATTR_MAXATTR: ::c_int = 5;
pub const CTRL_ATTR_OPS: ::c_int = 6;
pub const CTRL_ATTR_MCAST_GROUPS: ::c_int = 7;

pub const CTRL_ATTR_OP_UNSPEC: ::c_int = 0;
pub const CTRL_ATTR_OP_ID: ::c_int = 1;
pub const CTRL_ATTR_OP_FLAGS: ::c_int = 2;

pub const CTRL_ATTR_MCAST_GRP_UNSPEC: ::c_int = 0;
pub const CTRL_ATTR_MCAST_GRP_NAME: ::c_int = 1;
pub const CTRL_ATTR_MCAST_GRP_ID: ::c_int = 2;

// linux/if_packet.h
pub const PACKET_ADD_MEMBERSHIP: ::c_int = 1;
pub const PACKET_DROP_MEMBERSHIP: ::c_int = 2;

pub const PACKET_MR_MULTICAST: ::c_int = 0;
pub const PACKET_MR_PROMISC: ::c_int = 1;
pub const PACKET_MR_ALLMULTI: ::c_int = 2;

// linux/netfilter.h
pub const NF_DROP: ::c_int = 0;
pub const NF_ACCEPT: ::c_int = 1;
pub const NF_STOLEN: ::c_int = 2;
pub const NF_QUEUE: ::c_int = 3;
pub const NF_REPEAT: ::c_int = 4;
pub const NF_STOP: ::c_int = 5;
pub const NF_MAX_VERDICT: ::c_int = NF_STOP;

pub const NF_VERDICT_MASK: ::c_int = 0x000000ff;
pub const NF_VERDICT_FLAG_QUEUE_BYPASS: ::c_int = 0x00008000;

pub const NF_VERDICT_QMASK: ::c_int = 0xffff0000;
pub const NF_VERDICT_QBITS: ::c_int = 16;

pub const NF_VERDICT_BITS: ::c_int = 16;

pub const NF_INET_PRE_ROUTING: ::c_int = 0;
pub const NF_INET_LOCAL_IN: ::c_int = 1;
pub const NF_INET_FORWARD: ::c_int = 2;
pub const NF_INET_LOCAL_OUT: ::c_int = 3;
pub const NF_INET_POST_ROUTING: ::c_int = 4;
pub const NF_INET_NUMHOOKS: ::c_int = 5;

// Some NFPROTO are not compatible with musl and are defined in submodules.
pub const NFPROTO_UNSPEC: ::c_int = 0;
pub const NFPROTO_IPV4: ::c_int = 2;
pub const NFPROTO_ARP: ::c_int = 3;
pub const NFPROTO_BRIDGE: ::c_int = 7;
pub const NFPROTO_IPV6: ::c_int = 10;
pub const NFPROTO_DECNET: ::c_int = 12;
pub const NFPROTO_NUMPROTO: ::c_int = 13;
pub const NFPROTO_INET: ::c_int = 1;
pub const NFPROTO_NETDEV: ::c_int = 5;

pub const NF_NETDEV_INGRESS: ::c_int = 0;
pub const NF_NETDEV_NUMHOOKS: ::c_int = 1;

// linux/netfilter_ipv4.h
pub const NF_IP_PRE_ROUTING: ::c_int = 0;
pub const NF_IP_LOCAL_IN: ::c_int = 1;
pub const NF_IP_FORWARD: ::c_int = 2;
pub const NF_IP_LOCAL_OUT: ::c_int = 3;
pub const NF_IP_POST_ROUTING: ::c_int = 4;
pub const NF_IP_NUMHOOKS: ::c_int = 5;

pub const NF_IP_PRI_FIRST: ::c_int = ::INT_MIN;
pub const NF_IP_PRI_CONNTRACK_DEFRAG: ::c_int = -400;
pub const NF_IP_PRI_RAW: ::c_int = -300;
pub const NF_IP_PRI_SELINUX_FIRST: ::c_int = -225;
pub const NF_IP_PRI_CONNTRACK: ::c_int = -200;
pub const NF_IP_PRI_MANGLE: ::c_int = -150;
pub const NF_IP_PRI_NAT_DST: ::c_int = -100;
pub const NF_IP_PRI_FILTER: ::c_int = 0;
pub const NF_IP_PRI_SECURITY: ::c_int = 50;
pub const NF_IP_PRI_NAT_SRC: ::c_int = 100;
pub const NF_IP_PRI_SELINUX_LAST: ::c_int = 225;
pub const NF_IP_PRI_CONNTRACK_HELPER: ::c_int = 300;
pub const NF_IP_PRI_CONNTRACK_CONFIRM: ::c_int = ::INT_MAX;
pub const NF_IP_PRI_LAST: ::c_int = ::INT_MAX;

// linux/netfilter_ipv6.h
pub const NF_IP6_PRE_ROUTING: ::c_int = 0;
pub const NF_IP6_LOCAL_IN: ::c_int = 1;
pub const NF_IP6_FORWARD: ::c_int = 2;
pub const NF_IP6_LOCAL_OUT: ::c_int = 3;
pub const NF_IP6_POST_ROUTING: ::c_int = 4;
pub const NF_IP6_NUMHOOKS: ::c_int = 5;

pub const NF_IP6_PRI_FIRST: ::c_int = ::INT_MIN;
pub const NF_IP6_PRI_CONNTRACK_DEFRAG: ::c_int = -400;
pub const NF_IP6_PRI_RAW: ::c_int = -300;
pub const NF_IP6_PRI_SELINUX_FIRST: ::c_int = -225;
pub const NF_IP6_PRI_CONNTRACK: ::c_int = -200;
pub const NF_IP6_PRI_MANGLE: ::c_int = -150;
pub const NF_IP6_PRI_NAT_DST: ::c_int = -100;
pub const NF_IP6_PRI_FILTER: ::c_int = 0;
pub const NF_IP6_PRI_SECURITY: ::c_int = 50;
pub const NF_IP6_PRI_NAT_SRC: ::c_int = 100;
pub const NF_IP6_PRI_SELINUX_LAST: ::c_int = 225;
pub const NF_IP6_PRI_CONNTRACK_HELPER: ::c_int = 300;
pub const NF_IP6_PRI_LAST: ::c_int = ::INT_MAX;

// linux/netfilter_ipv6/ip6_tables.h
pub const IP6T_SO_ORIGINAL_DST: ::c_int = 80;

pub const SIOCADDRT: ::c_ulong = 0x0000890B;
pub const SIOCDELRT: ::c_ulong = 0x0000890C;
pub const SIOCGIFNAME: ::c_ulong = 0x00008910;
pub const SIOCSIFLINK: ::c_ulong = 0x00008911;
pub const SIOCGIFCONF: ::c_ulong = 0x00008912;
pub const SIOCGIFFLAGS: ::c_ulong = 0x00008913;
pub const SIOCSIFFLAGS: ::c_ulong = 0x00008914;
pub const SIOCGIFADDR: ::c_ulong = 0x00008915;
pub const SIOCSIFADDR: ::c_ulong = 0x00008916;
pub const SIOCGIFDSTADDR: ::c_ulong = 0x00008917;
pub const SIOCSIFDSTADDR: ::c_ulong = 0x00008918;
pub const SIOCGIFBRDADDR: ::c_ulong = 0x00008919;
pub const SIOCSIFBRDADDR: ::c_ulong = 0x0000891A;
pub const SIOCGIFNETMASK: ::c_ulong = 0x0000891B;
pub const SIOCSIFNETMASK: ::c_ulong = 0x0000891C;
pub const SIOCGIFMETRIC: ::c_ulong = 0x0000891D;
pub const SIOCSIFMETRIC: ::c_ulong = 0x0000891E;
pub const SIOCGIFMEM: ::c_ulong = 0x0000891F;
pub const SIOCSIFMEM: ::c_ulong = 0x00008920;
pub const SIOCGIFMTU: ::c_ulong = 0x00008921;
pub const SIOCSIFMTU: ::c_ulong = 0x00008922;
pub const SIOCSIFHWADDR: ::c_ulong = 0x00008924;
pub const SIOCGIFENCAP: ::c_ulong = 0x00008925;
pub const SIOCSIFENCAP: ::c_ulong = 0x00008926;
pub const SIOCGIFHWADDR: ::c_ulong = 0x00008927;
pub const SIOCGIFSLAVE: ::c_ulong = 0x00008929;
pub const SIOCSIFSLAVE: ::c_ulong = 0x00008930;
pub const SIOCADDMULTI: ::c_ulong = 0x00008931;
pub const SIOCDELMULTI: ::c_ulong = 0x00008932;
pub const SIOCGIFINDEX: ::c_ulong = 0x00008933;
pub const SIOGIFINDEX: ::c_ulong = SIOCGIFINDEX;
pub const SIOCSIFPFLAGS: ::c_ulong = 0x00008934;
pub const SIOCGIFPFLAGS: ::c_ulong = 0x00008935;
pub const SIOCDIFADDR: ::c_ulong = 0x00008936;
pub const SIOCSIFHWBROADCAST: ::c_ulong = 0x00008937;
pub const SIOCGIFCOUNT: ::c_ulong = 0x00008938;
pub const SIOCGIFBR: ::c_ulong = 0x00008940;
pub const SIOCSIFBR: ::c_ulong = 0x00008941;
pub const SIOCGIFTXQLEN: ::c_ulong = 0x00008942;
pub const SIOCSIFTXQLEN: ::c_ulong = 0x00008943;
pub const SIOCETHTOOL: ::c_ulong = 0x00008946;
pub const SIOCGMIIPHY: ::c_ulong = 0x00008947;
pub const SIOCGMIIREG: ::c_ulong = 0x00008948;
pub const SIOCSMIIREG: ::c_ulong = 0x00008949;
pub const SIOCWANDEV: ::c_ulong = 0x0000894A;
pub const SIOCOUTQNSD: ::c_ulong = 0x0000894B;
pub const SIOCGSKNS: ::c_ulong = 0x0000894C;
pub const SIOCDARP: ::c_ulong = 0x00008953;
pub const SIOCGARP: ::c_ulong = 0x00008954;
pub const SIOCSARP: ::c_ulong = 0x00008955;
pub const SIOCDRARP: ::c_ulong = 0x00008960;
pub const SIOCGRARP: ::c_ulong = 0x00008961;
pub const SIOCSRARP: ::c_ulong = 0x00008962;
pub const SIOCGIFMAP: ::c_ulong = 0x00008970;
pub const SIOCSIFMAP: ::c_ulong = 0x00008971;
pub const SIOCSHWTSTAMP: ::c_ulong = 0x000089b0;
pub const SIOCGHWTSTAMP: ::c_ulong = 0x000089b1;

pub const IPTOS_TOS_MASK: u8 = 0x1E;
pub const IPTOS_PREC_MASK: u8 = 0xE0;

pub const IPTOS_ECN_NOT_ECT: u8 = 0x00;

pub const RTF_UP: ::c_ushort = 0x0001;
pub const RTF_GATEWAY: ::c_ushort = 0x0002;

pub const RTF_HOST: ::c_ushort = 0x0004;
pub const RTF_REINSTATE: ::c_ushort = 0x0008;
pub const RTF_DYNAMIC: ::c_ushort = 0x0010;
pub const RTF_MODIFIED: ::c_ushort = 0x0020;
pub const RTF_MTU: ::c_ushort = 0x0040;
pub const RTF_MSS: ::c_ushort = RTF_MTU;
pub const RTF_WINDOW: ::c_ushort = 0x0080;
pub const RTF_IRTT: ::c_ushort = 0x0100;
pub const RTF_REJECT: ::c_ushort = 0x0200;
pub const RTF_STATIC: ::c_ushort = 0x0400;
pub const RTF_XRESOLVE: ::c_ushort = 0x0800;
pub const RTF_NOFORWARD: ::c_ushort = 0x1000;
pub const RTF_THROW: ::c_ushort = 0x2000;
pub const RTF_NOPMTUDISC: ::c_ushort = 0x4000;

pub const RTF_DEFAULT: u32 = 0x00010000;
pub const RTF_ALLONLINK: u32 = 0x00020000;
pub const RTF_ADDRCONF: u32 = 0x00040000;
pub const RTF_LINKRT: u32 = 0x00100000;
pub const RTF_NONEXTHOP: u32 = 0x00200000;
pub const RTF_CACHE: u32 = 0x01000000;
pub const RTF_FLOW: u32 = 0x02000000;
pub const RTF_POLICY: u32 = 0x04000000;

pub const RTCF_VALVE: u32 = 0x00200000;
pub const RTCF_MASQ: u32 = 0x00400000;
pub const RTCF_NAT: u32 = 0x00800000;
pub const RTCF_DOREDIRECT: u32 = 0x01000000;
pub const RTCF_LOG: u32 = 0x02000000;
pub const RTCF_DIRECTSRC: u32 = 0x04000000;

pub const RTF_LOCAL: u32 = 0x80000000;
pub const RTF_INTERFACE: u32 = 0x40000000;
pub const RTF_MULTICAST: u32 = 0x20000000;
pub const RTF_BROADCAST: u32 = 0x10000000;
pub const RTF_NAT: u32 = 0x08000000;
pub const RTF_ADDRCLASSMASK: u32 = 0xF8000000;

pub const RT_CLASS_UNSPEC: u8 = 0;
pub const RT_CLASS_DEFAULT: u8 = 253;
pub const RT_CLASS_MAIN: u8 = 254;
pub const RT_CLASS_LOCAL: u8 = 255;
pub const RT_CLASS_MAX: u8 = 255;

// linux/neighbor.h
pub const NUD_NONE: u16 = 0x00;
pub const NUD_INCOMPLETE: u16 = 0x01;
pub const NUD_REACHABLE: u16 = 0x02;
pub const NUD_STALE: u16 = 0x04;
pub const NUD_DELAY: u16 = 0x08;
pub const NUD_PROBE: u16 = 0x10;
pub const NUD_FAILED: u16 = 0x20;
pub const NUD_NOARP: u16 = 0x40;
pub const NUD_PERMANENT: u16 = 0x80;

pub const NTF_USE: u8 = 0x01;
pub const NTF_SELF: u8 = 0x02;
pub const NTF_MASTER: u8 = 0x04;
pub const NTF_PROXY: u8 = 0x08;
pub const NTF_ROUTER: u8 = 0x80;

pub const NDA_UNSPEC: ::c_ushort = 0;
pub const NDA_DST: ::c_ushort = 1;
pub const NDA_LLADDR: ::c_ushort = 2;
pub const NDA_CACHEINFO: ::c_ushort = 3;
pub const NDA_PROBES: ::c_ushort = 4;
pub const NDA_VLAN: ::c_ushort = 5;
pub const NDA_PORT: ::c_ushort = 6;
pub const NDA_VNI: ::c_ushort = 7;
pub const NDA_IFINDEX: ::c_ushort = 8;

// linux/netlink.h
pub const NLA_ALIGNTO: ::c_int = 4;

pub const NETLINK_ROUTE: ::c_int = 0;
pub const NETLINK_UNUSED: ::c_int = 1;
pub const NETLINK_USERSOCK: ::c_int = 2;
pub const NETLINK_FIREWALL: ::c_int = 3;
pub const NETLINK_SOCK_DIAG: ::c_int = 4;
pub const NETLINK_NFLOG: ::c_int = 5;
pub const NETLINK_XFRM: ::c_int = 6;
pub const NETLINK_SELINUX: ::c_int = 7;
pub const NETLINK_ISCSI: ::c_int = 8;
pub const NETLINK_AUDIT: ::c_int = 9;
pub const NETLINK_FIB_LOOKUP: ::c_int = 10;
pub const NETLINK_CONNECTOR: ::c_int = 11;
pub const NETLINK_NETFILTER: ::c_int = 12;
pub const NETLINK_IP6_FW: ::c_int = 13;
pub const NETLINK_DNRTMSG: ::c_int = 14;
pub const NETLINK_KOBJECT_UEVENT: ::c_int = 15;
pub const NETLINK_GENERIC: ::c_int = 16;
pub const NETLINK_SCSITRANSPORT: ::c_int = 18;
pub const NETLINK_ECRYPTFS: ::c_int = 19;
pub const NETLINK_RDMA: ::c_int = 20;
pub const NETLINK_CRYPTO: ::c_int = 21;
pub const NETLINK_INET_DIAG: ::c_int = NETLINK_SOCK_DIAG;

pub const NLM_F_REQUEST: ::c_int = 1;
pub const NLM_F_MULTI: ::c_int = 2;
pub const NLM_F_ACK: ::c_int = 4;
pub const NLM_F_ECHO: ::c_int = 8;
pub const NLM_F_DUMP_INTR: ::c_int = 16;
pub const NLM_F_DUMP_FILTERED: ::c_int = 32;

pub const NLM_F_ROOT: ::c_int = 0x100;
pub const NLM_F_MATCH: ::c_int = 0x200;
pub const NLM_F_ATOMIC: ::c_int = 0x400;
pub const NLM_F_DUMP: ::c_int = NLM_F_ROOT | NLM_F_MATCH;

pub const NLM_F_REPLACE: ::c_int = 0x100;
pub const NLM_F_EXCL: ::c_int = 0x200;
pub const NLM_F_CREATE: ::c_int = 0x400;
pub const NLM_F_APPEND: ::c_int = 0x800;

pub const NETLINK_ADD_MEMBERSHIP: ::c_int = 1;
pub const NETLINK_DROP_MEMBERSHIP: ::c_int = 2;
pub const NETLINK_PKTINFO: ::c_int = 3;
pub const NETLINK_BROADCAST_ERROR: ::c_int = 4;
pub const NETLINK_NO_ENOBUFS: ::c_int = 5;
pub const NETLINK_RX_RING: ::c_int = 6;
pub const NETLINK_TX_RING: ::c_int = 7;
pub const NETLINK_LISTEN_ALL_NSID: ::c_int = 8;
pub const NETLINK_LIST_MEMBERSHIPS: ::c_int = 9;
pub const NETLINK_CAP_ACK: ::c_int = 10;
pub const NETLINK_EXT_ACK: ::c_int = 11;
pub const NETLINK_GET_STRICT_CHK: ::c_int = 12;

pub const NLA_F_NESTED: ::c_int = 1 << 15;
pub const NLA_F_NET_BYTEORDER: ::c_int = 1 << 14;
pub const NLA_TYPE_MASK: ::c_int = !(NLA_F_NESTED | NLA_F_NET_BYTEORDER);

// linux/rtnetlink.h
pub const TCA_UNSPEC: ::c_ushort = 0;
pub const TCA_KIND: ::c_ushort = 1;
pub const TCA_OPTIONS: ::c_ushort = 2;
pub const TCA_STATS: ::c_ushort = 3;
pub const TCA_XSTATS: ::c_ushort = 4;
pub const TCA_RATE: ::c_ushort = 5;
pub const TCA_FCNT: ::c_ushort = 6;
pub const TCA_STATS2: ::c_ushort = 7;
pub const TCA_STAB: ::c_ushort = 8;

pub const RTM_NEWLINK: u16 = 16;
pub const RTM_DELLINK: u16 = 17;
pub const RTM_GETLINK: u16 = 18;
pub const RTM_SETLINK: u16 = 19;
pub const RTM_NEWADDR: u16 = 20;
pub const RTM_DELADDR: u16 = 21;
pub const RTM_GETADDR: u16 = 22;
pub const RTM_NEWROUTE: u16 = 24;
pub const RTM_DELROUTE: u16 = 25;
pub const RTM_GETROUTE: u16 = 26;
pub const RTM_NEWNEIGH: u16 = 28;
pub const RTM_DELNEIGH: u16 = 29;
pub const RTM_GETNEIGH: u16 = 30;
pub const RTM_NEWRULE: u16 = 32;
pub const RTM_DELRULE: u16 = 33;
pub const RTM_GETRULE: u16 = 34;
pub const RTM_NEWQDISC: u16 = 36;
pub const RTM_DELQDISC: u16 = 37;
pub const RTM_GETQDISC: u16 = 38;
pub const RTM_NEWTCLASS: u16 = 40;
pub const RTM_DELTCLASS: u16 = 41;
pub const RTM_GETTCLASS: u16 = 42;
pub const RTM_NEWTFILTER: u16 = 44;
pub const RTM_DELTFILTER: u16 = 45;
pub const RTM_GETTFILTER: u16 = 46;
pub const RTM_NEWACTION: u16 = 48;
pub const RTM_DELACTION: u16 = 49;
pub const RTM_GETACTION: u16 = 50;
pub const RTM_NEWPREFIX: u16 = 52;
pub const RTM_GETMULTICAST: u16 = 58;
pub const RTM_GETANYCAST: u16 = 62;
pub const RTM_NEWNEIGHTBL: u16 = 64;
pub const RTM_GETNEIGHTBL: u16 = 66;
pub const RTM_SETNEIGHTBL: u16 = 67;
pub const RTM_NEWNDUSEROPT: u16 = 68;
pub const RTM_NEWADDRLABEL: u16 = 72;
pub const RTM_DELADDRLABEL: u16 = 73;
pub const RTM_GETADDRLABEL: u16 = 74;
pub const RTM_GETDCB: u16 = 78;
pub const RTM_SETDCB: u16 = 79;
pub const RTM_NEWNETCONF: u16 = 80;
pub const RTM_GETNETCONF: u16 = 82;
pub const RTM_NEWMDB: u16 = 84;
pub const RTM_DELMDB: u16 = 85;
pub const RTM_GETMDB: u16 = 86;
pub const RTM_NEWNSID: u16 = 88;
pub const RTM_DELNSID: u16 = 89;
pub const RTM_GETNSID: u16 = 90;

pub const RTM_F_NOTIFY: ::c_uint = 0x100;
pub const RTM_F_CLONED: ::c_uint = 0x200;
pub const RTM_F_EQUALIZE: ::c_uint = 0x400;
pub const RTM_F_PREFIX: ::c_uint = 0x800;

pub const RTA_UNSPEC: ::c_ushort = 0;
pub const RTA_DST: ::c_ushort = 1;
pub const RTA_SRC: ::c_ushort = 2;
pub const RTA_IIF: ::c_ushort = 3;
pub const RTA_OIF: ::c_ushort = 4;
pub const RTA_GATEWAY: ::c_ushort = 5;
pub const RTA_PRIORITY: ::c_ushort = 6;
pub const RTA_PREFSRC: ::c_ushort = 7;
pub const RTA_METRICS: ::c_ushort = 8;
pub const RTA_MULTIPATH: ::c_ushort = 9;
pub const RTA_PROTOINFO: ::c_ushort = 10; // No longer used
pub const RTA_FLOW: ::c_ushort = 11;
pub const RTA_CACHEINFO: ::c_ushort = 12;
pub const RTA_SESSION: ::c_ushort = 13; // No longer used
pub const RTA_MP_ALGO: ::c_ushort = 14; // No longer used
pub const RTA_TABLE: ::c_ushort = 15;
pub const RTA_MARK: ::c_ushort = 16;
pub const RTA_MFC_STATS: ::c_ushort = 17;

pub const RTN_UNSPEC: ::c_uchar = 0;
pub const RTN_UNICAST: ::c_uchar = 1;
pub const RTN_LOCAL: ::c_uchar = 2;
pub const RTN_BROADCAST: ::c_uchar = 3;
pub const RTN_ANYCAST: ::c_uchar = 4;
pub const RTN_MULTICAST: ::c_uchar = 5;
pub const RTN_BLACKHOLE: ::c_uchar = 6;
pub const RTN_UNREACHABLE: ::c_uchar = 7;
pub const RTN_PROHIBIT: ::c_uchar = 8;
pub const RTN_THROW: ::c_uchar = 9;
pub const RTN_NAT: ::c_uchar = 10;
pub const RTN_XRESOLVE: ::c_uchar = 11;

pub const RTPROT_UNSPEC: ::c_uchar = 0;
pub const RTPROT_REDIRECT: ::c_uchar = 1;
pub const RTPROT_KERNEL: ::c_uchar = 2;
pub const RTPROT_BOOT: ::c_uchar = 3;
pub const RTPROT_STATIC: ::c_uchar = 4;

pub const RT_SCOPE_UNIVERSE: ::c_uchar = 0;
pub const RT_SCOPE_SITE: ::c_uchar = 200;
pub const RT_SCOPE_LINK: ::c_uchar = 253;
pub const RT_SCOPE_HOST: ::c_uchar = 254;
pub const RT_SCOPE_NOWHERE: ::c_uchar = 255;

pub const RT_TABLE_UNSPEC: ::c_uchar = 0;
pub const RT_TABLE_COMPAT: ::c_uchar = 252;
pub const RT_TABLE_DEFAULT: ::c_uchar = 253;
pub const RT_TABLE_MAIN: ::c_uchar = 254;
pub const RT_TABLE_LOCAL: ::c_uchar = 255;

pub const RTMSG_OVERRUN: u32 = ::NLMSG_OVERRUN as u32;
pub const RTMSG_NEWDEVICE: u32 = 0x11;
pub const RTMSG_DELDEVICE: u32 = 0x12;
pub const RTMSG_NEWROUTE: u32 = 0x21;
pub const RTMSG_DELROUTE: u32 = 0x22;
pub const RTMSG_NEWRULE: u32 = 0x31;
pub const RTMSG_DELRULE: u32 = 0x32;
pub const RTMSG_CONTROL: u32 = 0x40;
pub const RTMSG_AR_FAILED: u32 = 0x51;

pub const MAX_ADDR_LEN: usize = 7;
pub const ARPD_UPDATE: ::c_ushort = 0x01;
pub const ARPD_LOOKUP: ::c_ushort = 0x02;
pub const ARPD_FLUSH: ::c_ushort = 0x03;
pub const ATF_MAGIC: ::c_int = 0x80;

pub const RTEXT_FILTER_VF: ::c_int = 1 << 0;
pub const RTEXT_FILTER_BRVLAN: ::c_int = 1 << 1;
pub const RTEXT_FILTER_BRVLAN_COMPRESSED: ::c_int = 1 << 2;
pub const RTEXT_FILTER_SKIP_STATS: ::c_int = 1 << 3;
pub const RTEXT_FILTER_MRP: ::c_int = 1 << 4;
pub const RTEXT_FILTER_CFM_CONFIG: ::c_int = 1 << 5;
pub const RTEXT_FILTER_CFM_STATUS: ::c_int = 1 << 6;

// userspace compat definitions for RTNLGRP_*
pub const RTMGRP_LINK: ::c_int = 0x00001;
pub const RTMGRP_NOTIFY: ::c_int = 0x00002;
pub const RTMGRP_NEIGH: ::c_int = 0x00004;
pub const RTMGRP_TC: ::c_int = 0x00008;
pub const RTMGRP_IPV4_IFADDR: ::c_int = 0x00010;
pub const RTMGRP_IPV4_MROUTE: ::c_int = 0x00020;
pub const RTMGRP_IPV4_ROUTE: ::c_int = 0x00040;
pub const RTMGRP_IPV4_RULE: ::c_int = 0x00080;
pub const RTMGRP_IPV6_IFADDR: ::c_int = 0x00100;
pub const RTMGRP_IPV6_MROUTE: ::c_int = 0x00200;
pub const RTMGRP_IPV6_ROUTE: ::c_int = 0x00400;
pub const RTMGRP_IPV6_IFINFO: ::c_int = 0x00800;
pub const RTMGRP_DECnet_IFADDR: ::c_int = 0x01000;
pub const RTMGRP_DECnet_ROUTE: ::c_int = 0x04000;
pub const RTMGRP_IPV6_PREFIX: ::c_int = 0x20000;

// enum rtnetlink_groups
pub const RTNLGRP_NONE: ::c_uint = 0x00;
pub const RTNLGRP_LINK: ::c_uint = 0x01;
pub const RTNLGRP_NOTIFY: ::c_uint = 0x02;
pub const RTNLGRP_NEIGH: ::c_uint = 0x03;
pub const RTNLGRP_TC: ::c_uint = 0x04;
pub const RTNLGRP_IPV4_IFADDR: ::c_uint = 0x05;
pub const RTNLGRP_IPV4_MROUTE: ::c_uint = 0x06;
pub const RTNLGRP_IPV4_ROUTE: ::c_uint = 0x07;
pub const RTNLGRP_IPV4_RULE: ::c_uint = 0x08;
pub const RTNLGRP_IPV6_IFADDR: ::c_uint = 0x09;
pub const RTNLGRP_IPV6_MROUTE: ::c_uint = 0x0a;
pub const RTNLGRP_IPV6_ROUTE: ::c_uint = 0x0b;
pub const RTNLGRP_IPV6_IFINFO: ::c_uint = 0x0c;
pub const RTNLGRP_DECnet_IFADDR: ::c_uint = 0x0d;
pub const RTNLGRP_NOP2: ::c_uint = 0x0e;
pub const RTNLGRP_DECnet_ROUTE: ::c_uint = 0x0f;
pub const RTNLGRP_DECnet_RULE: ::c_uint = 0x10;
pub const RTNLGRP_NOP4: ::c_uint = 0x11;
pub const RTNLGRP_IPV6_PREFIX: ::c_uint = 0x12;
pub const RTNLGRP_IPV6_RULE: ::c_uint = 0x13;
pub const RTNLGRP_ND_USEROPT: ::c_uint = 0x14;
pub const RTNLGRP_PHONET_IFADDR: ::c_uint = 0x15;
pub const RTNLGRP_PHONET_ROUTE: ::c_uint = 0x16;
pub const RTNLGRP_DCB: ::c_uint = 0x17;
pub const RTNLGRP_IPV4_NETCONF: ::c_uint = 0x18;
pub const RTNLGRP_IPV6_NETCONF: ::c_uint = 0x19;
pub const RTNLGRP_MDB: ::c_uint = 0x1a;
pub const RTNLGRP_MPLS_ROUTE: ::c_uint = 0x1b;
pub const RTNLGRP_NSID: ::c_uint = 0x1c;
pub const RTNLGRP_MPLS_NETCONF: ::c_uint = 0x1d;
pub const RTNLGRP_IPV4_MROUTE_R: ::c_uint = 0x1e;
pub const RTNLGRP_IPV6_MROUTE_R: ::c_uint = 0x1f;
pub const RTNLGRP_NEXTHOP: ::c_uint = 0x20;
pub const RTNLGRP_BRVLAN: ::c_uint = 0x21;
pub const RTNLGRP_MCTP_IFADDR: ::c_uint = 0x22;
pub const RTNLGRP_TUNNEL: ::c_uint = 0x23;
pub const RTNLGRP_STATS: ::c_uint = 0x24;

// linux/module.h
pub const MODULE_INIT_IGNORE_MODVERSIONS: ::c_uint = 0x0001;
pub const MODULE_INIT_IGNORE_VERMAGIC: ::c_uint = 0x0002;

// linux/net_tstamp.h
pub const SOF_TIMESTAMPING_TX_HARDWARE: ::c_uint = 1 << 0;
pub const SOF_TIMESTAMPING_TX_SOFTWARE: ::c_uint = 1 << 1;
pub const SOF_TIMESTAMPING_RX_HARDWARE: ::c_uint = 1 << 2;
pub const SOF_TIMESTAMPING_RX_SOFTWARE: ::c_uint = 1 << 3;
pub const SOF_TIMESTAMPING_SOFTWARE: ::c_uint = 1 << 4;
pub const SOF_TIMESTAMPING_SYS_HARDWARE: ::c_uint = 1 << 5;
pub const SOF_TIMESTAMPING_RAW_HARDWARE: ::c_uint = 1 << 6;
pub const SOF_TIMESTAMPING_OPT_ID: ::c_uint = 1 << 7;
pub const SOF_TIMESTAMPING_TX_SCHED: ::c_uint = 1 << 8;
pub const SOF_TIMESTAMPING_TX_ACK: ::c_uint = 1 << 9;
pub const SOF_TIMESTAMPING_OPT_CMSG: ::c_uint = 1 << 10;
pub const SOF_TIMESTAMPING_OPT_TSONLY: ::c_uint = 1 << 11;
pub const SOF_TIMESTAMPING_OPT_STATS: ::c_uint = 1 << 12;
pub const SOF_TIMESTAMPING_OPT_PKTINFO: ::c_uint = 1 << 13;
pub const SOF_TIMESTAMPING_OPT_TX_SWHW: ::c_uint = 1 << 14;
pub const SOF_TXTIME_DEADLINE_MODE: u32 = 1 << 0;
pub const SOF_TXTIME_REPORT_ERRORS: u32 = 1 << 1;

pub const HWTSTAMP_TX_OFF: ::c_uint = 0;
pub const HWTSTAMP_TX_ON: ::c_uint = 1;
pub const HWTSTAMP_TX_ONESTEP_SYNC: ::c_uint = 2;
pub const HWTSTAMP_TX_ONESTEP_P2P: ::c_uint = 3;

pub const HWTSTAMP_FILTER_NONE: ::c_uint = 0;
pub const HWTSTAMP_FILTER_ALL: ::c_uint = 1;
pub const HWTSTAMP_FILTER_SOME: ::c_uint = 2;
pub const HWTSTAMP_FILTER_PTP_V1_L4_EVENT: ::c_uint = 3;
pub const HWTSTAMP_FILTER_PTP_V1_L4_SYNC: ::c_uint = 4;
pub const HWTSTAMP_FILTER_PTP_V1_L4_DELAY_REQ: ::c_uint = 5;
pub const HWTSTAMP_FILTER_PTP_V2_L4_EVENT: ::c_uint = 6;
pub const HWTSTAMP_FILTER_PTP_V2_L4_SYNC: ::c_uint = 7;
pub const HWTSTAMP_FILTER_PTP_V2_L4_DELAY_REQ: ::c_uint = 8;
pub const HWTSTAMP_FILTER_PTP_V2_L2_EVENT: ::c_uint = 9;
pub const HWTSTAMP_FILTER_PTP_V2_L2_SYNC: ::c_uint = 10;
pub const HWTSTAMP_FILTER_PTP_V2_L2_DELAY_REQ: ::c_uint = 11;
pub const HWTSTAMP_FILTER_PTP_V2_EVENT: ::c_uint = 12;
pub const HWTSTAMP_FILTER_PTP_V2_SYNC: ::c_uint = 13;
pub const HWTSTAMP_FILTER_PTP_V2_DELAY_REQ: ::c_uint = 14;
pub const HWTSTAMP_FILTER_NTP_ALL: ::c_uint = 15;

// linux/if_alg.h
pub const ALG_SET_KEY: ::c_int = 1;
pub const ALG_SET_IV: ::c_int = 2;
pub const ALG_SET_OP: ::c_int = 3;
pub const ALG_SET_AEAD_ASSOCLEN: ::c_int = 4;
pub const ALG_SET_AEAD_AUTHSIZE: ::c_int = 5;

pub const ALG_OP_DECRYPT: ::c_int = 0;
pub const ALG_OP_ENCRYPT: ::c_int = 1;

// include/uapi/linux/if.h
pub const IF_OPER_UNKNOWN: ::c_int = 0;
pub const IF_OPER_NOTPRESENT: ::c_int = 1;
pub const IF_OPER_DOWN: ::c_int = 2;
pub const IF_OPER_LOWERLAYERDOWN: ::c_int = 3;
pub const IF_OPER_TESTING: ::c_int = 4;
pub const IF_OPER_DORMANT: ::c_int = 5;
pub const IF_OPER_UP: ::c_int = 6;

pub const IF_LINK_MODE_DEFAULT: ::c_int = 0;
pub const IF_LINK_MODE_DORMANT: ::c_int = 1;
pub const IF_LINK_MODE_TESTING: ::c_int = 2;

// include/uapi/linux/udp.h
pub const UDP_CORK: ::c_int = 1;
pub const UDP_ENCAP: ::c_int = 100;
pub const UDP_NO_CHECK6_TX: ::c_int = 101;
pub const UDP_NO_CHECK6_RX: ::c_int = 102;

// include/uapi/linux/mman.h
pub const MAP_SHARED_VALIDATE: ::c_int = 0x3;

// include/uapi/asm-generic/mman-common.h
pub const MAP_FIXED_NOREPLACE: ::c_int = 0x100000;
pub const MLOCK_ONFAULT: ::c_uint = 0x01;

// uapi/linux/vm_sockets.h
pub const VMADDR_CID_ANY: ::c_uint = 0xFFFFFFFF;
pub const VMADDR_CID_HYPERVISOR: ::c_uint = 0;
#[deprecated(
    since = "0.2.74",
    note = "VMADDR_CID_RESERVED is removed since Linux v5.6 and \
            replaced with VMADDR_CID_LOCAL"
)]
pub const VMADDR_CID_RESERVED: ::c_uint = 1;
pub const VMADDR_CID_LOCAL: ::c_uint = 1;
pub const VMADDR_CID_HOST: ::c_uint = 2;
pub const VMADDR_PORT_ANY: ::c_uint = 0xFFFFFFFF;

// uapi/linux/inotify.h
pub const IN_ACCESS: u32 = 0x0000_0001;
pub const IN_MODIFY: u32 = 0x0000_0002;
pub const IN_ATTRIB: u32 = 0x0000_0004;
pub const IN_CLOSE_WRITE: u32 = 0x0000_0008;
pub const IN_CLOSE_NOWRITE: u32 = 0x0000_0010;
pub const IN_CLOSE: u32 = IN_CLOSE_WRITE | IN_CLOSE_NOWRITE;
pub const IN_OPEN: u32 = 0x0000_0020;
pub const IN_MOVED_FROM: u32 = 0x0000_0040;
pub const IN_MOVED_TO: u32 = 0x0000_0080;
pub const IN_MOVE: u32 = IN_MOVED_FROM | IN_MOVED_TO;
pub const IN_CREATE: u32 = 0x0000_0100;
pub const IN_DELETE: u32 = 0x0000_0200;
pub const IN_DELETE_SELF: u32 = 0x0000_0400;
pub const IN_MOVE_SELF: u32 = 0x0000_0800;
pub const IN_UNMOUNT: u32 = 0x0000_2000;
pub const IN_Q_OVERFLOW: u32 = 0x0000_4000;
pub const IN_IGNORED: u32 = 0x0000_8000;
pub const IN_ONLYDIR: u32 = 0x0100_0000;
pub const IN_DONT_FOLLOW: u32 = 0x0200_0000;
pub const IN_EXCL_UNLINK: u32 = 0x0400_0000;

// linux/keyctl.h
pub const KEY_SPEC_THREAD_KEYRING: i32 = -1;
pub const KEY_SPEC_PROCESS_KEYRING: i32 = -2;
pub const KEY_SPEC_SESSION_KEYRING: i32 = -3;
pub const KEY_SPEC_USER_KEYRING: i32 = -4;
pub const KEY_SPEC_USER_SESSION_KEYRING: i32 = -5;
pub const KEY_SPEC_GROUP_KEYRING: i32 = -6;
pub const KEY_SPEC_REQKEY_AUTH_KEY: i32 = -7;
pub const KEY_SPEC_REQUESTOR_KEYRING: i32 = -8;

pub const KEY_REQKEY_DEFL_NO_CHANGE: i32 = -1;
pub const KEY_REQKEY_DEFL_DEFAULT: i32 = 0;
pub const KEY_REQKEY_DEFL_THREAD_KEYRING: i32 = 1;
pub const KEY_REQKEY_DEFL_PROCESS_KEYRING: i32 = 2;
pub const KEY_REQKEY_DEFL_SESSION_KEYRING: i32 = 3;
pub const KEY_REQKEY_DEFL_USER_KEYRING: i32 = 4;
pub const KEY_REQKEY_DEFL_USER_SESSION_KEYRING: i32 = 5;
pub const KEY_REQKEY_DEFL_GROUP_KEYRING: i32 = 6;
pub const KEY_REQKEY_DEFL_REQUESTOR_KEYRING: i32 = 7;

pub const KEYCTL_GET_KEYRING_ID: u32 = 0;
pub const KEYCTL_JOIN_SESSION_KEYRING: u32 = 1;
pub const KEYCTL_UPDATE: u32 = 2;
pub const KEYCTL_REVOKE: u32 = 3;
pub const KEYCTL_CHOWN: u32 = 4;
pub const KEYCTL_SETPERM: u32 = 5;
pub const KEYCTL_DESCRIBE: u32 = 6;
pub const KEYCTL_CLEAR: u32 = 7;
pub const KEYCTL_LINK: u32 = 8;
pub const KEYCTL_UNLINK: u32 = 9;
pub const KEYCTL_SEARCH: u32 = 10;
pub const KEYCTL_READ: u32 = 11;
pub const KEYCTL_INSTANTIATE: u32 = 12;
pub const KEYCTL_NEGATE: u32 = 13;
pub const KEYCTL_SET_REQKEY_KEYRING: u32 = 14;
pub const KEYCTL_SET_TIMEOUT: u32 = 15;
pub const KEYCTL_ASSUME_AUTHORITY: u32 = 16;
pub const KEYCTL_GET_SECURITY: u32 = 17;
pub const KEYCTL_SESSION_TO_PARENT: u32 = 18;
pub const KEYCTL_REJECT: u32 = 19;
pub const KEYCTL_INSTANTIATE_IOV: u32 = 20;
pub const KEYCTL_INVALIDATE: u32 = 21;
pub const KEYCTL_GET_PERSISTENT: u32 = 22;

pub const IN_MASK_CREATE: u32 = 0x1000_0000;
pub const IN_MASK_ADD: u32 = 0x2000_0000;
pub const IN_ISDIR: u32 = 0x4000_0000;
pub const IN_ONESHOT: u32 = 0x8000_0000;

pub const IN_ALL_EVENTS: u32 = IN_ACCESS
    | IN_MODIFY
    | IN_ATTRIB
    | IN_CLOSE_WRITE
    | IN_CLOSE_NOWRITE
    | IN_OPEN
    | IN_MOVED_FROM
    | IN_MOVED_TO
    | IN_DELETE
    | IN_CREATE
    | IN_DELETE_SELF
    | IN_MOVE_SELF;

pub const IN_CLOEXEC: ::c_int = O_CLOEXEC;
pub const IN_NONBLOCK: ::c_int = O_NONBLOCK;

// uapi/linux/mount.h
pub const OPEN_TREE_CLONE: ::c_uint = 0x01;
pub const OPEN_TREE_CLOEXEC: ::c_uint = O_CLOEXEC as ::c_uint;

// uapi/linux/netfilter/nf_tables.h
pub const NFT_TABLE_MAXNAMELEN: ::c_int = 256;
pub const NFT_CHAIN_MAXNAMELEN: ::c_int = 256;
pub const NFT_SET_MAXNAMELEN: ::c_int = 256;
pub const NFT_OBJ_MAXNAMELEN: ::c_int = 256;
pub const NFT_USERDATA_MAXLEN: ::c_int = 256;

pub const NFT_REG_VERDICT: ::c_int = 0;
pub const NFT_REG_1: ::c_int = 1;
pub const NFT_REG_2: ::c_int = 2;
pub const NFT_REG_3: ::c_int = 3;
pub const NFT_REG_4: ::c_int = 4;
pub const __NFT_REG_MAX: ::c_int = 5;
pub const NFT_REG32_00: ::c_int = 8;
pub const NFT_REG32_01: ::c_int = 9;
pub const NFT_REG32_02: ::c_int = 10;
pub const NFT_REG32_03: ::c_int = 11;
pub const NFT_REG32_04: ::c_int = 12;
pub const NFT_REG32_05: ::c_int = 13;
pub const NFT_REG32_06: ::c_int = 14;
pub const NFT_REG32_07: ::c_int = 15;
pub const NFT_REG32_08: ::c_int = 16;
pub const NFT_REG32_09: ::c_int = 17;
pub const NFT_REG32_10: ::c_int = 18;
pub const NFT_REG32_11: ::c_int = 19;
pub const NFT_REG32_12: ::c_int = 20;
pub const NFT_REG32_13: ::c_int = 21;
pub const NFT_REG32_14: ::c_int = 22;
pub const NFT_REG32_15: ::c_int = 23;

pub const NFT_REG_SIZE: ::c_int = 16;
pub const NFT_REG32_SIZE: ::c_int = 4;

pub const NFT_CONTINUE: ::c_int = -1;
pub const NFT_BREAK: ::c_int = -2;
pub const NFT_JUMP: ::c_int = -3;
pub const NFT_GOTO: ::c_int = -4;
pub const NFT_RETURN: ::c_int = -5;

pub const NFT_MSG_NEWTABLE: ::c_int = 0;
pub const NFT_MSG_GETTABLE: ::c_int = 1;
pub const NFT_MSG_DELTABLE: ::c_int = 2;
pub const NFT_MSG_NEWCHAIN: ::c_int = 3;
pub const NFT_MSG_GETCHAIN: ::c_int = 4;
pub const NFT_MSG_DELCHAIN: ::c_int = 5;
pub const NFT_MSG_NEWRULE: ::c_int = 6;
pub const NFT_MSG_GETRULE: ::c_int = 7;
pub const NFT_MSG_DELRULE: ::c_int = 8;
pub const NFT_MSG_NEWSET: ::c_int = 9;
pub const NFT_MSG_GETSET: ::c_int = 10;
pub const NFT_MSG_DELSET: ::c_int = 11;
pub const NFT_MSG_NEWSETELEM: ::c_int = 12;
pub const NFT_MSG_GETSETELEM: ::c_int = 13;
pub const NFT_MSG_DELSETELEM: ::c_int = 14;
pub const NFT_MSG_NEWGEN: ::c_int = 15;
pub const NFT_MSG_GETGEN: ::c_int = 16;
pub const NFT_MSG_TRACE: ::c_int = 17;
cfg_if! {
    if #[cfg(not(target_arch = "sparc64"))] {
        pub const NFT_MSG_NEWOBJ: ::c_int = 18;
        pub const NFT_MSG_GETOBJ: ::c_int = 19;
        pub const NFT_MSG_DELOBJ: ::c_int = 20;
        pub const NFT_MSG_GETOBJ_RESET: ::c_int = 21;
    }
}
pub const NFT_MSG_MAX: ::c_int = 25;

pub const NFT_SET_ANONYMOUS: ::c_int = 0x1;
pub const NFT_SET_CONSTANT: ::c_int = 0x2;
pub const NFT_SET_INTERVAL: ::c_int = 0x4;
pub const NFT_SET_MAP: ::c_int = 0x8;
pub const NFT_SET_TIMEOUT: ::c_int = 0x10;
pub const NFT_SET_EVAL: ::c_int = 0x20;

pub const NFT_SET_POL_PERFORMANCE: ::c_int = 0;
pub const NFT_SET_POL_MEMORY: ::c_int = 1;

pub const NFT_SET_ELEM_INTERVAL_END: ::c_int = 0x1;

pub const NFT_DATA_VALUE: ::c_uint = 0;
pub const NFT_DATA_VERDICT: ::c_uint = 0xffffff00;

pub const NFT_DATA_RESERVED_MASK: ::c_uint = 0xffffff00;

pub const NFT_DATA_VALUE_MAXLEN: ::c_int = 64;

pub const NFT_BYTEORDER_NTOH: ::c_int = 0;
pub const NFT_BYTEORDER_HTON: ::c_int = 1;

pub const NFT_CMP_EQ: ::c_int = 0;
pub const NFT_CMP_NEQ: ::c_int = 1;
pub const NFT_CMP_LT: ::c_int = 2;
pub const NFT_CMP_LTE: ::c_int = 3;
pub const NFT_CMP_GT: ::c_int = 4;
pub const NFT_CMP_GTE: ::c_int = 5;

pub const NFT_RANGE_EQ: ::c_int = 0;
pub const NFT_RANGE_NEQ: ::c_int = 1;

pub const NFT_LOOKUP_F_INV: ::c_int = 1 << 0;

pub const NFT_DYNSET_OP_ADD: ::c_int = 0;
pub const NFT_DYNSET_OP_UPDATE: ::c_int = 1;

pub const NFT_DYNSET_F_INV: ::c_int = 1 << 0;

pub const NFT_PAYLOAD_LL_HEADER: ::c_int = 0;
pub const NFT_PAYLOAD_NETWORK_HEADER: ::c_int = 1;
pub const NFT_PAYLOAD_TRANSPORT_HEADER: ::c_int = 2;

pub const NFT_PAYLOAD_CSUM_NONE: ::c_int = 0;
pub const NFT_PAYLOAD_CSUM_INET: ::c_int = 1;

pub const NFT_META_LEN: ::c_int = 0;
pub const NFT_META_PROTOCOL: ::c_int = 1;
pub const NFT_META_PRIORITY: ::c_int = 2;
pub const NFT_META_MARK: ::c_int = 3;
pub const NFT_META_IIF: ::c_int = 4;
pub const NFT_META_OIF: ::c_int = 5;
pub const NFT_META_IIFNAME: ::c_int = 6;
pub const NFT_META_OIFNAME: ::c_int = 7;
pub const NFT_META_IIFTYPE: ::c_int = 8;
pub const NFT_META_OIFTYPE: ::c_int = 9;
pub const NFT_META_SKUID: ::c_int = 10;
pub const NFT_META_SKGID: ::c_int = 11;
pub const NFT_META_NFTRACE: ::c_int = 12;
pub const NFT_META_RTCLASSID: ::c_int = 13;
pub const NFT_META_SECMARK: ::c_int = 14;
pub const NFT_META_NFPROTO: ::c_int = 15;
pub const NFT_META_L4PROTO: ::c_int = 16;
pub const NFT_META_BRI_IIFNAME: ::c_int = 17;
pub const NFT_META_BRI_OIFNAME: ::c_int = 18;
pub const NFT_META_PKTTYPE: ::c_int = 19;
pub const NFT_META_CPU: ::c_int = 20;
pub const NFT_META_IIFGROUP: ::c_int = 21;
pub const NFT_META_OIFGROUP: ::c_int = 22;
pub const NFT_META_CGROUP: ::c_int = 23;
pub const NFT_META_PRANDOM: ::c_int = 24;

pub const NFT_CT_STATE: ::c_int = 0;
pub const NFT_CT_DIRECTION: ::c_int = 1;
pub const NFT_CT_STATUS: ::c_int = 2;
pub const NFT_CT_MARK: ::c_int = 3;
pub const NFT_CT_SECMARK: ::c_int = 4;
pub const NFT_CT_EXPIRATION: ::c_int = 5;
pub const NFT_CT_HELPER: ::c_int = 6;
pub const NFT_CT_L3PROTOCOL: ::c_int = 7;
pub const NFT_CT_SRC: ::c_int = 8;
pub const NFT_CT_DST: ::c_int = 9;
pub const NFT_CT_PROTOCOL: ::c_int = 10;
pub const NFT_CT_PROTO_SRC: ::c_int = 11;
pub const NFT_CT_PROTO_DST: ::c_int = 12;
pub const NFT_CT_LABELS: ::c_int = 13;
pub const NFT_CT_PKTS: ::c_int = 14;
pub const NFT_CT_BYTES: ::c_int = 15;

pub const NFT_LIMIT_PKTS: ::c_int = 0;
pub const NFT_LIMIT_PKT_BYTES: ::c_int = 1;

pub const NFT_LIMIT_F_INV: ::c_int = 1 << 0;

pub const NFT_QUEUE_FLAG_BYPASS: ::c_int = 0x01;
pub const NFT_QUEUE_FLAG_CPU_FANOUT: ::c_int = 0x02;
pub const NFT_QUEUE_FLAG_MASK: ::c_int = 0x03;

pub const NFT_QUOTA_F_INV: ::c_int = 1 << 0;

pub const NFT_REJECT_ICMP_UNREACH: ::c_int = 0;
pub const NFT_REJECT_TCP_RST: ::c_int = 1;
pub const NFT_REJECT_ICMPX_UNREACH: ::c_int = 2;

pub const NFT_REJECT_ICMPX_NO_ROUTE: ::c_int = 0;
pub const NFT_REJECT_ICMPX_PORT_UNREACH: ::c_int = 1;
pub const NFT_REJECT_ICMPX_HOST_UNREACH: ::c_int = 2;
pub const NFT_REJECT_ICMPX_ADMIN_PROHIBITED: ::c_int = 3;

pub const NFT_NAT_SNAT: ::c_int = 0;
pub const NFT_NAT_DNAT: ::c_int = 1;

pub const NFT_TRACETYPE_UNSPEC: ::c_int = 0;
pub const NFT_TRACETYPE_POLICY: ::c_int = 1;
pub const NFT_TRACETYPE_RETURN: ::c_int = 2;
pub const NFT_TRACETYPE_RULE: ::c_int = 3;

pub const NFT_NG_INCREMENTAL: ::c_int = 0;
pub const NFT_NG_RANDOM: ::c_int = 1;

// linux/input.h
pub const FF_MAX: ::__u16 = 0x7f;
pub const FF_CNT: usize = FF_MAX as usize + 1;

// linux/input-event-codes.h
pub const INPUT_PROP_MAX: ::__u16 = 0x1f;
pub const INPUT_PROP_CNT: usize = INPUT_PROP_MAX as usize + 1;
pub const EV_MAX: ::__u16 = 0x1f;
pub const EV_CNT: usize = EV_MAX as usize + 1;
pub const SYN_MAX: ::__u16 = 0xf;
pub const SYN_CNT: usize = SYN_MAX as usize + 1;
pub const KEY_MAX: ::__u16 = 0x2ff;
pub const KEY_CNT: usize = KEY_MAX as usize + 1;
pub const REL_MAX: ::__u16 = 0x0f;
pub const REL_CNT: usize = REL_MAX as usize + 1;
pub const ABS_MAX: ::__u16 = 0x3f;
pub const ABS_CNT: usize = ABS_MAX as usize + 1;
pub const SW_MAX: ::__u16 = 0x10;
pub const SW_CNT: usize = SW_MAX as usize + 1;
pub const MSC_MAX: ::__u16 = 0x07;
pub const MSC_CNT: usize = MSC_MAX as usize + 1;
pub const LED_MAX: ::__u16 = 0x0f;
pub const LED_CNT: usize = LED_MAX as usize + 1;
pub const REP_MAX: ::__u16 = 0x01;
pub const REP_CNT: usize = REP_MAX as usize + 1;
pub const SND_MAX: ::__u16 = 0x07;
pub const SND_CNT: usize = SND_MAX as usize + 1;

// linux/uinput.h
pub const UINPUT_VERSION: ::c_uint = 5;
pub const UINPUT_MAX_NAME_SIZE: usize = 80;

// uapi/linux/fanotify.h
pub const FAN_ACCESS: u64 = 0x0000_0001;
pub const FAN_MODIFY: u64 = 0x0000_0002;
pub const FAN_CLOSE_WRITE: u64 = 0x0000_0008;
pub const FAN_CLOSE_NOWRITE: u64 = 0x0000_0010;
pub const FAN_OPEN: u64 = 0x0000_0020;

pub const FAN_Q_OVERFLOW: u64 = 0x0000_4000;

pub const FAN_OPEN_PERM: u64 = 0x0001_0000;
pub const FAN_ACCESS_PERM: u64 = 0x0002_0000;

pub const FAN_ONDIR: u64 = 0x4000_0000;

pub const FAN_EVENT_ON_CHILD: u64 = 0x0800_0000;

pub const FAN_CLOSE: u64 = FAN_CLOSE_WRITE | FAN_CLOSE_NOWRITE;

pub const FAN_CLOEXEC: ::c_uint = 0x0000_0001;
pub const FAN_NONBLOCK: ::c_uint = 0x0000_0002;

pub const FAN_CLASS_NOTIF: ::c_uint = 0x0000_0000;
pub const FAN_CLASS_CONTENT: ::c_uint = 0x0000_0004;
pub const FAN_CLASS_PRE_CONTENT: ::c_uint = 0x0000_0008;

pub const FAN_UNLIMITED_QUEUE: ::c_uint = 0x0000_0010;
pub const FAN_UNLIMITED_MARKS: ::c_uint = 0x0000_0020;

pub const FAN_MARK_ADD: ::c_uint = 0x0000_0001;
pub const FAN_MARK_REMOVE: ::c_uint = 0x0000_0002;
pub const FAN_MARK_DONT_FOLLOW: ::c_uint = 0x0000_0004;
pub const FAN_MARK_ONLYDIR: ::c_uint = 0x0000_0008;
pub const FAN_MARK_IGNORED_MASK: ::c_uint = 0x0000_0020;
pub const FAN_MARK_IGNORED_SURV_MODIFY: ::c_uint = 0x0000_0040;
pub const FAN_MARK_FLUSH: ::c_uint = 0x0000_0080;

pub const FANOTIFY_METADATA_VERSION: u8 = 3;

pub const FAN_ALLOW: u32 = 0x01;
pub const FAN_DENY: u32 = 0x02;

pub const FAN_NOFD: ::c_int = -1;

pub const FUTEX_WAIT: ::c_int = 0;
pub const FUTEX_WAKE: ::c_int = 1;
pub const FUTEX_FD: ::c_int = 2;
pub const FUTEX_REQUEUE: ::c_int = 3;
pub const FUTEX_CMP_REQUEUE: ::c_int = 4;
pub const FUTEX_WAKE_OP: ::c_int = 5;
pub const FUTEX_LOCK_PI: ::c_int = 6;
pub const FUTEX_UNLOCK_PI: ::c_int = 7;
pub const FUTEX_TRYLOCK_PI: ::c_int = 8;
pub const FUTEX_WAIT_BITSET: ::c_int = 9;
pub const FUTEX_WAKE_BITSET: ::c_int = 10;
pub const FUTEX_WAIT_REQUEUE_PI: ::c_int = 11;
pub const FUTEX_CMP_REQUEUE_PI: ::c_int = 12;
pub const FUTEX_LOCK_PI2: ::c_int = 13;

pub const FUTEX_PRIVATE_FLAG: ::c_int = 128;
pub const FUTEX_CLOCK_REALTIME: ::c_int = 256;
pub const FUTEX_CMD_MASK: ::c_int = !(FUTEX_PRIVATE_FLAG | FUTEX_CLOCK_REALTIME);

pub const FUTEX_BITSET_MATCH_ANY: ::c_int = 0xffffffff;

pub const FUTEX_OP_SET: ::c_int = 0;
pub const FUTEX_OP_ADD: ::c_int = 1;
pub const FUTEX_OP_OR: ::c_int = 2;
pub const FUTEX_OP_ANDN: ::c_int = 3;
pub const FUTEX_OP_XOR: ::c_int = 4;

pub const FUTEX_OP_OPARG_SHIFT: ::c_int = 8;

pub const FUTEX_OP_CMP_EQ: ::c_int = 0;
pub const FUTEX_OP_CMP_NE: ::c_int = 1;
pub const FUTEX_OP_CMP_LT: ::c_int = 2;
pub const FUTEX_OP_CMP_LE: ::c_int = 3;
pub const FUTEX_OP_CMP_GT: ::c_int = 4;
pub const FUTEX_OP_CMP_GE: ::c_int = 5;

pub fn FUTEX_OP(op: ::c_int, oparg: ::c_int, cmp: ::c_int, cmparg: ::c_int) -> ::c_int {
    ((op & 0xf) << 28) | ((cmp & 0xf) << 24) | ((oparg & 0xfff) << 12) | (cmparg & 0xfff)
}

// linux/kexec.h
pub const KEXEC_ON_CRASH: ::c_int = 0x00000001;
pub const KEXEC_PRESERVE_CONTEXT: ::c_int = 0x00000002;
pub const KEXEC_ARCH_MASK: ::c_int = 0xffff0000;
pub const KEXEC_FILE_UNLOAD: ::c_int = 0x00000001;
pub const KEXEC_FILE_ON_CRASH: ::c_int = 0x00000002;
pub const KEXEC_FILE_NO_INITRAMFS: ::c_int = 0x00000004;

// linux/reboot.h
pub const LINUX_REBOOT_MAGIC1: ::c_int = 0xfee1dead;
pub const LINUX_REBOOT_MAGIC2: ::c_int = 672274793;
pub const LINUX_REBOOT_MAGIC2A: ::c_int = 85072278;
pub const LINUX_REBOOT_MAGIC2B: ::c_int = 369367448;
pub const LINUX_REBOOT_MAGIC2C: ::c_int = 537993216;

pub const LINUX_REBOOT_CMD_RESTART: ::c_int = 0x01234567;
pub const LINUX_REBOOT_CMD_HALT: ::c_int = 0xCDEF0123;
pub const LINUX_REBOOT_CMD_CAD_ON: ::c_int = 0x89ABCDEF;
pub const LINUX_REBOOT_CMD_CAD_OFF: ::c_int = 0x00000000;
pub const LINUX_REBOOT_CMD_POWER_OFF: ::c_int = 0x4321FEDC;
pub const LINUX_REBOOT_CMD_RESTART2: ::c_int = 0xA1B2C3D4;
pub const LINUX_REBOOT_CMD_SW_SUSPEND: ::c_int = 0xD000FCE2;
pub const LINUX_REBOOT_CMD_KEXEC: ::c_int = 0x45584543;

pub const REG_EXTENDED: ::c_int = 1;
pub const REG_ICASE: ::c_int = 2;
pub const REG_NEWLINE: ::c_int = 4;
pub const REG_NOSUB: ::c_int = 8;

pub const REG_NOTBOL: ::c_int = 1;
pub const REG_NOTEOL: ::c_int = 2;

pub const REG_ENOSYS: ::c_int = -1;
pub const REG_NOMATCH: ::c_int = 1;
pub const REG_BADPAT: ::c_int = 2;
pub const REG_ECOLLATE: ::c_int = 3;
pub const REG_ECTYPE: ::c_int = 4;
pub const REG_EESCAPE: ::c_int = 5;
pub const REG_ESUBREG: ::c_int = 6;
pub const REG_EBRACK: ::c_int = 7;
pub const REG_EPAREN: ::c_int = 8;
pub const REG_EBRACE: ::c_int = 9;
pub const REG_BADBR: ::c_int = 10;
pub const REG_ERANGE: ::c_int = 11;
pub const REG_ESPACE: ::c_int = 12;
pub const REG_BADRPT: ::c_int = 13;

// linux/errqueue.h
pub const SO_EE_ORIGIN_NONE: u8 = 0;
pub const SO_EE_ORIGIN_LOCAL: u8 = 1;
pub const SO_EE_ORIGIN_ICMP: u8 = 2;
pub const SO_EE_ORIGIN_ICMP6: u8 = 3;
pub const SO_EE_ORIGIN_TXSTATUS: u8 = 4;
pub const SO_EE_ORIGIN_TIMESTAMPING: u8 = SO_EE_ORIGIN_TXSTATUS;

// errno.h
pub const EPERM: ::c_int = 1;
pub const ENOENT: ::c_int = 2;
pub const ESRCH: ::c_int = 3;
pub const EINTR: ::c_int = 4;
pub const EIO: ::c_int = 5;
pub const ENXIO: ::c_int = 6;
pub const E2BIG: ::c_int = 7;
pub const ENOEXEC: ::c_int = 8;
pub const EBADF: ::c_int = 9;
pub const ECHILD: ::c_int = 10;
pub const EAGAIN: ::c_int = 11;
pub const ENOMEM: ::c_int = 12;
pub const EACCES: ::c_int = 13;
pub const EFAULT: ::c_int = 14;
pub const ENOTBLK: ::c_int = 15;
pub const EBUSY: ::c_int = 16;
pub const EEXIST: ::c_int = 17;
pub const EXDEV: ::c_int = 18;
pub const ENODEV: ::c_int = 19;
pub const ENOTDIR: ::c_int = 20;
pub const EISDIR: ::c_int = 21;
pub const EINVAL: ::c_int = 22;
pub const ENFILE: ::c_int = 23;
pub const EMFILE: ::c_int = 24;
pub const ENOTTY: ::c_int = 25;
pub const ETXTBSY: ::c_int = 26;
pub const EFBIG: ::c_int = 27;
pub const ENOSPC: ::c_int = 28;
pub const ESPIPE: ::c_int = 29;
pub const EROFS: ::c_int = 30;
pub const EMLINK: ::c_int = 31;
pub const EPIPE: ::c_int = 32;
pub const EDOM: ::c_int = 33;
pub const ERANGE: ::c_int = 34;
pub const EWOULDBLOCK: ::c_int = EAGAIN;

// linux/can.h
pub const CAN_EFF_FLAG: canid_t = 0x80000000;
pub const CAN_RTR_FLAG: canid_t = 0x40000000;
pub const CAN_ERR_FLAG: canid_t = 0x20000000;
pub const CAN_SFF_MASK: canid_t = 0x000007FF;
pub const CAN_EFF_MASK: canid_t = 0x1FFFFFFF;
pub const CAN_ERR_MASK: canid_t = 0x1FFFFFFF;
pub const CANXL_PRIO_MASK: ::canid_t = CAN_SFF_MASK;

pub const CAN_SFF_ID_BITS: ::c_int = 11;
pub const CAN_EFF_ID_BITS: ::c_int = 29;
pub const CANXL_PRIO_BITS: ::c_int = CAN_SFF_ID_BITS;

pub const CAN_MAX_DLC: ::c_int = 8;
pub const CAN_MAX_DLEN: usize = 8;
pub const CANFD_MAX_DLC: ::c_int = 15;
pub const CANFD_MAX_DLEN: usize = 64;

pub const CANFD_BRS: ::c_int = 0x01;
pub const CANFD_ESI: ::c_int = 0x02;

pub const CANXL_MIN_DLC: ::c_int = 0;
pub const CANXL_MAX_DLC: ::c_int = 2047;
pub const CANXL_MAX_DLC_MASK: ::c_int = 0x07FF;
pub const CANXL_MIN_DLEN: usize = 1;
pub const CANXL_MAX_DLEN: usize = 2048;

pub const CANXL_XLF: ::c_int = 0x80;
pub const CANXL_SEC: ::c_int = 0x01;

cfg_if! {
    if #[cfg(libc_align)] {
        pub const CAN_MTU: usize = ::mem::size_of::<can_frame>();
        pub const CANFD_MTU: usize = ::mem::size_of::<canfd_frame>();
        pub const CANXL_MTU: usize = ::mem::size_of::<canxl_frame>();
        // FIXME: use `core::mem::offset_of!` once that is available
        // https://github.com/rust-lang/rfcs/pull/3308
        // pub const CANXL_HDR_SIZE: usize = core::mem::offset_of!(canxl_frame, data);
        pub const CANXL_HDR_SIZE: usize = 12;
        pub const CANXL_MIN_MTU: usize = CANXL_HDR_SIZE + 64;
        pub const CANXL_MAX_MTU: usize = CANXL_MTU;
    }
}

pub const CAN_RAW: ::c_int = 1;
pub const CAN_BCM: ::c_int = 2;
pub const CAN_TP16: ::c_int = 3;
pub const CAN_TP20: ::c_int = 4;
pub const CAN_MCNET: ::c_int = 5;
pub const CAN_ISOTP: ::c_int = 6;
pub const CAN_J1939: ::c_int = 7;
pub const CAN_NPROTO: ::c_int = 8;

pub const SOL_CAN_BASE: ::c_int = 100;

pub const CAN_INV_FILTER: canid_t = 0x20000000;
pub const CAN_RAW_FILTER_MAX: ::c_int = 512;

// linux/can/raw.h
pub const SOL_CAN_RAW: ::c_int = SOL_CAN_BASE + CAN_RAW;
pub const CAN_RAW_FILTER: ::c_int = 1;
pub const CAN_RAW_ERR_FILTER: ::c_int = 2;
pub const CAN_RAW_LOOPBACK: ::c_int = 3;
pub const CAN_RAW_RECV_OWN_MSGS: ::c_int = 4;
pub const CAN_RAW_FD_FRAMES: ::c_int = 5;
pub const CAN_RAW_JOIN_FILTERS: ::c_int = 6;
pub const CAN_RAW_XL_FRAMES: ::c_int = 7;

// linux/can/j1939.h
pub const SOL_CAN_J1939: ::c_int = SOL_CAN_BASE + CAN_J1939;

pub const J1939_MAX_UNICAST_ADDR: ::c_uchar = 0xfd;
pub const J1939_IDLE_ADDR: ::c_uchar = 0xfe;
pub const J1939_NO_ADDR: ::c_uchar = 0xff;
pub const J1939_NO_NAME: ::c_ulong = 0;
pub const J1939_PGN_REQUEST: ::c_uint = 0x0ea00;
pub const J1939_PGN_ADDRESS_CLAIMED: ::c_uint = 0x0ee00;
pub const J1939_PGN_ADDRESS_COMMANDED: ::c_uint = 0x0fed8;
pub const J1939_PGN_PDU1_MAX: ::c_uint = 0x3ff00;
pub const J1939_PGN_MAX: ::c_uint = 0x3ffff;
pub const J1939_NO_PGN: ::c_uint = 0x40000;

pub const SO_J1939_FILTER: ::c_int = 1;
pub const SO_J1939_PROMISC: ::c_int = 2;
pub const SO_J1939_SEND_PRIO: ::c_int = 3;
pub const SO_J1939_ERRQUEUE: ::c_int = 4;

pub const SCM_J1939_DEST_ADDR: ::c_int = 1;
pub const SCM_J1939_DEST_NAME: ::c_int = 2;
pub const SCM_J1939_PRIO: ::c_int = 3;
pub const SCM_J1939_ERRQUEUE: ::c_int = 4;

pub const J1939_NLA_PAD: ::c_int = 0;
pub const J1939_NLA_BYTES_ACKED: ::c_int = 1;
pub const J1939_NLA_TOTAL_SIZE: ::c_int = 2;
pub const J1939_NLA_PGN: ::c_int = 3;
pub const J1939_NLA_SRC_NAME: ::c_int = 4;
pub const J1939_NLA_DEST_NAME: ::c_int = 5;
pub const J1939_NLA_SRC_ADDR: ::c_int = 6;
pub const J1939_NLA_DEST_ADDR: ::c_int = 7;

pub const J1939_EE_INFO_NONE: ::c_int = 0;
pub const J1939_EE_INFO_TX_ABORT: ::c_int = 1;
pub const J1939_EE_INFO_RX_RTS: ::c_int = 2;
pub const J1939_EE_INFO_RX_DPO: ::c_int = 3;
pub const J1939_EE_INFO_RX_ABORT: ::c_int = 4;

pub const J1939_FILTER_MAX: ::c_int = 512;

// linux/sctp.h
pub const SCTP_FUTURE_ASSOC: ::c_int = 0;
pub const SCTP_CURRENT_ASSOC: ::c_int = 1;
pub const SCTP_ALL_ASSOC: ::c_int = 2;
pub const SCTP_RTOINFO: ::c_int = 0;
pub const SCTP_ASSOCINFO: ::c_int = 1;
pub const SCTP_INITMSG: ::c_int = 2;
pub const SCTP_NODELAY: ::c_int = 3;
pub const SCTP_AUTOCLOSE: ::c_int = 4;
pub const SCTP_SET_PEER_PRIMARY_ADDR: ::c_int = 5;
pub const SCTP_PRIMARY_ADDR: ::c_int = 6;
pub const SCTP_ADAPTATION_LAYER: ::c_int = 7;
pub const SCTP_DISABLE_FRAGMENTS: ::c_int = 8;
pub const SCTP_PEER_ADDR_PARAMS: ::c_int = 9;
pub const SCTP_DEFAULT_SEND_PARAM: ::c_int = 10;
pub const SCTP_EVENTS: ::c_int = 11;
pub const SCTP_I_WANT_MAPPED_V4_ADDR: ::c_int = 12;
pub const SCTP_MAXSEG: ::c_int = 13;
pub const SCTP_STATUS: ::c_int = 14;
pub const SCTP_GET_PEER_ADDR_INFO: ::c_int = 15;
pub const SCTP_DELAYED_ACK_TIME: ::c_int = 16;
pub const SCTP_DELAYED_ACK: ::c_int = SCTP_DELAYED_ACK_TIME;
pub const SCTP_DELAYED_SACK: ::c_int = SCTP_DELAYED_ACK_TIME;
pub const SCTP_CONTEXT: ::c_int = 17;
pub const SCTP_FRAGMENT_INTERLEAVE: ::c_int = 18;
pub const SCTP_PARTIAL_DELIVERY_POINT: ::c_int = 19;
pub const SCTP_MAX_BURST: ::c_int = 20;
pub const SCTP_AUTH_CHUNK: ::c_int = 21;
pub const SCTP_HMAC_IDENT: ::c_int = 22;
pub const SCTP_AUTH_KEY: ::c_int = 23;
pub const SCTP_AUTH_ACTIVE_KEY: ::c_int = 24;
pub const SCTP_AUTH_DELETE_KEY: ::c_int = 25;
pub const SCTP_PEER_AUTH_CHUNKS: ::c_int = 26;
pub const SCTP_LOCAL_AUTH_CHUNKS: ::c_int = 27;
pub const SCTP_GET_ASSOC_NUMBER: ::c_int = 28;
pub const SCTP_GET_ASSOC_ID_LIST: ::c_int = 29;
pub const SCTP_AUTO_ASCONF: ::c_int = 30;
pub const SCTP_PEER_ADDR_THLDS: ::c_int = 31;
pub const SCTP_RECVRCVINFO: ::c_int = 32;
pub const SCTP_RECVNXTINFO: ::c_int = 33;
pub const SCTP_DEFAULT_SNDINFO: ::c_int = 34;
pub const SCTP_AUTH_DEACTIVATE_KEY: ::c_int = 35;
pub const SCTP_REUSE_PORT: ::c_int = 36;
pub const SCTP_PEER_ADDR_THLDS_V2: ::c_int = 37;
pub const SCTP_PR_SCTP_NONE: ::c_int = 0x0000;
pub const SCTP_PR_SCTP_TTL: ::c_int = 0x0010;
pub const SCTP_PR_SCTP_RTX: ::c_int = 0x0020;
pub const SCTP_PR_SCTP_PRIO: ::c_int = 0x0030;
pub const SCTP_PR_SCTP_MAX: ::c_int = SCTP_PR_SCTP_PRIO;
pub const SCTP_PR_SCTP_MASK: ::c_int = 0x0030;
pub const SCTP_ENABLE_RESET_STREAM_REQ: ::c_int = 0x01;
pub const SCTP_ENABLE_RESET_ASSOC_REQ: ::c_int = 0x02;
pub const SCTP_ENABLE_CHANGE_ASSOC_REQ: ::c_int = 0x04;
pub const SCTP_ENABLE_STRRESET_MASK: ::c_int = 0x07;
pub const SCTP_STREAM_RESET_INCOMING: ::c_int = 0x01;
pub const SCTP_STREAM_RESET_OUTGOING: ::c_int = 0x02;

pub const SCTP_INIT: ::c_int = 0;
pub const SCTP_SNDRCV: ::c_int = 1;
pub const SCTP_SNDINFO: ::c_int = 2;
pub const SCTP_RCVINFO: ::c_int = 3;
pub const SCTP_NXTINFO: ::c_int = 4;
pub const SCTP_PRINFO: ::c_int = 5;
pub const SCTP_AUTHINFO: ::c_int = 6;
pub const SCTP_DSTADDRV4: ::c_int = 7;
pub const SCTP_DSTADDRV6: ::c_int = 8;

pub const SCTP_UNORDERED: ::c_int = 1 << 0;
pub const SCTP_ADDR_OVER: ::c_int = 1 << 1;
pub const SCTP_ABORT: ::c_int = 1 << 2;
pub const SCTP_SACK_IMMEDIATELY: ::c_int = 1 << 3;
pub const SCTP_SENDALL: ::c_int = 1 << 6;
pub const SCTP_PR_SCTP_ALL: ::c_int = 1 << 7;
pub const SCTP_NOTIFICATION: ::c_int = MSG_NOTIFICATION;
pub const SCTP_EOF: ::c_int = ::MSG_FIN;

/* DCCP socket options */
pub const DCCP_SOCKOPT_PACKET_SIZE: ::c_int = 1;
pub const DCCP_SOCKOPT_SERVICE: ::c_int = 2;
pub const DCCP_SOCKOPT_CHANGE_L: ::c_int = 3;
pub const DCCP_SOCKOPT_CHANGE_R: ::c_int = 4;
pub const DCCP_SOCKOPT_GET_CUR_MPS: ::c_int = 5;
pub const DCCP_SOCKOPT_SERVER_TIMEWAIT: ::c_int = 6;
pub const DCCP_SOCKOPT_SEND_CSCOV: ::c_int = 10;
pub const DCCP_SOCKOPT_RECV_CSCOV: ::c_int = 11;
pub const DCCP_SOCKOPT_AVAILABLE_CCIDS: ::c_int = 12;
pub const DCCP_SOCKOPT_CCID: ::c_int = 13;
pub const DCCP_SOCKOPT_TX_CCID: ::c_int = 14;
pub const DCCP_SOCKOPT_RX_CCID: ::c_int = 15;
pub const DCCP_SOCKOPT_QPOLICY_ID: ::c_int = 16;
pub const DCCP_SOCKOPT_QPOLICY_TXQLEN: ::c_int = 17;
pub const DCCP_SOCKOPT_CCID_RX_INFO: ::c_int = 128;
pub const DCCP_SOCKOPT_CCID_TX_INFO: ::c_int = 192;

/// maximum number of services provided on the same listening port
pub const DCCP_SERVICE_LIST_MAX_LEN: ::c_int = 32;

f! {
    pub fn NLA_ALIGN(len: ::c_int) -> ::c_int {
        return ((len) + NLA_ALIGNTO - 1) & !(NLA_ALIGNTO - 1)
    }

    pub fn CMSG_NXTHDR(mhdr: *const msghdr,
                       cmsg: *const cmsghdr) -> *mut cmsghdr {
        if ((*cmsg).cmsg_len as usize) < ::mem::size_of::<cmsghdr>() {
            return 0 as *mut cmsghdr;
        };
        let next = (cmsg as usize +
                    super::CMSG_ALIGN((*cmsg).cmsg_len as usize))
            as *mut cmsghdr;
        let max = (*mhdr).msg_control as usize
            + (*mhdr).msg_controllen as usize;
        if (next.offset(1)) as usize > max ||
            next as usize + super::CMSG_ALIGN((*next).cmsg_len as usize) > max
        {
            0 as *mut cmsghdr
        } else {
            next as *mut cmsghdr
        }
    }

    pub fn CPU_ALLOC_SIZE(count: ::c_int) -> ::size_t {
        let _dummy: cpu_set_t = ::mem::zeroed();
        let size_in_bits = 8 * ::mem::size_of_val(&_dummy.bits[0]);
        ((count as ::size_t + size_in_bits - 1) / 8) as ::size_t
    }

    pub fn CPU_ZERO(cpuset: &mut cpu_set_t) -> () {
        for slot in cpuset.bits.iter_mut() {
            *slot = 0;
        }
    }

    pub fn CPU_SET(cpu: usize, cpuset: &mut cpu_set_t) -> () {
        let size_in_bits
            = 8 * ::mem::size_of_val(&cpuset.bits[0]); // 32, 64 etc
        let (idx, offset) = (cpu / size_in_bits, cpu % size_in_bits);
        cpuset.bits[idx] |= 1 << offset;
        ()
    }

    pub fn CPU_CLR(cpu: usize, cpuset: &mut cpu_set_t) -> () {
        let size_in_bits
            = 8 * ::mem::size_of_val(&cpuset.bits[0]); // 32, 64 etc
        let (idx, offset) = (cpu / size_in_bits, cpu % size_in_bits);
        cpuset.bits[idx] &= !(1 << offset);
        ()
    }

    pub fn CPU_ISSET(cpu: usize, cpuset: &cpu_set_t) -> bool {
        let size_in_bits = 8 * ::mem::size_of_val(&cpuset.bits[0]);
        let (idx, offset) = (cpu / size_in_bits, cpu % size_in_bits);
        0 != (cpuset.bits[idx] & (1 << offset))
    }

    pub fn CPU_COUNT_S(size: usize, cpuset: &cpu_set_t) -> ::c_int {
        let mut s: u32 = 0;
        let size_of_mask = ::mem::size_of_val(&cpuset.bits[0]);
        for i in cpuset.bits[..(size / size_of_mask)].iter() {
            s += i.count_ones();
        };
        s as ::c_int
    }

    pub fn CPU_COUNT(cpuset: &cpu_set_t) -> ::c_int {
        CPU_COUNT_S(::mem::size_of::<cpu_set_t>(), cpuset)
    }

    pub fn CPU_EQUAL(set1: &cpu_set_t, set2: &cpu_set_t) -> bool {
        set1.bits == set2.bits
    }

    pub fn SCTP_PR_INDEX(policy: ::c_int) -> ::c_int {
        policy >> 4 - 1
    }

    pub fn SCTP_PR_POLICY(policy: ::c_int) -> ::c_int {
        policy & SCTP_PR_SCTP_MASK
    }

    pub fn SCTP_PR_SET_POLICY(flags: &mut ::c_int, policy: ::c_int) -> () {
        *flags &= !SCTP_PR_SCTP_MASK;
        *flags |= policy;
        ()
    }

    pub fn major(dev: ::dev_t) -> ::c_uint {
        let mut major = 0;
        major |= (dev & 0x00000000000fff00) >> 8;
        major |= (dev & 0xfffff00000000000) >> 32;
        major as ::c_uint
    }

    pub fn minor(dev: ::dev_t) -> ::c_uint {
        let mut minor = 0;
        minor |= (dev & 0x00000000000000ff) >> 0;
        minor |= (dev & 0x00000ffffff00000) >> 12;
        minor as ::c_uint
    }

    pub fn IPTOS_TOS(tos: u8) -> u8 {
        tos & IPTOS_TOS_MASK
    }

    pub fn IPTOS_PREC(tos: u8) -> u8 {
        tos & IPTOS_PREC_MASK
    }

    pub fn RT_TOS(tos: u8) -> u8 {
        tos & ::IPTOS_TOS_MASK
    }

    pub fn RT_ADDRCLASS(flags: u32) -> u32 {
        flags >> 23
    }

    pub fn RT_LOCALADDR(flags: u32) -> bool {
        (flags & RTF_ADDRCLASSMASK) == (RTF_LOCAL | RTF_INTERFACE)
    }

    pub fn SO_EE_OFFENDER(ee: *const ::sock_extended_err) -> *mut ::sockaddr {
        ee.offset(1) as *mut ::sockaddr
    }

    pub fn BPF_RVAL(code: ::__u32) -> ::__u32 {
        code & 0x18
    }

    pub fn BPF_MISCOP(code: ::__u32) -> ::__u32 {
        code & 0xf8
    }

    pub fn BPF_STMT(code: ::__u16, k: ::__u32) -> sock_filter {
        sock_filter{code: code, jt: 0, jf: 0, k: k}
    }

    pub fn BPF_JUMP(code: ::__u16, k: ::__u32, jt: ::__u8, jf: ::__u8) -> sock_filter {
        sock_filter{code: code, jt: jt, jf: jf, k: k}
    }
}

safe_f! {
    pub {const} fn makedev(major: ::c_uint, minor: ::c_uint) -> ::dev_t {
        let major = major as ::dev_t;
        let minor = minor as ::dev_t;
        let mut dev = 0;
        dev |= (major & 0x00000fff) << 8;
        dev |= (major & 0xfffff000) << 32;
        dev |= (minor & 0x000000ff) << 0;
        dev |= (minor & 0xffffff00) << 12;
        dev
    }

    pub {const} fn SCTP_PR_TTL_ENABLED(policy: ::c_int) -> bool {
        policy == SCTP_PR_SCTP_TTL
    }

    pub {const} fn SCTP_PR_RTX_ENABLED(policy: ::c_int) -> bool {
        policy == SCTP_PR_SCTP_RTX
    }

    pub {const} fn SCTP_PR_PRIO_ENABLED(policy: ::c_int) -> bool {
        policy == SCTP_PR_SCTP_PRIO
    }
}

cfg_if! {
    if #[cfg(all(not(target_env = "uclibc"), not(target_env = "ohos")))] {
        extern "C" {
            pub fn aio_read(aiocbp: *mut aiocb) -> ::c_int;
            pub fn aio_write(aiocbp: *mut aiocb) -> ::c_int;
            pub fn aio_fsync(op: ::c_int, aiocbp: *mut aiocb) -> ::c_int;
            pub fn aio_error(aiocbp: *const aiocb) -> ::c_int;
            pub fn aio_return(aiocbp: *mut aiocb) -> ::ssize_t;
            pub fn aio_suspend(
                aiocb_list: *const *const aiocb,
                nitems: ::c_int,
                timeout: *const ::timespec,
            ) -> ::c_int;
            pub fn aio_cancel(fd: ::c_int, aiocbp: *mut aiocb) -> ::c_int;
            pub fn lio_listio(
                mode: ::c_int,
                aiocb_list: *const *mut aiocb,
                nitems: ::c_int,
                sevp: *mut ::sigevent,
            ) -> ::c_int;
        }
    }
}

cfg_if! {
    if #[cfg(not(target_env = "uclibc"))] {
        extern "C" {
            pub fn pwritev(
                fd: ::c_int,
                iov: *const ::iovec,
                iovcnt: ::c_int,
                offset: ::off_t,
            ) -> ::ssize_t;
            pub fn preadv(
                fd: ::c_int,
                iov: *const ::iovec,
                iovcnt: ::c_int,
                offset: ::off_t,
            ) -> ::ssize_t;
            pub fn getnameinfo(
                sa: *const ::sockaddr,
                salen: ::socklen_t,
                host: *mut ::c_char,
                hostlen: ::socklen_t,
                serv: *mut ::c_char,
                sevlen: ::socklen_t,
                flags: ::c_int,
            ) -> ::c_int;
            pub fn getloadavg(
                loadavg: *mut ::c_double,
                nelem: ::c_int
            ) -> ::c_int;
            pub fn process_vm_readv(
                pid: ::pid_t,
                local_iov: *const ::iovec,
                liovcnt: ::c_ulong,
                remote_iov: *const ::iovec,
                riovcnt: ::c_ulong,
                flags: ::c_ulong,
            ) -> isize;
            pub fn process_vm_writev(
                pid: ::pid_t,
                local_iov: *const ::iovec,
                liovcnt: ::c_ulong,
                remote_iov: *const ::iovec,
                riovcnt: ::c_ulong,
                flags: ::c_ulong,
            ) -> isize;
            pub fn futimes(
                fd: ::c_int,
                times: *const ::timeval
            ) -> ::c_int;
        }
    }
}

// These functions are not available on OpenHarmony
cfg_if! {
    if #[cfg(not(target_env = "ohos"))] {
        extern "C" {
            // Only `getspnam_r` is implemented for musl, out of all of the reenterant
            // functions from `shadow.h`.
            // https://git.musl-libc.org/cgit/musl/tree/include/shadow.h
            pub fn getspnam_r(
                name: *const ::c_char,
                spbuf: *mut spwd,
                buf: *mut ::c_char,
                buflen: ::size_t,
                spbufp: *mut *mut spwd,
            ) -> ::c_int;

            pub fn shm_open(name: *const c_char, oflag: ::c_int, mode: mode_t) -> ::c_int;
            pub fn shm_unlink(name: *const ::c_char) -> ::c_int;

            pub fn mq_open(name: *const ::c_char, oflag: ::c_int, ...) -> ::mqd_t;
            pub fn mq_close(mqd: ::mqd_t) -> ::c_int;
            pub fn mq_unlink(name: *const ::c_char) -> ::c_int;
            pub fn mq_receive(
                mqd: ::mqd_t,
                msg_ptr: *mut ::c_char,
                msg_len: ::size_t,
                msg_prio: *mut ::c_uint,
            ) -> ::ssize_t;
            pub fn mq_timedreceive(
                mqd: ::mqd_t,
                msg_ptr: *mut ::c_char,
                msg_len: ::size_t,
                msg_prio: *mut ::c_uint,
                abs_timeout: *const ::timespec,
            ) -> ::ssize_t;
            pub fn mq_send(
                mqd: ::mqd_t,
                msg_ptr: *const ::c_char,
                msg_len: ::size_t,
                msg_prio: ::c_uint,
            ) -> ::c_int;
            pub fn mq_timedsend(
                mqd: ::mqd_t,
                msg_ptr: *const ::c_char,
                msg_len: ::size_t,
                msg_prio: ::c_uint,
                abs_timeout: *const ::timespec,
            ) -> ::c_int;
            pub fn mq_getattr(mqd: ::mqd_t, attr: *mut ::mq_attr) -> ::c_int;
            pub fn mq_setattr(
                mqd: ::mqd_t,
                newattr: *const ::mq_attr,
                oldattr: *mut ::mq_attr
            ) -> ::c_int;

            pub fn pthread_mutex_consistent(mutex: *mut pthread_mutex_t) -> ::c_int;
            pub fn pthread_cancel(thread: ::pthread_t) -> ::c_int;
            pub fn pthread_mutexattr_getrobust(
                attr: *const pthread_mutexattr_t,
                robustness: *mut ::c_int,
            ) -> ::c_int;
            pub fn pthread_mutexattr_setrobust(
                attr: *mut pthread_mutexattr_t,
                robustness: ::c_int,
            ) -> ::c_int;
        }
    }
}

extern "C" {
    #[cfg_attr(
        not(any(target_env = "musl", target_env = "ohos")),
        link_name = "__xpg_strerror_r"
    )]
    pub fn strerror_r(errnum: ::c_int, buf: *mut c_char, buflen: ::size_t) -> ::c_int;

    pub fn abs(i: ::c_int) -> ::c_int;
    pub fn labs(i: ::c_long) -> ::c_long;
    pub fn rand() -> ::c_int;
    pub fn srand(seed: ::c_uint);

    pub fn drand48() -> ::c_double;
    pub fn erand48(xseed: *mut ::c_ushort) -> ::c_double;
    pub fn lrand48() -> ::c_long;
    pub fn nrand48(xseed: *mut ::c_ushort) -> ::c_long;
    pub fn mrand48() -> ::c_long;
    pub fn jrand48(xseed: *mut ::c_ushort) -> ::c_long;
    pub fn srand48(seed: ::c_long);
    pub fn seed48(xseed: *mut ::c_ushort) -> *mut ::c_ushort;
    pub fn lcong48(p: *mut ::c_ushort);

    pub fn lutimes(file: *const ::c_char, times: *const ::timeval) -> ::c_int;

    pub fn setpwent();
    pub fn endpwent();
    pub fn getpwent() -> *mut passwd;
    pub fn setgrent();
    pub fn endgrent();
    pub fn getgrent() -> *mut ::group;
    pub fn setspent();
    pub fn endspent();
    pub fn getspent() -> *mut spwd;

    pub fn getspnam(name: *const ::c_char) -> *mut spwd;

    // System V IPC
    pub fn shmget(key: ::key_t, size: ::size_t, shmflg: ::c_int) -> ::c_int;
    pub fn shmat(shmid: ::c_int, shmaddr: *const ::c_void, shmflg: ::c_int) -> *mut ::c_void;
    pub fn shmdt(shmaddr: *const ::c_void) -> ::c_int;
    pub fn shmctl(shmid: ::c_int, cmd: ::c_int, buf: *mut ::shmid_ds) -> ::c_int;
    pub fn ftok(pathname: *const ::c_char, proj_id: ::c_int) -> ::key_t;
    pub fn semget(key: ::key_t, nsems: ::c_int, semflag: ::c_int) -> ::c_int;
    pub fn semop(semid: ::c_int, sops: *mut ::sembuf, nsops: ::size_t) -> ::c_int;
    pub fn semctl(semid: ::c_int, semnum: ::c_int, cmd: ::c_int, ...) -> ::c_int;
    pub fn msgctl(msqid: ::c_int, cmd: ::c_int, buf: *mut msqid_ds) -> ::c_int;
    pub fn msgget(key: ::key_t, msgflg: ::c_int) -> ::c_int;
    pub fn msgrcv(
        msqid: ::c_int,
        msgp: *mut ::c_void,
        msgsz: ::size_t,
        msgtyp: ::c_long,
        msgflg: ::c_int,
    ) -> ::ssize_t;
    pub fn msgsnd(
        msqid: ::c_int,
        msgp: *const ::c_void,
        msgsz: ::size_t,
        msgflg: ::c_int,
    ) -> ::c_int;

    pub fn mprotect(addr: *mut ::c_void, len: ::size_t, prot: ::c_int) -> ::c_int;
    pub fn __errno_location() -> *mut ::c_int;

    pub fn fallocate(fd: ::c_int, mode: ::c_int, offset: ::off_t, len: ::off_t) -> ::c_int;
    pub fn posix_fallocate(fd: ::c_int, offset: ::off_t, len: ::off_t) -> ::c_int;
    pub fn readahead(fd: ::c_int, offset: ::off64_t, count: ::size_t) -> ::ssize_t;
    pub fn getxattr(
        path: *const c_char,
        name: *const c_char,
        value: *mut ::c_void,
        size: ::size_t,
    ) -> ::ssize_t;
    pub fn lgetxattr(
        path: *const c_char,
        name: *const c_char,
        value: *mut ::c_void,
        size: ::size_t,
    ) -> ::ssize_t;
    pub fn fgetxattr(
        filedes: ::c_int,
        name: *const c_char,
        value: *mut ::c_void,
        size: ::size_t,
    ) -> ::ssize_t;
    pub fn setxattr(
        path: *const c_char,
        name: *const c_char,
        value: *const ::c_void,
        size: ::size_t,
        flags: ::c_int,
    ) -> ::c_int;
    pub fn lsetxattr(
        path: *const c_char,
        name: *const c_char,
        value: *const ::c_void,
        size: ::size_t,
        flags: ::c_int,
    ) -> ::c_int;
    pub fn fsetxattr(
        filedes: ::c_int,
        name: *const c_char,
        value: *const ::c_void,
        size: ::size_t,
        flags: ::c_int,
    ) -> ::c_int;
    pub fn listxattr(path: *const c_char, list: *mut c_char, size: ::size_t) -> ::ssize_t;
    pub fn llistxattr(path: *const c_char, list: *mut c_char, size: ::size_t) -> ::ssize_t;
    pub fn flistxattr(filedes: ::c_int, list: *mut c_char, size: ::size_t) -> ::ssize_t;
    pub fn removexattr(path: *const c_char, name: *const c_char) -> ::c_int;
    pub fn lremovexattr(path: *const c_char, name: *const c_char) -> ::c_int;
    pub fn fremovexattr(filedes: ::c_int, name: *const c_char) -> ::c_int;
    pub fn signalfd(fd: ::c_int, mask: *const ::sigset_t, flags: ::c_int) -> ::c_int;
    pub fn timerfd_create(clockid: ::clockid_t, flags: ::c_int) -> ::c_int;
    pub fn timerfd_gettime(fd: ::c_int, curr_value: *mut itimerspec) -> ::c_int;
    pub fn timerfd_settime(
        fd: ::c_int,
        flags: ::c_int,
        new_value: *const itimerspec,
        old_value: *mut itimerspec,
    ) -> ::c_int;
    pub fn quotactl(
        cmd: ::c_int,
        special: *const ::c_char,
        id: ::c_int,
        data: *mut ::c_char,
    ) -> ::c_int;
    pub fn epoll_pwait(
        epfd: ::c_int,
        events: *mut ::epoll_event,
        maxevents: ::c_int,
        timeout: ::c_int,
        sigmask: *const ::sigset_t,
    ) -> ::c_int;
    pub fn dup3(oldfd: ::c_int, newfd: ::c_int, flags: ::c_int) -> ::c_int;
    pub fn mkostemp(template: *mut ::c_char, flags: ::c_int) -> ::c_int;
    pub fn mkostemps(template: *mut ::c_char, suffixlen: ::c_int, flags: ::c_int) -> ::c_int;
    pub fn sigtimedwait(
        set: *const sigset_t,
        info: *mut siginfo_t,
        timeout: *const ::timespec,
    ) -> ::c_int;
    pub fn sigwaitinfo(set: *const sigset_t, info: *mut siginfo_t) -> ::c_int;
    pub fn nl_langinfo_l(item: ::nl_item, locale: ::locale_t) -> *mut ::c_char;
    pub fn accept4(
        fd: ::c_int,
        addr: *mut ::sockaddr,
        len: *mut ::socklen_t,
        flg: ::c_int,
    ) -> ::c_int;
    pub fn pthread_getaffinity_np(
        thread: ::pthread_t,
        cpusetsize: ::size_t,
        cpuset: *mut ::cpu_set_t,
    ) -> ::c_int;
    pub fn pthread_setaffinity_np(
        thread: ::pthread_t,
        cpusetsize: ::size_t,
        cpuset: *const ::cpu_set_t,
    ) -> ::c_int;
    pub fn pthread_setschedprio(native: ::pthread_t, priority: ::c_int) -> ::c_int;
    pub fn reboot(how_to: ::c_int) -> ::c_int;
    pub fn setfsgid(gid: ::gid_t) -> ::c_int;
    pub fn setfsuid(uid: ::uid_t) -> ::c_int;

    // Not available now on Android
    pub fn mkfifoat(dirfd: ::c_int, pathname: *const ::c_char, mode: ::mode_t) -> ::c_int;
    pub fn if_nameindex() -> *mut if_nameindex;
    pub fn if_freenameindex(ptr: *mut if_nameindex);
    pub fn sync_file_range(
        fd: ::c_int,
        offset: ::off64_t,
        nbytes: ::off64_t,
        flags: ::c_uint,
    ) -> ::c_int;
    pub fn mremap(
        addr: *mut ::c_void,
        len: ::size_t,
        new_len: ::size_t,
        flags: ::c_int,
        ...
    ) -> *mut ::c_void;

    pub fn glob(
        pattern: *const c_char,
        flags: ::c_int,
        errfunc: ::Option<extern "C" fn(epath: *const c_char, errno: ::c_int) -> ::c_int>,
        pglob: *mut ::glob_t,
    ) -> ::c_int;
    pub fn globfree(pglob: *mut ::glob_t);

    pub fn posix_madvise(addr: *mut ::c_void, len: ::size_t, advice: ::c_int) -> ::c_int;

    pub fn seekdir(dirp: *mut ::DIR, loc: ::c_long);

    pub fn telldir(dirp: *mut ::DIR) -> ::c_long;
    pub fn madvise(addr: *mut ::c_void, len: ::size_t, advice: ::c_int) -> ::c_int;

    pub fn msync(addr: *mut ::c_void, len: ::size_t, flags: ::c_int) -> ::c_int;
    pub fn remap_file_pages(
        addr: *mut ::c_void,
        size: ::size_t,
        prot: ::c_int,
        pgoff: ::size_t,
        flags: ::c_int,
    ) -> ::c_int;
    pub fn recvfrom(
        socket: ::c_int,
        buf: *mut ::c_void,
        len: ::size_t,
        flags: ::c_int,
        addr: *mut ::sockaddr,
        addrlen: *mut ::socklen_t,
    ) -> ::ssize_t;
    pub fn mkstemps(template: *mut ::c_char, suffixlen: ::c_int) -> ::c_int;

    pub fn nl_langinfo(item: ::nl_item) -> *mut ::c_char;

    pub fn getdomainname(name: *mut ::c_char, len: ::size_t) -> ::c_int;
    pub fn setdomainname(name: *const ::c_char, len: ::size_t) -> ::c_int;
    pub fn vhangup() -> ::c_int;
    pub fn sync();
    pub fn syncfs(fd: ::c_int) -> ::c_int;
    pub fn syscall(num: ::c_long, ...) -> ::c_long;
    pub fn sched_getaffinity(pid: ::pid_t, cpusetsize: ::size_t, cpuset: *mut cpu_set_t)
        -> ::c_int;
    pub fn sched_setaffinity(
        pid: ::pid_t,
        cpusetsize: ::size_t,
        cpuset: *const cpu_set_t,
    ) -> ::c_int;
    pub fn epoll_create(size: ::c_int) -> ::c_int;
    pub fn epoll_create1(flags: ::c_int) -> ::c_int;
    pub fn epoll_wait(
        epfd: ::c_int,
        events: *mut ::epoll_event,
        maxevents: ::c_int,
        timeout: ::c_int,
    ) -> ::c_int;
    pub fn epoll_ctl(epfd: ::c_int, op: ::c_int, fd: ::c_int, event: *mut ::epoll_event)
        -> ::c_int;
    pub fn pthread_getschedparam(
        native: ::pthread_t,
        policy: *mut ::c_int,
        param: *mut ::sched_param,
    ) -> ::c_int;
    pub fn unshare(flags: ::c_int) -> ::c_int;
    pub fn umount(target: *const ::c_char) -> ::c_int;
    pub fn sched_get_priority_max(policy: ::c_int) -> ::c_int;
    pub fn tee(fd_in: ::c_int, fd_out: ::c_int, len: ::size_t, flags: ::c_uint) -> ::ssize_t;
    pub fn settimeofday(tv: *const ::timeval, tz: *const ::timezone) -> ::c_int;
    pub fn splice(
        fd_in: ::c_int,
        off_in: *mut ::loff_t,
        fd_out: ::c_int,
        off_out: *mut ::loff_t,
        len: ::size_t,
        flags: ::c_uint,
    ) -> ::ssize_t;
    pub fn eventfd(init: ::c_uint, flags: ::c_int) -> ::c_int;
    pub fn sched_rr_get_interval(pid: ::pid_t, tp: *mut ::timespec) -> ::c_int;
    pub fn sem_timedwait(sem: *mut sem_t, abstime: *const ::timespec) -> ::c_int;
    pub fn sem_getvalue(sem: *mut sem_t, sval: *mut ::c_int) -> ::c_int;
    pub fn sched_setparam(pid: ::pid_t, param: *const ::sched_param) -> ::c_int;
    pub fn setns(fd: ::c_int, nstype: ::c_int) -> ::c_int;
    pub fn swapoff(path: *const ::c_char) -> ::c_int;
    pub fn vmsplice(
        fd: ::c_int,
        iov: *const ::iovec,
        nr_segs: ::size_t,
        flags: ::c_uint,
    ) -> ::ssize_t;
    pub fn mount(
        src: *const ::c_char,
        target: *const ::c_char,
        fstype: *const ::c_char,
        flags: ::c_ulong,
        data: *const ::c_void,
    ) -> ::c_int;
    pub fn personality(persona: ::c_ulong) -> ::c_int;
    pub fn prctl(option: ::c_int, ...) -> ::c_int;
    pub fn sched_getparam(pid: ::pid_t, param: *mut ::sched_param) -> ::c_int;
    pub fn ppoll(
        fds: *mut ::pollfd,
        nfds: nfds_t,
        timeout: *const ::timespec,
        sigmask: *const sigset_t,
    ) -> ::c_int;
    pub fn pthread_mutexattr_getprotocol(
        attr: *const pthread_mutexattr_t,
        protocol: *mut ::c_int,
    ) -> ::c_int;
    pub fn pthread_mutexattr_setprotocol(
        attr: *mut pthread_mutexattr_t,
        protocol: ::c_int,
    ) -> ::c_int;

    pub fn pthread_mutex_timedlock(
        lock: *mut pthread_mutex_t,
        abstime: *const ::timespec,
    ) -> ::c_int;
    pub fn pthread_barrierattr_init(attr: *mut ::pthread_barrierattr_t) -> ::c_int;
    pub fn pthread_barrierattr_destroy(attr: *mut ::pthread_barrierattr_t) -> ::c_int;
    pub fn pthread_barrierattr_getpshared(
        attr: *const ::pthread_barrierattr_t,
        shared: *mut ::c_int,
    ) -> ::c_int;
    pub fn pthread_barrierattr_setpshared(
        attr: *mut ::pthread_barrierattr_t,
        shared: ::c_int,
    ) -> ::c_int;
    pub fn pthread_barrier_init(
        barrier: *mut pthread_barrier_t,
        attr: *const ::pthread_barrierattr_t,
        count: ::c_uint,
    ) -> ::c_int;
    pub fn pthread_barrier_destroy(barrier: *mut pthread_barrier_t) -> ::c_int;
    pub fn pthread_barrier_wait(barrier: *mut pthread_barrier_t) -> ::c_int;
    pub fn pthread_spin_init(lock: *mut ::pthread_spinlock_t, pshared: ::c_int) -> ::c_int;
    pub fn pthread_spin_destroy(lock: *mut ::pthread_spinlock_t) -> ::c_int;
    pub fn pthread_spin_lock(lock: *mut ::pthread_spinlock_t) -> ::c_int;
    pub fn pthread_spin_trylock(lock: *mut ::pthread_spinlock_t) -> ::c_int;
    pub fn pthread_spin_unlock(lock: *mut ::pthread_spinlock_t) -> ::c_int;
    pub fn clone(
        cb: extern "C" fn(*mut ::c_void) -> ::c_int,
        child_stack: *mut ::c_void,
        flags: ::c_int,
        arg: *mut ::c_void,
        ...
    ) -> ::c_int;
    pub fn sched_getscheduler(pid: ::pid_t) -> ::c_int;
    pub fn clock_nanosleep(
        clk_id: ::clockid_t,
        flags: ::c_int,
        rqtp: *const ::timespec,
        rmtp: *mut ::timespec,
    ) -> ::c_int;
    pub fn pthread_attr_getguardsize(
        attr: *const ::pthread_attr_t,
        guardsize: *mut ::size_t,
    ) -> ::c_int;
    pub fn sethostname(name: *const ::c_char, len: ::size_t) -> ::c_int;
    pub fn sched_get_priority_min(policy: ::c_int) -> ::c_int;
    pub fn pthread_condattr_getpshared(
        attr: *const pthread_condattr_t,
        pshared: *mut ::c_int,
    ) -> ::c_int;
    pub fn sysinfo(info: *mut ::sysinfo) -> ::c_int;
    pub fn umount2(target: *const ::c_char, flags: ::c_int) -> ::c_int;
    pub fn pthread_setschedparam(
        native: ::pthread_t,
        policy: ::c_int,
        param: *const ::sched_param,
    ) -> ::c_int;
    pub fn swapon(path: *const ::c_char, swapflags: ::c_int) -> ::c_int;
    pub fn sched_setscheduler(
        pid: ::pid_t,
        policy: ::c_int,
        param: *const ::sched_param,
    ) -> ::c_int;
    pub fn sendfile(
        out_fd: ::c_int,
        in_fd: ::c_int,
        offset: *mut off_t,
        count: ::size_t,
    ) -> ::ssize_t;
    pub fn sigsuspend(mask: *const ::sigset_t) -> ::c_int;
    pub fn getgrgid_r(
        gid: ::gid_t,
        grp: *mut ::group,
        buf: *mut ::c_char,
        buflen: ::size_t,
        result: *mut *mut ::group,
    ) -> ::c_int;
    pub fn sigaltstack(ss: *const stack_t, oss: *mut stack_t) -> ::c_int;
    pub fn sem_close(sem: *mut sem_t) -> ::c_int;
    pub fn getdtablesize() -> ::c_int;
    pub fn getgrnam_r(
        name: *const ::c_char,
        grp: *mut ::group,
        buf: *mut ::c_char,
        buflen: ::size_t,
        result: *mut *mut ::group,
    ) -> ::c_int;
    pub fn initgroups(user: *const ::c_char, group: ::gid_t) -> ::c_int;
    pub fn pthread_sigmask(how: ::c_int, set: *const sigset_t, oldset: *mut sigset_t) -> ::c_int;
    pub fn sem_open(name: *const ::c_char, oflag: ::c_int, ...) -> *mut sem_t;
    pub fn getgrnam(name: *const ::c_char) -> *mut ::group;
    pub fn pthread_kill(thread: ::pthread_t, sig: ::c_int) -> ::c_int;
    pub fn sem_unlink(name: *const ::c_char) -> ::c_int;
    pub fn daemon(nochdir: ::c_int, noclose: ::c_int) -> ::c_int;
    pub fn getpwnam_r(
        name: *const ::c_char,
        pwd: *mut passwd,
        buf: *mut ::c_char,
        buflen: ::size_t,
        result: *mut *mut passwd,
    ) -> ::c_int;
    pub fn getpwuid_r(
        uid: ::uid_t,
        pwd: *mut passwd,
        buf: *mut ::c_char,
        buflen: ::size_t,
        result: *mut *mut passwd,
    ) -> ::c_int;
    pub fn sigwait(set: *const sigset_t, sig: *mut ::c_int) -> ::c_int;
    pub fn pthread_atfork(
        prepare: ::Option<unsafe extern "C" fn()>,
        parent: ::Option<unsafe extern "C" fn()>,
        child: ::Option<unsafe extern "C" fn()>,
    ) -> ::c_int;
    pub fn getgrgid(gid: ::gid_t) -> *mut ::group;
    pub fn getgrouplist(
        user: *const ::c_char,
        group: ::gid_t,
        groups: *mut ::gid_t,
        ngroups: *mut ::c_int,
    ) -> ::c_int;
    pub fn pthread_mutexattr_getpshared(
        attr: *const pthread_mutexattr_t,
        pshared: *mut ::c_int,
    ) -> ::c_int;
    pub fn popen(command: *const c_char, mode: *const c_char) -> *mut ::FILE;
    pub fn faccessat(
        dirfd: ::c_int,
        pathname: *const ::c_char,
        mode: ::c_int,
        flags: ::c_int,
    ) -> ::c_int;
    pub fn pthread_create(
        native: *mut ::pthread_t,
        attr: *const ::pthread_attr_t,
        f: extern "C" fn(*mut ::c_void) -> *mut ::c_void,
        value: *mut ::c_void,
    ) -> ::c_int;
    pub fn dl_iterate_phdr(
        callback: ::Option<
            unsafe extern "C" fn(
                info: *mut ::dl_phdr_info,
                size: ::size_t,
                data: *mut ::c_void,
            ) -> ::c_int,
        >,
        data: *mut ::c_void,
    ) -> ::c_int;

    pub fn setmntent(filename: *const ::c_char, ty: *const ::c_char) -> *mut ::FILE;
    pub fn getmntent(stream: *mut ::FILE) -> *mut ::mntent;
    pub fn addmntent(stream: *mut ::FILE, mnt: *const ::mntent) -> ::c_int;
    pub fn endmntent(streamp: *mut ::FILE) -> ::c_int;
    pub fn hasmntopt(mnt: *const ::mntent, opt: *const ::c_char) -> *mut ::c_char;

    pub fn posix_spawn(
        pid: *mut ::pid_t,
        path: *const ::c_char,
        file_actions: *const ::posix_spawn_file_actions_t,
        attrp: *const ::posix_spawnattr_t,
        argv: *const *mut ::c_char,
        envp: *const *mut ::c_char,
    ) -> ::c_int;
    pub fn posix_spawnp(
        pid: *mut ::pid_t,
        file: *const ::c_char,
        file_actions: *const ::posix_spawn_file_actions_t,
        attrp: *const ::posix_spawnattr_t,
        argv: *const *mut ::c_char,
        envp: *const *mut ::c_char,
    ) -> ::c_int;
    pub fn posix_spawnattr_init(attr: *mut posix_spawnattr_t) -> ::c_int;
    pub fn posix_spawnattr_destroy(attr: *mut posix_spawnattr_t) -> ::c_int;
    pub fn posix_spawnattr_getsigdefault(
        attr: *const posix_spawnattr_t,
        default: *mut ::sigset_t,
    ) -> ::c_int;
    pub fn posix_spawnattr_setsigdefault(
        attr: *mut posix_spawnattr_t,
        default: *const ::sigset_t,
    ) -> ::c_int;
    pub fn posix_spawnattr_getsigmask(
        attr: *const posix_spawnattr_t,
        default: *mut ::sigset_t,
    ) -> ::c_int;
    pub fn posix_spawnattr_setsigmask(
        attr: *mut posix_spawnattr_t,
        default: *const ::sigset_t,
    ) -> ::c_int;
    pub fn posix_spawnattr_getflags(
        attr: *const posix_spawnattr_t,
        flags: *mut ::c_short,
    ) -> ::c_int;
    pub fn posix_spawnattr_setflags(attr: *mut posix_spawnattr_t, flags: ::c_short) -> ::c_int;
    pub fn posix_spawnattr_getpgroup(
        attr: *const posix_spawnattr_t,
        flags: *mut ::pid_t,
    ) -> ::c_int;
    pub fn posix_spawnattr_setpgroup(attr: *mut posix_spawnattr_t, flags: ::pid_t) -> ::c_int;
    pub fn posix_spawnattr_getschedpolicy(
        attr: *const posix_spawnattr_t,
        flags: *mut ::c_int,
    ) -> ::c_int;
    pub fn posix_spawnattr_setschedpolicy(attr: *mut posix_spawnattr_t, flags: ::c_int) -> ::c_int;
    pub fn posix_spawnattr_getschedparam(
        attr: *const posix_spawnattr_t,
        param: *mut ::sched_param,
    ) -> ::c_int;
    pub fn posix_spawnattr_setschedparam(
        attr: *mut posix_spawnattr_t,
        param: *const ::sched_param,
    ) -> ::c_int;

    pub fn posix_spawn_file_actions_init(actions: *mut posix_spawn_file_actions_t) -> ::c_int;
    pub fn posix_spawn_file_actions_destroy(actions: *mut posix_spawn_file_actions_t) -> ::c_int;
    pub fn posix_spawn_file_actions_addopen(
        actions: *mut posix_spawn_file_actions_t,
        fd: ::c_int,
        path: *const ::c_char,
        oflag: ::c_int,
        mode: ::mode_t,
    ) -> ::c_int;
    pub fn posix_spawn_file_actions_addclose(
        actions: *mut posix_spawn_file_actions_t,
        fd: ::c_int,
    ) -> ::c_int;
    pub fn posix_spawn_file_actions_adddup2(
        actions: *mut posix_spawn_file_actions_t,
        fd: ::c_int,
        newfd: ::c_int,
    ) -> ::c_int;
    pub fn fread_unlocked(
        ptr: *mut ::c_void,
        size: ::size_t,
        nobj: ::size_t,
        stream: *mut ::FILE,
    ) -> ::size_t;
    pub fn inotify_rm_watch(fd: ::c_int, wd: ::c_int) -> ::c_int;
    pub fn inotify_init() -> ::c_int;
    pub fn inotify_init1(flags: ::c_int) -> ::c_int;
    pub fn inotify_add_watch(fd: ::c_int, path: *const ::c_char, mask: u32) -> ::c_int;
    pub fn fanotify_init(flags: ::c_uint, event_f_flags: ::c_uint) -> ::c_int;

    pub fn regcomp(preg: *mut ::regex_t, pattern: *const ::c_char, cflags: ::c_int) -> ::c_int;

    pub fn regexec(
        preg: *const ::regex_t,
        input: *const ::c_char,
        nmatch: ::size_t,
        pmatch: *mut regmatch_t,
        eflags: ::c_int,
    ) -> ::c_int;

    pub fn regerror(
        errcode: ::c_int,
        preg: *const ::regex_t,
        errbuf: *mut ::c_char,
        errbuf_size: ::size_t,
    ) -> ::size_t;

    pub fn regfree(preg: *mut ::regex_t);

    pub fn iconv_open(tocode: *const ::c_char, fromcode: *const ::c_char) -> iconv_t;
    pub fn iconv(
        cd: iconv_t,
        inbuf: *mut *mut ::c_char,
        inbytesleft: *mut ::size_t,
        outbuf: *mut *mut ::c_char,
        outbytesleft: *mut ::size_t,
    ) -> ::size_t;
    pub fn iconv_close(cd: iconv_t) -> ::c_int;

    pub fn gettid() -> ::pid_t;

    pub fn timer_create(
        clockid: ::clockid_t,
        sevp: *mut ::sigevent,
        timerid: *mut ::timer_t,
    ) -> ::c_int;
    pub fn timer_delete(timerid: ::timer_t) -> ::c_int;
    pub fn timer_getoverrun(timerid: ::timer_t) -> ::c_int;
    pub fn timer_gettime(timerid: ::timer_t, curr_value: *mut ::itimerspec) -> ::c_int;
    pub fn timer_settime(
        timerid: ::timer_t,
        flags: ::c_int,
        new_value: *const ::itimerspec,
        old_value: *mut ::itimerspec,
    ) -> ::c_int;

    pub fn gethostid() -> ::c_long;

    pub fn pthread_getcpuclockid(thread: ::pthread_t, clk_id: *mut ::clockid_t) -> ::c_int;
    pub fn memmem(
        haystack: *const ::c_void,
        haystacklen: ::size_t,
        needle: *const ::c_void,
        needlelen: ::size_t,
    ) -> *mut ::c_void;
    pub fn sched_getcpu() -> ::c_int;

    pub fn pthread_getname_np(thread: ::pthread_t, name: *mut ::c_char, len: ::size_t) -> ::c_int;
    pub fn pthread_setname_np(thread: ::pthread_t, name: *const ::c_char) -> ::c_int;
    pub fn getopt_long(
        argc: ::c_int,
        argv: *const *mut c_char,
        optstring: *const c_char,
        longopts: *const option,
        longindex: *mut ::c_int,
    ) -> ::c_int;

    pub fn copy_file_range(
        fd_in: ::c_int,
        off_in: *mut ::off64_t,
        fd_out: ::c_int,
        off_out: *mut ::off64_t,
        len: ::size_t,
        flags: ::c_uint,
    ) -> ::ssize_t;
}

// LFS64 extensions
//
// * musl has 64-bit versions only so aliases the LFS64 symbols to the standard ones
cfg_if! {
    if #[cfg(not(target_env = "musl"))] {
        extern "C" {
            pub fn fallocate64(
                fd: ::c_int,
                mode: ::c_int,
                offset: ::off64_t,
                len: ::off64_t
            ) -> ::c_int;
            pub fn fgetpos64(stream: *mut ::FILE, ptr: *mut fpos64_t) -> ::c_int;
            pub fn fopen64(filename: *const c_char, mode: *const c_char) -> *mut ::FILE;
            pub fn freopen64(
                filename: *const c_char,
                mode: *const c_char,
                file: *mut ::FILE,
            ) -> *mut ::FILE;
            pub fn fseeko64(stream: *mut ::FILE, offset: ::off64_t, whence: ::c_int) -> ::c_int;
            pub fn fsetpos64(stream: *mut ::FILE, ptr: *const fpos64_t) -> ::c_int;
            pub fn ftello64(stream: *mut ::FILE) -> ::off64_t;
            pub fn posix_fallocate64(fd: ::c_int, offset: ::off64_t, len: ::off64_t) -> ::c_int;
            pub fn sendfile64(
                out_fd: ::c_int,
                in_fd: ::c_int,
                offset: *mut off64_t,
                count: ::size_t,
            ) -> ::ssize_t;
            pub fn tmpfile64() -> *mut ::FILE;
        }
    }
}

cfg_if! {
    if #[cfg(target_env = "uclibc")] {
        mod uclibc;
        pub use self::uclibc::*;
    } else if #[cfg(any(target_env = "musl", target_env = "ohos"))] {
        mod musl;
        pub use self::musl::*;
    } else if #[cfg(target_env = "gnu")] {
        mod gnu;
        pub use self::gnu::*;
    }
}

mod arch;
pub use self::arch::*;

cfg_if! {
    if #[cfg(libc_align)] {
        #[macro_use]
        mod align;
    } else {
        #[macro_use]
        mod no_align;
    }
}
expand_align!();

cfg_if! {
    if #[cfg(libc_non_exhaustive)] {
        mod non_exhaustive;
        pub use self::non_exhaustive::*;
    }
}
