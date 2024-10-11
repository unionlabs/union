package zstd

/*
From https://github.com/dustin/randbo
All credits for the code below goes there :) (There wasn't a license so I'm distributing as is)
*/

import (
	"io"
	"math/rand"
	"time"
)

// randbytes creates a stream of non-crypto quality random bytes
type randbytes struct {
	rand.Source
}

// NewRandBytes creates a new random reader with a time source.
func NewRandBytes() io.Reader {
	return NewRandBytesFrom(rand.NewSource(time.Now().UnixNano()))
}

// NewRandBytesFrom creates a new reader from your own rand.Source
func NewRandBytesFrom(src rand.Source) io.Reader {
	return &randbytes{src}
}

// Read satisfies io.Reader
func (r *randbytes) Read(p []byte) (n int, err error) {
	todo := len(p)
	offset := 0
	for {
		val := int64(r.Int63())
		for i := 0; i < 8; i++ {
			p[offset] = byte(val)
			todo--
			if todo == 0 {
				return len(p), nil
			}
			offset++
			val >>= 8
		}
	}
}
