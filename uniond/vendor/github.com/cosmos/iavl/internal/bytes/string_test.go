package common

import (
	"runtime"
	"strconv"
	"testing"
	"time"

	"github.com/stretchr/testify/suite"
)

func TestStringSuite(t *testing.T) {
	suite.Run(t, new(StringSuite))
}

type StringSuite struct{ suite.Suite }

func unsafeConvertStr() []byte {
	return UnsafeStrToBytes("abc")
}

func (s *StringSuite) TestUnsafeStrToBytes() {
	// we convert in other function to trigger GC. We want to check that
	// the underlying array in []bytes is accessible after GC will finish swapping.
	for i := 0; i < 5; i++ {
		b := unsafeConvertStr()
		runtime.GC()
		<-time.NewTimer(2 * time.Millisecond).C
		b2 := append(b, 'd') //nolint:gocritic
		s.Equal("abc", string(b))
		s.Equal("abcd", string(b2))
	}
}

func unsafeConvertBytes() string {
	return UnsafeBytesToStr([]byte("abc"))
}

func (s *StringSuite) TestUnsafeBytesToStr() {
	// we convert in other function to trigger GC. We want to check that
	// the underlying array in []bytes is accessible after GC will finish swapping.
	for i := 0; i < 5; i++ {
		str := unsafeConvertBytes()
		runtime.GC()
		<-time.NewTimer(2 * time.Millisecond).C
		s.Equal("abc", str)
	}
}

func BenchmarkUnsafeStrToBytes(b *testing.B) {
	for i := 0; i < b.N; i++ {
		UnsafeStrToBytes(strconv.Itoa(i))
	}
}
