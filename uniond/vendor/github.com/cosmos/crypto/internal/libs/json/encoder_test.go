package json_test

import (
	"testing"
	"time"

	"github.com/cosmos/crypto/internal/libs/json"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestMarshal(t *testing.T) {
	s := "string"
	sPtr := &s
	i64 := int64(64)
	ti := time.Date(2020, 6, 2, 18, 5, 13, 4346374, time.FixedZone("UTC+2", 2*60*60))
	car := &Car{Wheels: 4}
	boat := Boat{Sail: true}

	testcases := map[string]struct {
		value  any
		output string
	}{
		"nil":             {nil, `null`},
		"string":          {"foo", `"foo"`},
		"float32":         {float32(3.14), `3.14`},
		"float32 neg":     {float32(-3.14), `-3.14`},
		"float64":         {float64(3.14), `3.14`},
		"float64 neg":     {float64(-3.14), `-3.14`},
		"int32":           {int32(32), `32`},
		"int64":           {int64(64), `"64"`},
		"int64 neg":       {int64(-64), `"-64"`},
		"int64 ptr":       {&i64, `"64"`},
		"uint64":          {uint64(64), `"64"`},
		"time":            {ti, `"2020-06-02T16:05:13.004346374Z"`},
		"time empty":      {time.Time{}, `"0001-01-01T00:00:00Z"`},
		"time ptr":        {&ti, `"2020-06-02T16:05:13.004346374Z"`},
		"customptr":       {CustomPtr{Value: "x"}, `{"Value":"x"}`}, // same as encoding/json
		"customptr ptr":   {&CustomPtr{Value: "x"}, `"custom"`},
		"customvalue":     {CustomValue{Value: "x"}, `"custom"`},
		"customvalue ptr": {&CustomValue{Value: "x"}, `"custom"`},
		"slice nil":       {[]int(nil), `null`},
		"slice empty":     {[]int{}, `[]`},
		"slice bytes":     {[]byte{1, 2, 3}, `"AQID"`},
		"slice int64":     {[]int64{1, 2, 3}, `["1","2","3"]`},
		"slice int64 ptr": {[]*int64{&i64, nil}, `["64",null]`},
		"array bytes":     {[3]byte{1, 2, 3}, `"AQID"`},
		"array int64":     {[3]int64{1, 2, 3}, `["1","2","3"]`},
		"map nil":         {map[string]int64(nil), `{}`}, // retain Amino compatibility
		"map empty":       {map[string]int64{}, `{}`},
		"map int64":       {map[string]int64{"a": 1, "b": 2, "c": 3}, `{"a":"1","b":"2","c":"3"}`},
		"car":             {car, `{"type":"vehicle/car","value":{"Wheels":4}}`},
		"car value":       {*car, `{"type":"vehicle/car","value":{"Wheels":4}}`},
		"car iface":       {Vehicle(car), `{"type":"vehicle/car","value":{"Wheels":4}}`},
		"car nil":         {(*Car)(nil), `null`},
		"boat":            {boat, `{"type":"vehicle/boat","value":{"Sail":true}}`},
		"boat ptr":        {&boat, `{"type":"vehicle/boat","value":{"Sail":true}}`},
		"boat iface":      {Vehicle(boat), `{"type":"vehicle/boat","value":{"Sail":true}}`},
		"key public":      {PublicKey{1, 2, 3, 4, 5, 6, 7, 8}, `{"type":"key/public","value":"AQIDBAUGBwg="}`},
		"tags": {
			Tags{JSONName: "name", OmitEmpty: "foo", Hidden: "bar", Tags: &Tags{JSONName: "child"}},
			`{"name":"name","OmitEmpty":"foo","tags":{"name":"child"}}`,
		},
		"tags empty": {Tags{}, `{"name":""}`},
		// The encoding of the Car and Boat fields do not have type wrappers, even though they get
		// type wrappers when encoded directly (see "car" and "boat" tests). This is to retain the
		// same behavior as Amino. If the field was a Vehicle interface instead, it would get
		// type wrappers, as seen in the Vehicles field.
		"struct": {
			Struct{
				Bool: true, Float64: 3.14, Int32: 32, Int64: 64, Int64Ptr: &i64,
				String: "foo", StringPtrPtr: &sPtr, Bytes: []byte{1, 2, 3},
				Time: ti, Car: car, Boat: boat, Vehicles: []Vehicle{car, boat},
				Child: &Struct{Bool: false, String: "child"}, private: "private",
			},
			`{
				"Bool":true, "Float64":3.14, "Int32":32, "Int64":"64", "Int64Ptr":"64",
				"String":"foo", "StringPtrPtr": "string", "Bytes":"AQID",
				"Time":"2020-06-02T16:05:13.004346374Z",
				"Car":{"Wheels":4},
				"Boat":{"Sail":true},
				"Vehicles":[
					{"type":"vehicle/car","value":{"Wheels":4}},
					{"type":"vehicle/boat","value":{"Sail":true}}
				],
				"Child":{
					"Bool":false, "Float64":0, "Int32":0, "Int64":"0", "Int64Ptr":null,
					"String":"child", "StringPtrPtr":null, "Bytes":null,
					"Time":"0001-01-01T00:00:00Z",
					"Car":null, "Boat":{"Sail":false}, "Vehicles":null, "Child":null
				}
			}`,
		},
	}
	for name, tc := range testcases {
		tc := tc
		t.Run(name, func(t *testing.T) {
			bz, err := json.Marshal(tc.value)
			require.NoError(t, err)
			assert.JSONEq(t, tc.output, string(bz))
		})
	}
}

func BenchmarkJsonMarshalStruct(b *testing.B) {
	s := "string"
	sPtr := &s
	i64 := int64(64)
	ti := time.Date(2020, 6, 2, 18, 5, 13, 4346374, time.FixedZone("UTC+2", 2*60*60))
	car := &Car{Wheels: 4}
	boat := Boat{Sail: true}
	for i := 0; i < b.N; i++ {
		_, _ = json.Marshal(Struct{
			Bool: true, Float64: 3.14, Int32: 32, Int64: 64, Int64Ptr: &i64,
			String: "foo", StringPtrPtr: &sPtr, Bytes: []byte{1, 2, 3},
			Time: ti, Car: car, Boat: boat, Vehicles: []Vehicle{car, boat},
			Child: &Struct{Bool: false, String: "child"}, private: "private",
		})
	}
}
