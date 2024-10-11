package query_test

import (
	"testing"

	"github.com/cometbft/cometbft/libs/pubsub/query"
)

const testQuery = `tm.events.type='NewBlock' AND abci.account.name='Igor'`

var testEvents = map[string][]string{
	"tm.events.index": {
		"25",
	},
	"tm.events.type": {
		"NewBlock",
	},
	"abci.account.name": {
		"Anya", "Igor",
	},
}

func BenchmarkParseCustom(b *testing.B) {
	for i := 0; i < b.N; i++ {
		_, err := query.New(testQuery)
		if err != nil {
			b.Fatal(err)
		}
	}
}

func BenchmarkMatchCustom(b *testing.B) {
	q, err := query.New(testQuery)
	if err != nil {
		b.Fatal(err)
	}
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		ok, err := q.Matches(testEvents)
		if err != nil {
			b.Fatal(err)
		} else if !ok {
			b.Error("no match")
		}
	}
}
