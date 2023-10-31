//! `hermit-abi` is small interface to call functions from the unikernel
//! [RustyHermit](https://github.com/hermitcore/libhermit-rs).

#![no_std]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::result_unit_err)]

pub mod errno;
pub mod tcplistener;
pub mod tcpstream;

use core::mem::MaybeUninit;

use libc::c_void;

// sysmbols, which are part of the library operating system

extern "C" {
	fn sys_rand() -> u32;
	fn sys_srand(seed: u32);
	fn sys_secure_rand32(value: *mut u32) -> i32;
	fn sys_secure_rand64(value: *mut u64) -> i32;
	fn sys_get_processor_count() -> usize;
	fn sys_malloc(size: usize, align: usize) -> *mut u8;
	fn sys_realloc(ptr: *mut u8, size: usize, align: usize, new_size: usize) -> *mut u8;
	fn sys_free(ptr: *mut u8, size: usize, align: usize);
	fn sys_init_queue(ptr: usize) -> i32;
	fn sys_notify(id: usize, count: i32) -> i32;
	fn sys_add_queue(id: usize, timeout_ns: i64) -> i32;
	fn sys_wait(id: usize) -> i32;
	fn sys_destroy_queue(id: usize) -> i32;
	fn sys_read(fd: i32, buf: *mut u8, len: usize) -> isize;
	fn sys_write(fd: i32, buf: *const u8, len: usize) -> isize;
	fn sys_close(fd: i32) -> i32;
	fn sys_futex_wait(
		address: *mut u32,
		expected: u32,
		timeout: *const timespec,
		flags: u32,
	) -> i32;
	fn sys_futex_wake(address: *mut u32, count: i32) -> i32;
	fn sys_sem_init(sem: *mut *const c_void, value: u32) -> i32;
	fn sys_sem_destroy(sem: *const c_void) -> i32;
	fn sys_sem_post(sem: *const c_void) -> i32;
	fn sys_sem_trywait(sem: *const c_void) -> i32;
	fn sys_sem_timedwait(sem: *const c_void, ms: u32) -> i32;
	fn sys_recmutex_init(recmutex: *mut *const c_void) -> i32;
	fn sys_recmutex_destroy(recmutex: *const c_void) -> i32;
	fn sys_recmutex_lock(recmutex: *const c_void) -> i32;
	fn sys_recmutex_unlock(recmutex: *const c_void) -> i32;
	fn sys_getpid() -> u32;
	fn sys_exit(arg: i32) -> !;
	fn sys_abort() -> !;
	fn sys_usleep(usecs: u64);
	fn sys_spawn(
		id: *mut Tid,
		func: extern "C" fn(usize),
		arg: usize,
		prio: u8,
		core_id: isize,
	) -> i32;
	fn sys_spawn2(
		func: extern "C" fn(usize),
		arg: usize,
		prio: u8,
		stack_size: usize,
		core_id: isize,
	) -> Tid;
	fn sys_join(id: Tid) -> i32;
	fn sys_yield();
	fn sys_clock_gettime(clock_id: u64, tp: *mut timespec) -> i32;
	fn sys_open(name: *const i8, flags: i32, mode: i32) -> i32;
	fn sys_unlink(name: *const i8) -> i32;
	fn sys_network_init() -> i32;
	fn sys_block_current_task();
	fn sys_block_current_task_with_timeout(timeout: u64);
	fn sys_wakeup_task(tid: Tid);
	fn sys_get_priority() -> u8;
	fn sys_set_priority(tid: Tid, prio: u8);
}

/// A thread handle type
pub type Tid = u32;

/// Maximum number of priorities
pub const NO_PRIORITIES: usize = 31;

/// Priority of a thread
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct Priority(u8);

impl Priority {
	pub const fn into(self) -> u8 {
		self.0
	}

	pub const fn from(x: u8) -> Self {
		Priority(x)
	}
}

pub const HIGH_PRIO: Priority = Priority::from(3);
pub const NORMAL_PRIO: Priority = Priority::from(2);
pub const LOW_PRIO: Priority = Priority::from(1);

/// A handle, identifying a socket
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Handle(usize);

