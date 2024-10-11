package runtime_test

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/grpc-ecosystem/grpc-gateway/runtime"
	"google.golang.org/genproto/googleapis/rpc/errdetails"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

func TestDefaultHTTPError(t *testing.T) {
	ctx := context.Background()

	statusWithDetails, _ := status.New(codes.FailedPrecondition, "failed precondition").WithDetails(
		&errdetails.PreconditionFailure{},
	)

	for _, spec := range []struct {
		err         error
		status      int
		msg         string
		marshaler   runtime.Marshaler
		contentType string
		details     string
	}{
		{
			err:         fmt.Errorf("example error"),
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
			err:         fmt.Errorf("example error"),
			status:      http.StatusInternalServerError,
			marshaler:   &CustomMarshaler{&runtime.JSONPb{}},
			contentType: "Custom-Content-Type",
			msg:         "example error",
		},
	} {
		w := httptest.NewRecorder()
		req, _ := http.NewRequest("", "", nil) // Pass in an empty request to match the signature
		runtime.DefaultHTTPError(ctx, &runtime.ServeMux{}, &runtime.JSONPb{}, w, req, spec.err)

		if got, want := w.Header().Get("Content-Type"), "application/json"; got != want {
			t.Errorf(`w.Header().Get("Content-Type") = %q; want %q; on spec.err=%v`, got, want, spec.err)
		}
		if got, want := w.Code, spec.status; got != want {
			t.Errorf("w.Code = %d; want %d", got, want)
		}

		body := make(map[string]interface{})
		if err := json.Unmarshal(w.Body.Bytes(), &body); err != nil {
			t.Errorf("json.Unmarshal(%q, &body) failed with %v; want success", w.Body.Bytes(), err)
			continue
		}

		if got, want := body["error"].(string), spec.msg; !strings.Contains(got, want) {
			t.Errorf(`body["error"] = %q; want %q; on spec.err=%v`, got, want, spec.err)
		}
		if got, want := body["message"].(string), spec.msg; !strings.Contains(got, want) {
			t.Errorf(`body["message"] = %q; want %q; on spec.err=%v`, got, want, spec.err)
		}

		if spec.details != "" {
			details, ok := body["details"].([]interface{})
			if !ok {
				t.Errorf(`body["details"] = %T; want %T`, body["details"], []interface{}{})
				continue
			}
			if len(details) != 1 {
				t.Errorf(`len(body["details"]) = %v; want 1`, len(details))
				continue
			}
			if details[0].(map[string]interface{})["@type"] != spec.details {
				t.Errorf(`details.@type = %s; want %s`, details[0].(map[string]interface{})["@type"], spec.details)
			}
		}
	}
}
