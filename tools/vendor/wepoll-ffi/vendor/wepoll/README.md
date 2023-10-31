# wepoll - epoll for windows

[![][ci status badge]][ci status link]

This library implements the [epoll][man epoll] API for Windows
applications. It is fast and scalable, and it closely resembles the API
and behavior of Linux' epoll.

## Rationale

Unlike Linux, OS X, and many other operating systems, Windows doesn't
have a good API for receiving socket state notifications. It only
supports the `select` and `WSAPoll` APIs, but they
[don't scale][select scale] and suffer from
[other issues][wsapoll broken].

Using I/O completion ports isn't always practical when software is
designed to be cross-platform. Wepoll offers an alternative that is
much closer to a drop-in replacement for software that was designed
to run on Linux.

## Features

* Can poll 100000s of sockets efficiently.
* Fully thread-safe.
* Multiple threads can poll the same epoll port.
* Sockets can be added to multiple epoll sets.
* All epoll events (`EPOLLIN`, `EPOLLOUT`, `EPOLLPRI`, `EPOLLRDHUP`)
  are supported.
* Level-triggered and one-shot (`EPOLLONESTHOT`) modes are supported
* Trivial to embed: you need [only two files][dist].

## Limitations

* Only works with sockets.
* Edge-triggered (`EPOLLET`) mode isn't supported.

## How to use

The library is [distributed][dist] as a single source file
([wepoll.c][wepoll.c]) and a single header file ([wepoll.h][wepoll.h]).<br>
Compile the .c file as part of your project, and include the header wherever
needed.

## Compatibility

* Requires Windows Vista or higher.
* Can be compiled with recent versions of MSVC, Clang, and GCC.

## API

### General remarks

* The epoll port is a `HANDLE`, not a file descriptor.
* All functions set both `errno` and `GetLastError()` on failure.
* For more extensive documentation, see the [epoll(7) man page][man epoll],
  and the per-function man pages that are linked below.

### epoll_create/epoll_create1

```c
HANDLE epoll_create(int size);
HANDLE epoll_create1(int flags);
```

* Create a new epoll instance (port).
* `size` is ignored but most be greater than zero.
* `flags` must be zero as there are no supported flags.
* Returns `NULL` on failure.
* [Linux man page][man epoll_create]

### epoll_close

```c
int epoll_close(HANDLE ephnd);
```

* Close an epoll port.
* Do not attempt to close the epoll port with `close()`,
  `CloseHandle()` or `closesocket()`.

### epoll_ctl

```c
int epoll_ctl(HANDLE ephnd,
              int op,
              SOCKET sock,
              struct epoll_event* event);
```

* Control which socket events are monitored by an epoll port.
* `ephnd` must be a HANDLE created by
  [`epoll_create()`](#epoll_createepoll_create1) or
  [`epoll_create1()`](#epoll_createepoll_create1).
* `op` must be one of `EPOLL_CTL_ADD`, `EPOLL_CTL_MOD`, `EPOLL_CTL_DEL`.
* `sock` must be a valid socket created by [`socket()`][msdn socket],
  [`WSASocket()`][msdn wsasocket], or [`accept()`][msdn accept].
* `event` should be a pointer to a [`struct epoll_event`](#struct-epoll_event).<br>
  If `op` is `EPOLL_CTL_DEL` then the `event` parameter is ignored, and it
  may be `NULL`.
* Returns 0 on success, -1 on failure.
* It is recommended to always explicitly remove a socket from its epoll
  set using `EPOLL_CTL_DEL` *before* closing it.<br>
  As on Linux, closed sockets are automatically removed from the epoll set, but
  wepoll may not be able to detect that a socket was closed until the next call
  to [`epoll_wait()`](#epoll_wait).
* [Linux man page][man epoll_ctl]

### epoll_wait

```c
int epoll_wait(HANDLE ephnd,
               struct epoll_event* events,
               int maxevents,
               int timeout);
```

* Receive socket events from an epoll port.
* `events` should point to a caller-allocated array of
  [`epoll_event`](#struct-epoll_event) structs, which will receive the
  reported events.
* `maxevents` is the maximum number of events that will be written to the
  `events` array, and must be greater than zero.
* `timeout` specifies whether to block when no events are immediately available.
  - `<0` block indefinitely
  - `0`  report any events that are already waiting, but don't block
  - `≥1` block for at most N milliseconds
* Return value:
  - `-1` an error occurred
  - `0`  timed out without any events to report
  - `≥1` the number of events stored in the `events` buffer
* [Linux man page][man epoll_wait]

### struct epoll_event

```c
typedef union epoll_data {
  void* ptr;
  int fd;
  uint32_t u32;
  uint64_t u64;
  SOCKET sock;        /* Windows specific */
  HANDLE hnd;         /* Windows specific */
} epoll_data_t;
```

```c
struct epoll_event {
  uint32_t events;    /* Epoll events and flags */
  epoll_data_t data;  /* User data variable */
};
```

* The `events` field is a bit mask containing the events being
  monitored/reported, and optional flags.<br>
  Flags are accepted by [`epoll_ctl()`](#epoll_ctl), but they are not reported
  back by [`epoll_wait()`](#epoll_wait).
* The `data` field can be used to associate application-specific information
  with a socket; its value will be returned unmodified by
  [`epoll_wait()`](#epoll_wait).
* [Linux man page][man epoll_ctl]

| Event         | Description                                                          |
|---------------|----------------------------------------------------------------------|
| `EPOLLIN`     | incoming data available, or incoming connection ready to be accepted |
| `EPOLLOUT`    | ready to send data, or outgoing connection successfully established  |
| `EPOLLRDHUP`  | remote peer initiated graceful socket shutdown                       |
| `EPOLLPRI`    | out-of-band data available for reading                               |
| `EPOLLERR`    | socket error<sup>1</sup>                                             |
| `EPOLLHUP`    | socket hang-up<sup>1</sup>                                           |
| `EPOLLRDNORM` | same as `EPOLLIN`                                                    |
| `EPOLLRDBAND` | same as `EPOLLPRI`                                                   |
| `EPOLLWRNORM` | same as `EPOLLOUT`                                                   |
| `EPOLLWRBAND` | same as `EPOLLOUT`                                                   |
| `EPOLLMSG`    | never reported                                                       |

| Flag             | Description               |
|------------------|---------------------------|
| `EPOLLONESHOT`   | report event(s) only once |
| `EPOLLET`        | not supported by wepoll   |
| `EPOLLEXCLUSIVE` | not supported by wepoll   |
| `EPOLLWAKEUP`    | not supported by wepoll   |

<sup>1</sup>: the `EPOLLERR` and `EPOLLHUP` events may always be reported by
[`epoll_wait()`](#epoll_wait), regardless of the event mask that was passed to
[`epoll_ctl()`](#epoll_ctl).


[ci status badge]:  https://ci.appveyor.com/api/projects/status/github/piscisaureus/wepoll?branch=master&svg=true
[ci status link]:   https://ci.appveyor.com/project/piscisaureus/wepoll/branch/master
[dist]:             https://github.com/piscisaureus/wepoll/tree/dist
[man epoll]:        http://man7.org/linux/man-pages/man7/epoll.7.html
[man epoll_create]: http://man7.org/linux/man-pages/man2/epoll_create.2.html
[man epoll_ctl]:    http://man7.org/linux/man-pages/man2/epoll_ctl.2.html
[man epoll_wait]:   http://man7.org/linux/man-pages/man2/epoll_wait.2.html
[msdn accept]:      https://msdn.microsoft.com/en-us/library/windows/desktop/ms737526(v=vs.85).aspx
[msdn socket]:      https://msdn.microsoft.com/en-us/library/windows/desktop/ms740506(v=vs.85).aspx
[msdn wsasocket]:   https://msdn.microsoft.com/en-us/library/windows/desktop/ms742212(v=vs.85).aspx
[select scale]:     https://daniel.haxx.se/docs/poll-vs-select.html
[wsapoll broken]:   https://daniel.haxx.se/blog/2012/10/10/wsapoll-is-broken/
[wepoll.c]:         https://github.com/piscisaureus/wepoll/blob/dist/wepoll.c
[wepoll.h]:         https://github.com/piscisaureus/wepoll/blob/dist/wepoll.h