pub const NSEC_PER_SEC: u64 = 1_000_000_000;
pub const FUTEX_RELATIVE_TIMEOUT: u32 = 1;
pub const CLOCK_REALTIME: u64 = 1;
pub const CLOCK_MONOTONIC: u64 = 4;
pub const STDIN_FILENO: libc::c_int = 0;
pub const STDOUT_FILENO: libc::c_int = 1;
pub const STDERR_FILENO: libc::c_int = 2;
pub const O_RDONLY: i32 = 0o0;
pub const O_WRONLY: i32 = 0o1;
pub const O_RDWR: i32 = 0o2;
pub const O_CREAT: i32 = 0o100;
pub const O_EXCL: i32 = 0o200;
pub const O_TRUNC: i32 = 0o1000;
pub const O_APPEND: i32 = 0o2000;

/// returns true if file descriptor `fd` is a tty
pub fn isatty(_fd: libc::c_int) -> bool {
	false
}

/// initialize the network stack
pub fn network_init() -> i32 {
	unsafe { sys_network_init() }
}

/// `timespec` is used by `clock_gettime` to retrieve the
/// current time
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct timespec {
	/// seconds
	pub tv_sec: i64,
	/// nanoseconds
	pub tv_nsec: i64,
}

/// Internet protocol version.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Version {
	Unspecified,
	Ipv4,
	Ipv6,
}

/// A four-octet IPv4 address.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
pub struct Ipv4Address(pub [u8; 4]);

/// A sixteen-octet IPv6 address.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
pub struct Ipv6Address(pub [u8; 16]);

/// An internetworking address.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum IpAddress {
	/// An unspecified address.
	/// May be used as a placeholder for storage where the address is not assigned yet.
	Unspecified,
	/// An IPv4 address.
	Ipv4(Ipv4Address),
	/// An IPv6 address.
	Ipv6(Ipv6Address),
}

/// determines the number of activated processors
#[inline(always)]
pub unsafe fn get_processor_count() -> usize {
	sys_get_processor_count()
}

#[doc(hidden)]
#[inline(always)]
pub unsafe fn malloc(size: usize, align: usize) -> *mut u8 {
	sys_malloc(size, align)
}

#[doc(hidden)]
#[inline(always)]
pub unsafe fn realloc(ptr: *mut u8, size: usize, align: usize, new_size: usize) -> *mut u8 {
	sys_realloc(ptr, size, align, new_size)
}

#[doc(hidden)]
#[inline(always)]
pub unsafe fn free(ptr: *mut u8, size: usize, align: usize) {
	sys_free(ptr, size, align)
}

#[inline(always)]
pub unsafe fn notify(id: usize, count: i32) -> i32 {
	sys_notify(id, count)
}

#[doc(hidden)]
#[inline(always)]
pub unsafe fn add_queue(id: usize, timeout_ns: i64) -> i32 {
	sys_add_queue(id, timeout_ns)
}

#[doc(hidden)]
#[inline(always)]
pub unsafe fn wait(id: usize) -> i32 {
	sys_wait(id)
}

#[doc(hidden)]
#[inline(always)]
pub unsafe fn init_queue(id: usize) -> i32 {
	sys_init_queue(id)
}

#[doc(hidden)]
#[inline(always)]
pub unsafe fn destroy_queue(id: usize) -> i32 {
	sys_destroy_queue(id)
}

/// read from a file descriptor
///
/// read() attempts to read `len` bytes of data from the object
/// referenced by the descriptor `fd` into the buffer pointed
/// to by `buf`.
#[inline(always)]
pub unsafe fn read(fd: i32, buf: *mut u8, len: usize) -> isize {
	sys_read(fd, buf, len)
}

/// write to a file descriptor
///
/// write() attempts to write `len` of data to the object
/// referenced by the descriptor `fd` from the
/// buffer pointed to by `buf`.
#[inline(always)]
pub unsafe fn write(fd: i32, buf: *const u8, len: usize) -> isize {
	sys_write(fd, buf, len)
}

/// close a file descriptor
///
/// The close() call deletes a file descriptor `fd` from the object
/// reference table.
#[inline(always)]
pub unsafe fn close(fd: i32) -> i32 {
	sys_close(fd)
}

/// If the value at address matches the expected value, park the current thread until it is either
/// woken up with [`futex_wake`] (returns 0) or an optional timeout elapses (returns -ETIMEDOUT).
///
/// Setting `timeout` to null means the function will only return if [`futex_wake`] is called.
/// Otherwise, `timeout` is interpreted as an absolute time measured with [`CLOCK_MONOTONIC`].
/// If [`FUTEX_RELATIVE_TIMEOUT`] is set in `flags` the timeout is understood to be relative
/// to the current time.
///
/// Returns -EINVAL if `address` is null, the timeout is negative or `flags` contains unknown values.
#[inline(always)]
pub unsafe fn futex_wait(
	address: *mut u32,
	expected: u32,
	timeout: *const timespec,
	flags: u32,
) -> i32 {
	sys_futex_wait(address, expected, timeout, flags)
}

