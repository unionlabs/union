//go:build windows

package stringbuffer

import "testing"

func Test_BufferCapacity(t *testing.T) {
	b := NewWString()

	c := b.Cap()
	if c < MinWStringCap {
		t.Fatalf("expected capacity >= %d, got %d", MinWStringCap, c)
	}

	if l := len(b.b); l != int(c) {
		t.Fatalf("buffer length (%d) and capacity (%d) mismatch", l, c)
	}

	n := uint32(1.5 * MinWStringCap)
	nn := b.ResizeTo(n)
	if len(b.b) != int(nn) {
		t.Fatalf("resized buffer should be %d, was %d", nn, len(b.b))
	}
	if n > nn {
		t.Fatalf("resized to a value smaller than requested")
	}
}

func Test_BufferFree(t *testing.T) {
	// make sure free-ing doesn't set pooled buffer to nil as well
	for i := 0; i < 256; i++ {
		// try allocating and freeing repeatedly since pool does not guarantee item reuse
		b := NewWString()
		b.Free()
		if b.b != nil {
			t.Fatalf("freed buffer is not nil")
		}

		b = NewWString()
		c := b.Cap()
		if c < MinWStringCap {
			t.Fatalf("expected capacity >= %d, got %d", MinWStringCap, c)
		}
	}
}
