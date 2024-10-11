package kv

type Pair struct {
	Key   []byte
	Value []byte
}

type Pairs struct {
	Pairs []Pair
}
