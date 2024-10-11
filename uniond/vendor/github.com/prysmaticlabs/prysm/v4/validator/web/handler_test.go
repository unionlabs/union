package web

import (
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/testing/assert"
)

func TestHandler(t *testing.T) {
	tests := []struct {
		name            string
		requestURI      string
		wantStatus      int
		wantContentType string
	}{
		{
			name:            "base route",
			requestURI:      "/",
			wantStatus:      200,
			wantContentType: "text/html; charset=utf-8",
		},
		{
			name:            "index.html",
			requestURI:      "/index.html",
			wantStatus:      200,
			wantContentType: "text/html; charset=utf-8",
		},
		{
			name:            "bad route",
			requestURI:      "/foobar_bad",
			wantStatus:      200, // Serves index.html by default.
			wantContentType: "text/html; charset=utf-8",
		},
		{
			name:            "favicon.ico",
			requestURI:      "/favicon.ico",
			wantStatus:      200,
			wantContentType: "",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			req := &http.Request{RequestURI: tt.requestURI}
			res := httptest.NewRecorder()
			Handler(res, req)
			assert.Equal(t, tt.wantStatus, res.Result().StatusCode)
			if tt.wantContentType != "" {
				assert.Equal(t, tt.wantContentType, res.Result().Header.Get("Content-Type"))
			}
		})
	}
}
