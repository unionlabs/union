package unknownproto

import (
	"fmt"
	"testing"

	"google.golang.org/protobuf/encoding/protowire"
)

func TestWireTypeToString(t *testing.T) {
	tests := []struct {
		typ  protowire.Type
		want string
	}{
		{typ: 0, want: "varint"},
		{typ: 1, want: "fixed64"},
		{typ: 2, want: "bytes"},
		{typ: 3, want: "start_group"},
		{typ: 4, want: "end_group"},
		{typ: 5, want: "fixed32"},
		{typ: 95, want: "unknown type: 95"},
	}

	for _, tt := range tests {
		t.Run(fmt.Sprintf("wireType=%d", tt.typ), func(t *testing.T) {
			if g, w := wireTypeToString(tt.typ), tt.want; g != w {
				t.Fatalf("Mismatch:\nGot:  %q\nWant: %q\n", g, w)
			}
		})
	}
}