/// Wake `count` threads waiting on the futex at `address`. Returns the number of threads
/// woken up (saturates to `i32::MAX`). If `count` is `i32::MAX`, wake up all matching
/// waiting threads. If `count` is negative or `address` is null, returns -EINVAL.
#[inline(always)]
pub unsafe fn futex_wake(address: *mut u32, count: i32) -> i32 {
	sys_futex_wake(address, count)
}

/// sem_init() initializes the unnamed semaphore at the address
/// pointed to by `sem`.  The `value` argument specifies the
/// initial value for the semaphore.
#[inline(always)]
pub unsafe fn sem_init(sem: *mut *const c_void, value: u32) -> i32 {
	sys_sem_init(sem, value)
}

/// sem_destroy() frees the unnamed semaphore at the address
/// pointed to by `sem`.
#[inline(always)]
pub unsafe fn sem_destroy(sem: *const c_void) -> i32 {
	sys_sem_destroy(sem)
}

/// sem_post() increments the semaphore pointed to by `sem`.
/// If the semaphore's value consequently becomes greater
/// than zero, then another thread blocked in a sem_wait call
/// will be woken up and proceed to lock the semaphore.
#[inline(always)]
pub unsafe fn sem_post(sem: *const c_void) -> i32 {
	sys_sem_post(sem)
}

/// try to decrement a semaphore
///
/// sem_trywait() is the same as sem_timedwait(), except that
/// if the  decrement cannot be immediately performed, then  call
/// returns a negative value instead of blocking.
#[inline(always)]
pub unsafe fn sem_trywait(sem: *const c_void) -> i32 {
	sys_sem_trywait(sem)
}

/// decrement a semaphore
///
/// sem_timedwait() decrements the semaphore pointed to by `sem`.
/// If the semaphore's value is greater than zero, then the
/// the function returns immediately. If the semaphore currently
/// has the value zero, then the call blocks until either
/// it becomes possible to perform the decrement of the time limit
/// to wait for the semaphore is expired. A time limit `ms` of
/// means infinity waiting time.
#[inline(always)]
pub unsafe fn sem_timedwait(sem: *const c_void, ms: u32) -> i32 {
	sys_sem_timedwait(sem, ms)
}

#[doc(hidden)]
#[inline(always)]
pub unsafe fn recmutex_init(recmutex: *mut *const c_void) -> i32 {
	sys_recmutex_init(recmutex)
}

#[doc(hidden)]
#[inline(always)]
pub unsafe fn recmutex_destroy(recmutex: *const c_void) -> i32 {
	sys_recmutex_destroy(recmutex)
}

#[doc(hidden)]
#[inline(always)]
pub unsafe fn recmutex_lock(recmutex: *const c_void) -> i32 {
	sys_recmutex_lock(recmutex)
}

#[doc(hidden)]
#[inline(always)]
pub unsafe fn recmutex_unlock(recmutex: *const c_void) -> i32 {
	sys_recmutex_unlock(recmutex)
}

/// Determines the id of the current thread
#[inline(always)]
pub unsafe fn getpid() -> u32 {
	sys_getpid()
}

/// cause normal termination and return `arg`
/// to the host system
#[inline(always)]
pub unsafe fn exit(arg: i32) -> ! {
	sys_exit(arg)
}

/// cause abnormal termination
#[inline(always)]
pub unsafe fn abort() -> ! {
	sys_abort()
}

/// suspend execution for microsecond intervals
///
/// The usleep() function suspends execution of the calling
/// thread for (at least) `usecs` microseconds.
#[inline(always)]
pub unsafe fn usleep(usecs: u64) {
	sys_usleep(usecs)
}

/// spawn a new thread
///
/// spawn() starts a new thread. The new thread starts execution
/// by invoking `func(usize)`; `arg` is passed as the argument
/// to `func`. `prio` defines the priority of the new thread,
/// which can be between `LOW_PRIO` and `HIGH_PRIO`.
/// `core_id` defines the core, where the thread is located.
/// A negative value give the operating system the possibility
/// to select the core by its own.
#[inline(always)]
pub unsafe fn spawn(
	id: *mut Tid,
	func: extern "C" fn(usize),
	arg: usize,
	prio: u8,
	core_id: isize,
) -> i32 {
	sys_spawn(id, func, arg, prio, core_id)
}

