package bip39

import "testing"

var words = []string{
	"wolf afraid artwork blanket carpet cricket wolf afraid artwork blanket carpet cricket",
	"artwork blanket carpet cricket disorder disorder artwork blanket carpet cricket disorder disorder",
	"carpet cricket disorder cricket cricket artwork carpet cricket disorder cricket cricket artwork ",
}

func BenchmarkIsMnemonicValid(b *testing.B) {
	b.ReportAllocs()
	var sharp interface{}
	for i := 0; i < b.N; i++ {
		for _, word := range words {
			ok := IsMnemonicValid(word)
			if !ok {
				b.Fatal("returned false")
			}
			sharp = ok
		}
	}
	if sharp == nil {
		b.Fatal("benchmark was not run")
	}
}
