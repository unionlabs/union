package timepb

import (
	"fmt"

	durpb "google.golang.org/protobuf/types/known/durationpb"
	tspb "google.golang.org/protobuf/types/known/timestamppb"
)

func ExampleAdd() {
	t1 := &tspb.Timestamp{Seconds: 10, Nanos: 1}
	d := &durpb.Duration{Seconds: 1, Nanos: 1e9 - 1}
	t2 := Add(t1, d)

	fmt.Println(Compare(&tspb.Timestamp{Seconds: 12, Nanos: 0}, t2) == 0)
	fmt.Println(Compare(&tspb.Timestamp{Seconds: 10, Nanos: 1}, t1) == 0)
	fmt.Println(Compare(t1, t2))
	// Output:
	// true
	// true
	// -1
}
