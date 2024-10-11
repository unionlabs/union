package log_test

import (
	"encoding"
	"fmt"
	"reflect"
	"testing"
	"time"

	"github.com/go-kit/log"
)

func TestValueBinding(t *testing.T) {
	t.Parallel()
	var output []interface{}

	logger := log.Logger(log.LoggerFunc(func(keyvals ...interface{}) error {
		output = keyvals
		return nil
	}))

	start := time.Date(2015, time.April, 25, 0, 0, 0, 0, time.UTC)
	now := start
	mocktime := func() time.Time {
		now = now.Add(time.Second)
		return now
	}

	lc := log.With(logger, "ts", log.Timestamp(mocktime), "caller", log.DefaultCaller)

	lc.Log("foo", "bar")
	timestamp, ok := output[1].(time.Time)
	if !ok {
		t.Fatalf("want time.Time, have %T", output[1])
	}
	if want, have := start.Add(time.Second), timestamp; want != have {
		t.Errorf("output[1]: want %v, have %v", want, have)
	}
	if want, have := "value_test.go:31", fmt.Sprint(output[3]); want != have {
		t.Errorf("output[3]: want %s, have %s", want, have)
	}

	// A second attempt to confirm the bindings are truly dynamic.
	lc.Log("foo", "bar")
	timestamp, ok = output[1].(time.Time)
	if !ok {
		t.Fatalf("want time.Time, have %T", output[1])
	}
	if want, have := start.Add(2*time.Second), timestamp; want != have {
		t.Errorf("output[1]: want %v, have %v", want, have)
	}
	if want, have := "value_test.go:44", fmt.Sprint(output[3]); want != have {
		t.Errorf("output[3]: want %s, have %s", want, have)
	}
}

func TestValueBinding_loggingZeroKeyvals(t *testing.T) {
	t.Parallel()
	var output []interface{}

	logger := log.Logger(log.LoggerFunc(func(keyvals ...interface{}) error {
		output = keyvals
		return nil
	}))

	start := time.Date(2015, time.April, 25, 0, 0, 0, 0, time.UTC)
	now := start
	mocktime := func() time.Time {
		now = now.Add(time.Second)
		return now
	}

	logger = log.With(logger, "ts", log.Timestamp(mocktime))

	logger.Log()
	timestamp, ok := output[1].(time.Time)
	if !ok {
		t.Fatalf("want time.Time, have %T", output[1])
	}
	if want, have := start.Add(time.Second), timestamp; want != have {
		t.Errorf("output[1]: want %v, have %v", want, have)
	}

	// A second attempt to confirm the bindings are truly dynamic.
	logger.Log()
	timestamp, ok = output[1].(time.Time)
	if !ok {
		t.Fatalf("want time.Time, have %T", output[1])
	}
	if want, have := start.Add(2*time.Second), timestamp; want != have {
		t.Errorf("output[1]: want %v, have %v", want, have)
	}
}

func TestTimestampFormat(t *testing.T) {
	t.Parallel()

	start := time.Date(2015, time.April, 25, 0, 0, 0, 0, time.UTC)
	now := start
	mocktime := func() time.Time {
		now = now.Add(time.Second)
		return now
	}

	tv := log.TimestampFormat(mocktime, time.RFC822)

	if want, have := now.Add(time.Second).Format(time.RFC822), fmt.Sprint(tv()); want != have {
		t.Errorf("wrong time format: want %v, have %v", want, have)
	}

	if want, have := now.Add(2*time.Second).Format(time.RFC822), fmt.Sprint(tv()); want != have {
		t.Errorf("wrong time format: want %v, have %v", want, have)
	}

	mustMarshal := func(v interface{}) []byte {
		b, err := v.(encoding.TextMarshaler).MarshalText()
		if err != nil {
			t.Fatal("error marshaling text:", err)
		}
		return b
	}

	if want, have := now.Add(3*time.Second).AppendFormat(nil, time.RFC822), mustMarshal(tv()); !reflect.DeepEqual(want, have) {
		t.Errorf("wrong time format: want %s, have %s", want, have)
	}

	if want, have := now.Add(4*time.Second).AppendFormat(nil, time.RFC822), mustMarshal(tv()); !reflect.DeepEqual(want, have) {
		t.Errorf("wrong time format: want %s, have %s", want, have)
	}
}

func BenchmarkValueBindingTimestamp(b *testing.B) {
	logger := log.NewNopLogger()
	lc := log.With(logger, "ts", log.DefaultTimestamp)
	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		lc.Log("k", "v")
	}
}

func BenchmarkValueBindingCaller(b *testing.B) {
	logger := log.NewNopLogger()
	lc := log.With(logger, "caller", log.DefaultCaller)
	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		lc.Log("k", "v")
	}
}
