package grpc

import (
	"context"
	"fmt"
	"strings"
	"testing"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/errors/grpc/status"
	"github.com/cockroachdb/errors/testutils"
	"google.golang.org/grpc/codes"
)

func TestGrpc(t *testing.T) {

	tt := testutils.T{T: t}

	// A successful call should return the message, a nil error, and the status code should evaluate to codes.OK
	resp, err := Client.Echo(context.Background(), &EchoRequest{Text: "hello"})
	tt.Assert(err == nil)
	tt.Assert(resp.Reply == "echoing: hello")
	tt.Assert(status.Code(err) == codes.OK)

	// A sentinel error should be detectable across grpc boundaries
	// A failed call that does not have a status specified should evaluate to codes.Unknown
	_, err = Client.Echo(context.Background(), &EchoRequest{Text: "noecho"})
	tt.Assert(err != nil)
	tt.Assert(errors.Is(err, ErrCantEcho))
	tt.Assert(status.Code(err) == codes.Unknown)

	// A wrapped error should be unwrappable after crossing grpc boundaries
	_, err = Client.Echo(context.Background(), &EchoRequest{Text: "really_long_message"})
	tt.Assert(err != nil)
	tt.Assert(err.Error() == "really_long_message is too long: text is too long")
	tt.Assert(errors.Is(err, ErrTooLong))
	tt.Assert(errors.UnwrapAll(err).Error() == "text is too long")

	// A failed call with a specified status should evaluate correctly after crossing a grpc boundary
	_, err = Client.Echo(context.Background(), &EchoRequest{Text: "reverse"})
	tt.Assert(err != nil)
	tt.Assert(err.Error() == "reverse is not implemented")
	tt.Assert(status.Code(err) == codes.Unimplemented)

	// Sentinel error and status code in the same response
	// Printing the error out with detail should include the grpc status
	_, err = Client.Echo(context.Background(), &EchoRequest{Text: "internal"})
	tt.Assert(err != nil)
	tt.Assert(err.Error() == "there was a problem: internal error!")
	tt.Assert(status.Code(err) == codes.Internal)
	tt.Assert(errors.Is(err, ErrInternal))
	spv := fmt.Sprintf("%+v", err)
	t.Logf("spv:\n%s", spv)
	tt.Assert(strings.Contains(spv, "gRPC code: Internal"))
}
