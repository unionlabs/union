package tests

type Metadata struct {
	Version    uint8
	CodeHash   []byte `ssz-size:"32"`
	CodeLength uint16
}

type Chunk struct {
	FIO  uint8
	Code []byte `ssz-size:"32"` // Last chunk is right-padded with zeros
}

type CodeTrieSmall struct {
	Metadata *Metadata
	Chunks   []*Chunk `ssz-max:"4"`
}

type CodeTrieBig struct {
	Metadata *Metadata
	Chunks   []*Chunk `ssz-max:"1024"`
}
