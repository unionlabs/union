package common

import (
	"io"
	"sync"
)

type Buffer struct {
	Data   []byte
	B1     []byte
	B2     []byte
	B4     []byte
	B8     []byte
	B16    []byte
	offset int
}

func (b *Buffer) Write(w io.Writer, vs ...byte) error {
	if len(b.Data) < b.offset+len(vs) {
		_, err := w.Write(b.Data[:b.offset])
		if err != nil {
			return err
		}
		if len(b.Data) < len(vs) {
			b.Data = append(b.Data, make([]byte, len(vs)-len(b.Data))...)
		}
		b.offset = 0
	}
	for i := range vs {
		b.Data[b.offset+i] = vs[i]
	}
	b.offset += len(vs)
	return nil
}

func (b *Buffer) Flush(w io.Writer) error {
	_, err := w.Write(b.Data[:b.offset])
	return err
}

var bufPool = sync.Pool{
	New: func() interface{} {
		data := make([]byte, 64)
		return &Buffer{
			Data: data,
			B1:   data[:1],
			B2:   data[:2],
			B4:   data[:4],
			B8:   data[:8],
			B16:  data[:16],
		}
	},
}

func GetBuffer() *Buffer {
	buf := bufPool.Get().(*Buffer)
	buf.offset = 0
	return buf
}

func PutBuffer(buf *Buffer) {
	bufPool.Put(buf)
}
