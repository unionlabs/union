# timepb

`timepb` is a Go package that provides functions to do time operations with 
[protobuf timestamp](https://developers.google.com/protocol-buffers/docs/reference/google.protobuf#timestamp) 
and [protobuf duration](https://developers.google.com/protocol-buffers/docs/reference/google.protobuf#duration) 
structures.

### Example

``` go
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
```

