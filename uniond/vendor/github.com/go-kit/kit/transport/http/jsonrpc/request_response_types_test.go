package jsonrpc_test

import (
	"encoding/json"
	"fmt"
	"testing"

	"github.com/go-kit/kit/transport/http/jsonrpc"
)

func TestCanUnMarshalID(t *testing.T) {
	cases := []struct {
		JSON     string
		expType  string
		expValue interface{}
	}{
		{`12345`, "int", 12345},
		{`12345.6`, "float", 12345.6},
		{`"stringaling"`, "string", "stringaling"},
	}

	for _, c := range cases {
		r := jsonrpc.Request{}
		JSON := fmt.Sprintf(`{"id":%s}`, c.JSON)

		var foo interface{}
		_ = json.Unmarshal([]byte(JSON), &foo)

		err := json.Unmarshal([]byte(JSON), &r)
		if err != nil {
			t.Fatalf("Unexpected error unmarshaling JSON into request: %s\n", err)
		}
		id := r.ID

		switch c.expType {
		case "int":
			want := c.expValue.(int)
			got, err := id.Int()
			if err != nil {
				t.Fatal(err)
			}
			if got != want {
				t.Fatalf("'%s' Int(): want %d, got %d.", c.JSON, want, got)
			}

			// Allow an int ID to be interpreted as a float.
			wantf := float32(c.expValue.(int))
			gotf, err := id.Float32()
			if err != nil {
				t.Fatal(err)
			}
			if gotf != wantf {
				t.Fatalf("'%s' Int value as Float32(): want %f, got %f.", c.JSON, wantf, gotf)
			}

			_, err = id.String()
			if err == nil {
				t.Fatal("Expected String() to error for int value. Didn't.")
			}
		case "string":
			want := c.expValue.(string)
			got, err := id.String()
			if err != nil {
				t.Fatal(err)
			}
			if got != want {
				t.Fatalf("'%s' String(): want %s, got %s.", c.JSON, want, got)
			}

			_, err = id.Int()
			if err == nil {
				t.Fatal("Expected Int() to error for string value. Didn't.")
			}
			_, err = id.Float32()
			if err == nil {
				t.Fatal("Expected Float32() to error for string value. Didn't.")
			}
		case "float32":
			want := c.expValue.(float32)
			got, err := id.Float32()
			if err != nil {
				t.Fatal(err)
			}
			if got != want {
				t.Fatalf("'%s' Float32(): want %f, got %f.", c.JSON, want, got)
			}

			_, err = id.String()
			if err == nil {
				t.Fatal("Expected String() to error for float value. Didn't.")
			}
			_, err = id.Int()
			if err == nil {
				t.Fatal("Expected Int() to error for float value. Didn't.")
			}
		}
	}
}

func TestCanUnmarshalNullID(t *testing.T) {
	r := jsonrpc.Request{}
	JSON := `{"id":null}`
	err := json.Unmarshal([]byte(JSON), &r)
	if err != nil {
		t.Fatalf("Unexpected error unmarshaling JSON into request: %s\n", err)
	}

	if r.ID != nil {
		t.Fatalf("Expected ID to be nil, got %+v.\n", r.ID)
	}
}

func TestCanMarshalID(t *testing.T) {
	cases := []struct {
		JSON     string
		expType  string
		expValue interface{}
	}{
		{`12345`, "int", 12345},
		{`12345.6`, "float", 12345.6},
		{`"stringaling"`, "string", "stringaling"},
		{`null`, "null", nil},
	}

	for _, c := range cases {
		req := jsonrpc.Request{}
		JSON := fmt.Sprintf(`{"jsonrpc":"2.0","id":%s}`, c.JSON)
		json.Unmarshal([]byte(JSON), &req)
		resp := jsonrpc.Response{ID: req.ID, JSONRPC: req.JSONRPC}

		want := JSON
		bol, _ := json.Marshal(resp)
		got := string(bol)
		if got != want {
			t.Fatalf("'%s': want %s, got %s.", c.expType, want, got)
		}
	}
}
