package internal

func IntSliceToUint64Slice(in []int) []uint64 {
	res := make([]uint64, len(in))
	for i := range in {
		res[i] = uint64(in[i])
	}
	return res
}
