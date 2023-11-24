package testutil

import (
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/testing/protocmp"
)

// RequireProtoDeepEqual fails the test t if p1 and p2 are not equivalent protobuf messages.
// Where p1 and p2 are proto.Message or slices of proto.Message.
func RequireProtoDeepEqual(t *testing.T, p1, p2 interface{}) {
	t.Helper()
	require.Empty(t, cmp.Diff(p1, p2, protocmp.Transform()))
}
