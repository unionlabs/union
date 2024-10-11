package runtime_test

import (
	"context"
	"errors"
	"net/http"
	"net/http/httptest"
	"strconv"
	"strings"
	"testing"

	"github.com/grpc-ecosystem/grpc-gateway/v2/runtime"
	"google.golang.org/genproto/googleapis/rpc/errdetails"
	statuspb "google.golang.org/genproto/googleapis/rpc/status"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

func TestDefaultHTTPError(t *testing.T) {
	ctx := context.Background()

	statusWithDetails, _ := status.New(codes.FailedPrecondition, "failed precondition").WithDetails(
		&errdetails.PreconditionFailure{},
	)

	for i, spec := range []struct {
		err         error
		status      int
		msg         string
		marshaler   runtime.Marshaler
		contentType string
		details     string
	}{
		{
			err:         errors.New("example error"),
			status:      http.StatusInternalServerError,
			marshaler:   &runtime.JSONPb{},
			contentType: "application/json",
			msg:         "example error",
		},
		{
			err:         status.Error(codes.NotFound, "no such resource"),
			status:      http.StatusNotFound,
			marshaler:   &runtime.JSONPb{},
			contentType: "application/json",
			msg:         "no such resource",
		},
		{
			err:         statusWithDetails.Err(),
			status:      http.StatusBadRequest,
			marshaler:   &runtime.JSONPb{},
			contentType: "application/json",
			msg:         "failed precondition",
			details:     "type.googleapis.com/google.rpc.PreconditionFailure",
		},
		{
			err:         errors.New("example error"),
			status:      http.StatusInternalServerError,
			marshaler:   &CustomMarshaler{&runtime.JSONPb{}},
			contentType: "Custom-Content-Type",
			msg:         "example error",
		},
		{
			err: &runtime.HTTPStatusError{
				HTTPStatus: http.StatusMethodNotAllowed,
				Err:        status.Error(codes.Unimplemented, http.StatusText(http.StatusMethodNotAllowed)),
			},
			status:      http.StatusMethodNotAllowed,
			marshaler:   &runtime.JSONPb{},
			contentType: "application/json",
			msg:         "Method Not Allowed",
		},
	} {
		t.Run(strconv.Itoa(i), func(t *testing.T) {
			w := httptest.NewRecorder()
			req, _ := http.NewRequest("", "", nil) // Pass in an empty request to match the signature
			mux := runtime.NewServeMux()
			marshaler := &runtime.JSONPb{}
			runtime.HTTPError(ctx, mux, marshaler, w, req, spec.err)

			if got, want := w.Header().Get("Content-Type"), "application/json"; got != want {
				t.Errorf(`w.Header().Get("Content-Type") = %q; want %q; on spec.err=%v`, got, want, spec.err)
			}
			if got, want := w.Code, spec.status; got != want {
				t.Errorf("w.Code = %d; want %d", got, want)
			}

			var st statuspb.Status
			if err := marshaler.Unmarshal(w.Body.Bytes(), &st); err != nil {
				t.Errorf("marshaler.Unmarshal(%q, &body) failed with %v; want success", w.Body.Bytes(), err)
				return
			}

			if got, want := st.Message, spec.msg; !strings.Contains(got, want) {
				t.Errorf(`st.Message = %q; want %q; on spec.err=%v`, got, want, spec.err)
			}

			if spec.details != "" {
				if len(st.Details) != 1 {
					t.Errorf(`len(st.Details) = %v; want 1`, len(st.Details))
					return
				}
				if st.Details[0].TypeUrl != spec.details {
					t.Errorf(`details.type_url = %s; want %s`, st.Details[0].TypeUrl, spec.details)
				}
			}
		})
	}
}
