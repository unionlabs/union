go-buffer-pool
==================

[![](https://img.shields.io/badge/made%20by-Protocol%20Labs-blue.svg?style=flat-square)](https://protocol.ai)
[![](https://img.shields.io/badge/project-libp2p-yellow.svg?style=flat-square)](https://libp2p.io/)
[![](https://img.shields.io/badge/freenode-%23libp2p-yellow.svg?style=flat-square)](https://webchat.freenode.net/?channels=%23libp2p)
[![codecov](https://codecov.io/gh/libp2p/go-buffer-pool/branch/master/graph/badge.svg)](https://codecov.io/gh/libp2p/go-buffer-pool)
[![Travis CI](https://travis-ci.org/libp2p/go-buffer-pool.svg?branch=master)](https://travis-ci.org/libp2p/go-buffer-pool)
[![Discourse posts](https://img.shields.io/discourse/https/discuss.libp2p.io/posts.svg)](https://discuss.libp2p.io)

> A variable size buffer pool for go.

## Table of Contents

- [Use Case](#use-case)
    - [Advantages over GC](#advantages-over-gc)
    - [Disadvantages over GC:](#disadvantages-over-gc)
- [Contribute](#contribute)
- [License](#license)

## Use Case

Use this when you need to repeatedly allocate and free a bunch of temporary buffers of approximately the same size.

### Advantages over GC

* Reduces Memory Usage:
  * We don't have to wait for a GC to run before we can reuse memory. This is essential if you're repeatedly allocating large short-lived buffers.

* Reduces CPU usage:
  * It takes some load off of the GC (due to buffer reuse).
  * We don't have to zero buffers (fewer wasteful memory writes).

### Disadvantages over GC:

* Can leak memory contents. Unlike the go GC, we *don't* zero memory.
* All buffers have a capacity of a power of 2. This is fine if you either (a) actually need buffers with this size or (b) expect these buffers to be temporary.
* Requires that buffers be returned explicitly. This can lead to race conditions and memory corruption if the buffer is released while it's still in use.

## Contribute

PRs are welcome!

Small note: If editing the Readme, please conform to the [standard-readme](https://github.com/RichardLitt/standard-readme) specification.

## License

MIT © Protocol Labs
BSD © The Go Authors

---

The last gx published version of this module was: 0.1.3: QmQDvJoB6aJWN3sjr3xsgXqKCXf4jU5zdMXpDMsBkYVNqa
