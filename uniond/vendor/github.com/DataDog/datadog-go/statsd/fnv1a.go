package statsd

const (
	// FNV-1a
	offset32 = uint32(2166136261)
	prime32  = uint32(16777619)

	// init32 is what 32 bits hash values should be initialized with.
	init32 = offset32
)

// HashString32 returns the hash of s.
func hashString32(s string) uint32 {
	return addString32(init32, s)
}

// AddString32 adds the hash of s to the precomputed hash value h.
func addString32(h uint32, s string) uint32 {
	i := 0
	n := (len(s) / 8) * 8

	for i != n {
		h = (h ^ uint32(s[i])) * prime32
		h = (h ^ uint32(s[i+1])) * prime32
		h = (h ^ uint32(s[i+2])) * prime32
		h = (h ^ uint32(s[i+3])) * prime32
		h = (h ^ uint32(s[i+4])) * prime32
		h = (h ^ uint32(s[i+5])) * prime32
		h = (h ^ uint32(s[i+6])) * prime32
		h = (h ^ uint32(s[i+7])) * prime32
		i += 8
	}

	for _, c := range s[i:] {
		h = (h ^ uint32(c)) * prime32
	}

	return h
}
