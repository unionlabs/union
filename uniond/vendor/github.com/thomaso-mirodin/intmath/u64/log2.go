package u64

// Log2 returns log base 2 of n. It's the same as index of the highest
// bit set in n. n == 0 returns 0
func Log2(n uint64) uint64 {
	// Using uint instead of uint64 is about 25% faster
	// on x86 systems with the default Go compiler.
	var r, v uint
	if n < 1<<32 {
		v = uint(n)
	} else {
		r = 32
		v = uint(n >> 32)
	}
	if v >= 1<<16 {
		r += 16
		v >>= 16
	}
	if v >= 1<<8 {
		r += 8
		v >>= 8
	}
	if v >= 1<<4 {
		r += 4
		v >>= 4
	}
	if v >= 1<<2 {
		r += 2
		v >>= 2
	}
	r += v >> 1
	return uint64(r)
}