/// spawn a new thread with user-specified stack size
///
/// spawn2() starts a new thread. The new thread starts execution
/// by invoking `func(usize)`; `arg` is passed as the argument
/// to `func`. `prio` defines the priority of the new thread,
/// which can be between `LOW_PRIO` and `HIGH_PRIO`.
/// `core_id` defines the core, where the thread is located.
/// A negative value give the operating system the possibility
/// to select the core by its own.
/// In contrast to spawn(), spawn2() is able to define the
/// stack size.
#[inline(always)]
pub unsafe fn spawn2(
	func: extern "C" fn(usize),
	arg: usize,
	prio: u8,
	stack_size: usize,
	core_id: isize,
) -> Tid {
	sys_spawn2(func, arg, prio, stack_size, core_id)
}

/// join with a terminated thread
///
/// The join() function waits for the thread specified by `id`
/// to terminate.
#[inline(always)]
pub unsafe fn join(id: Tid) -> i32 {
	sys_join(id)
}

/// yield the processor
///
/// causes the calling thread to relinquish the CPU. The thread
/// is moved to the end of the queue for its static priority.
#[inline(always)]
pub unsafe fn yield_now() {
	sys_yield()
}

/// get current time
///
/// The clock_gettime() functions allow the calling thread
/// to retrieve the value used by a clock which is specified
/// by `clock_id`.
///
/// `CLOCK_REALTIME`: the system's real time clock,
/// expressed as the amount of time since the Epoch.
///
/// `CLOCK_MONOTONIC`: clock that increments monotonically,
/// tracking the time since an arbitrary point
#[inline(always)]
pub unsafe fn clock_gettime(clock_id: u64, tp: *mut timespec) -> i32 {
	sys_clock_gettime(clock_id, tp)
}

/// open and possibly create a file
///
/// The open() system call opens the file specified by `name`.
/// If the specified file does not exist, it may optionally
/// be created by open().
#[inline(always)]
pub unsafe fn open(name: *const i8, flags: i32, mode: i32) -> i32 {
	sys_open(name, flags, mode)
}

/// delete the file it refers to `name`
#[inline(always)]
pub unsafe fn unlink(name: *const i8) -> i32 {
	sys_unlink(name)
}

/// The largest number `rand` will return
pub const RAND_MAX: u64 = 2_147_483_647;

/// The function computes a sequence of pseudo-random integers
/// in the range of 0 to RAND_MAX
#[inline(always)]
pub unsafe fn rand() -> u32 {
	sys_rand()
}

/// The function sets its argument as the seed for a new sequence
/// of pseudo-random numbers to be returned by `rand`
#[inline(always)]
pub unsafe fn srand(seed: u32) {
	sys_srand(seed);
}

/// Create a cryptographicly secure 32bit random number with the support of
/// the underlying hardware. If the required hardware isn't available,
/// the function returns `None`.
#[inline(always)]
pub unsafe fn secure_rand32() -> Option<u32> {
	let mut rand = MaybeUninit::uninit();
	let res = sys_secure_rand32(rand.as_mut_ptr());
	(res == 0).then(|| rand.assume_init())
}

/// Create a cryptographicly secure 64bit random number with the support of
/// the underlying hardware. If the required hardware isn't available,
/// the function returns `None`.
#[inline(always)]
pub unsafe fn secure_rand64() -> Option<u64> {
	let mut rand = MaybeUninit::uninit();
	let res = sys_secure_rand64(rand.as_mut_ptr());
	(res == 0).then(|| rand.assume_init())
}

/// Add current task to the queue of blocked tasks. After calling `block_current_task`,
/// call `yield_now` to switch to another task.
#[inline(always)]
pub unsafe fn block_current_task() {
	sys_block_current_task();
}

/// Add current task to the queue of blocked tasks, but wake it when `timeout` milliseconds
/// have elapsed.
///
/// After calling `block_current_task`, call `yield_now` to switch to another task.
#[inline(always)]
pub unsafe fn block_current_task_with_timeout(timeout: u64) {
	sys_block_current_task_with_timeout(timeout);
}

/// Wakeup task with the thread id `tid`
#[inline(always)]
pub unsafe fn wakeup_task(tid: Tid) {
	sys_wakeup_task(tid);
}

/// Determine the priority of the current thread
#[inline(always)]
pub unsafe fn get_priority() -> Priority {
	Priority::from(sys_get_priority())
}

/// Determine the priority of the current thread
#[inline(always)]
pub unsafe fn set_priority(tid: Tid, prio: Priority) {
	sys_set_priority(tid, prio.into());
}
