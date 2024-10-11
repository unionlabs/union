package json_test

import (
	"reflect"
	"testing"
	"time"

	"github.com/cosmos/crypto/internal/libs/json"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestUnmarshal(t *testing.T) {
	i64Nil := (*int64)(nil)
	str := "string"
	strPtr := &str
	structNil := (*Struct)(nil)
	i32 := int32(32)
	i64 := int64(64)

	testcases := map[string]struct {
		json  string
		value any
		err   bool
	}{
		"bool true":           {"true", true, false},
		"bool false":          {"false", false, false},
		"float32":             {"3.14", float32(3.14), false},
		"float64":             {"3.14", float64(3.14), false},
		"int32":               {`32`, int32(32), false},
		"int32 string":        {`"32"`, int32(32), true},
		"int32 ptr":           {`32`, &i32, false},
		"int64":               {`"64"`, int64(64), false},
		"int64 noend":         {`"64`, int64(64), true},
		"int64 number":        {`64`, int64(64), true},
		"int64 ptr":           {`"64"`, &i64, false},
		"int64 ptr nil":       {`null`, i64Nil, false},
		"string":              {`"foo"`, "foo", false},
		"string noend":        {`"foo`, "foo", true},
		"string ptr":          {`"string"`, &str, false},
		"slice byte":          {`"AQID"`, []byte{1, 2, 3}, false},
		"slice bytes":         {`["AQID"]`, [][]byte{{1, 2, 3}}, false},
		"slice int32":         {`[1,2,3]`, []int32{1, 2, 3}, false},
		"slice int64":         {`["1","2","3"]`, []int64{1, 2, 3}, false},
		"slice int64 number":  {`[1,2,3]`, []int64{1, 2, 3}, true},
		"slice int64 ptr":     {`["64"]`, []*int64{&i64}, false},
		"slice int64 empty":   {`[]`, []int64(nil), false},
		"slice int64 null":    {`null`, []int64(nil), false},
		"array byte":          {`"AQID"`, [3]byte{1, 2, 3}, false},
		"array byte large":    {`"AQID"`, [4]byte{1, 2, 3, 4}, true},
		"array byte small":    {`"AQID"`, [2]byte{1, 2}, true},
		"array int32":         {`[1,2,3]`, [3]int32{1, 2, 3}, false},
		"array int64":         {`["1","2","3"]`, [3]int64{1, 2, 3}, false},
		"array int64 number":  {`[1,2,3]`, [3]int64{1, 2, 3}, true},
		"array int64 large":   {`["1","2","3"]`, [4]int64{1, 2, 3, 4}, true},
		"array int64 small":   {`["1","2","3"]`, [2]int64{1, 2}, true},
		"map bytes":           {`{"b":"AQID"}`, map[string][]byte{"b": {1, 2, 3}}, false},
		"map int32":           {`{"a":1,"b":2}`, map[string]int32{"a": 1, "b": 2}, false},
		"map int64":           {`{"a":"1","b":"2"}`, map[string]int64{"a": 1, "b": 2}, false},
		"map int64 empty":     {`{}`, map[string]int64{}, false},
		"map int64 null":      {`null`, map[string]int64(nil), false},
		"map int key":         {`{}`, map[int]int{}, true},
		"time":                {`"2020-06-03T17:35:30Z"`, time.Date(2020, 6, 3, 17, 35, 30, 0, time.UTC), false},
		"time non-utc":        {`"2020-06-03T17:35:30+02:00"`, time.Time{}, true},
		"time nozone":         {`"2020-06-03T17:35:30"`, time.Time{}, true},
		"car":                 {`{"type":"vehicle/car","value":{"Wheels":4}}`, Car{Wheels: 4}, false},
		"car ptr":             {`{"type":"vehicle/car","value":{"Wheels":4}}`, &Car{Wheels: 4}, false},
		"car iface":           {`{"type":"vehicle/car","value":{"Wheels":4}}`, Vehicle(&Car{Wheels: 4}), false},
		"boat":                {`{"type":"vehicle/boat","value":{"Sail":true}}`, Boat{Sail: true}, false},
		"boat ptr":            {`{"type":"vehicle/boat","value":{"Sail":true}}`, &Boat{Sail: true}, false},
		"boat iface":          {`{"type":"vehicle/boat","value":{"Sail":true}}`, Vehicle(Boat{Sail: true}), false},
		"boat into car":       {`{"type":"vehicle/boat","value":{"Sail":true}}`, Car{}, true},
		"boat into car iface": {`{"type":"vehicle/boat","value":{"Sail":true}}`, Vehicle(&Car{}), true},
		"shoes":               {`{"type":"vehicle/shoes","value":{"Soles":"rubber"}}`, Car{}, true},
		"shoes ptr":           {`{"type":"vehicle/shoes","value":{"Soles":"rubber"}}`, &Car{}, true},
		"shoes iface":         {`{"type":"vehicle/shoes","value":{"Soles":"rubbes"}}`, Vehicle(&Car{}), true},
		"key public":          {`{"type":"key/public","value":"AQIDBAUGBwg="}`, PublicKey{1, 2, 3, 4, 5, 6, 7, 8}, false},
		"key wrong":           {`{"type":"key/public","value":"AQIDBAUGBwg="}`, PrivateKey{1, 2, 3, 4, 5, 6, 7, 8}, true},
		"key into car":        {`{"type":"key/public","value":"AQIDBAUGBwg="}`, Vehicle(&Car{}), true},
		"tags": {
			`{"name":"name","OmitEmpty":"foo","Hidden":"bar","tags":{"name":"child"}}`,
			Tags{JSONName: "name", OmitEmpty: "foo", Tags: &Tags{JSONName: "child"}},
			false,
		},
		"tags ptr": {
			`{"name":"name","OmitEmpty":"foo","tags":null}`,
			&Tags{JSONName: "name", OmitEmpty: "foo"},
			false,
		},
		"tags real name": {`{"JSONName":"name"}`, Tags{}, false},
		"struct": {
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
				},
				"private": "foo", "unknown": "bar"
			}`,
			Struct{
				Bool: true, Float64: 3.14, Int32: 32, Int64: 64, Int64Ptr: &i64,
				String: "foo", StringPtrPtr: &strPtr, Bytes: []byte{1, 2, 3},
				Time: time.Date(2020, 6, 2, 16, 5, 13, 4346374, time.UTC),
				Car:  &Car{Wheels: 4}, Boat: Boat{Sail: true}, Vehicles: []Vehicle{
					Vehicle(&Car{Wheels: 4}),
					Vehicle(Boat{Sail: true}),
				},
				Child: &Struct{Bool: false, String: "child"},
			},
			false,
		},
		"struct key into vehicle": {`{"Vehicles":[
			{"type":"vehicle/car","value":{"Wheels":4}},
			{"type":"key/public","value":"MTIzNDU2Nzg="}
		]}`, Struct{}, true},
		"struct ptr null":  {`null`, structNil, false},
		"custom value":     {`{"Value":"foo"}`, CustomValue{}, false},
		"custom ptr":       {`"foo"`, &CustomPtr{Value: "custom"}, false},
		"custom ptr value": {`"foo"`, CustomPtr{Value: "custom"}, false},
		"invalid type":     {`"foo"`, Struct{}, true},
	}
	for name, tc := range testcases {
		tc := tc
		t.Run(name, func(t *testing.T) {
			// Create a target variable as a pointer to the zero value of the tc.value type,
			// and wrap it in an empty interface. Decode into that interface.
			target := reflect.New(reflect.TypeOf(tc.value)).Interface()
			err := json.Unmarshal([]byte(tc.json), target)
			if tc.err {
				require.Error(t, err)
				return
			}
			require.NoError(t, err)

			// Unwrap the target pointer and get the value behind the interface.
			actual := reflect.ValueOf(target).Elem().Interface()
			assert.Equal(t, tc.value, actual)
		})
	}
}
