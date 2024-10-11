package stdtypes

import (
	"io/ioutil"
	"sync"
	"testing"

	"github.com/cosmos/gogoproto/proto"
)

func TestConcurrentTextMarshal(t *testing.T) {
	// Verify that there are no race conditions when calling
	// TextMarshaler.Marshal on a protobuf message that contains a StdDuration

	std := StdTypes{}
	var wg sync.WaitGroup
	errs := make(chan error)

	tm := proto.TextMarshaler{}

	for i := 0; i < 2; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			if err := tm.Marshal(ioutil.Discard, &std); err != nil {
				errs <- err
			}
		}()
	}

	go func() {
		wg.Wait()
		close(errs)
	}()

	for err := range errs {
		if err != nil {
			t.Fatal(err)
		}
	}
}
