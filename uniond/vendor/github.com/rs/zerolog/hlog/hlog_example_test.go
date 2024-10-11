// +build !binary_log

package hlog_test

import (
	"net/http"
	"os"
	"time"

	"net/http/httptest"

	"github.com/rs/zerolog"
	"github.com/rs/zerolog/hlog"
)

// fake alice to avoid dep
type middleware func(http.Handler) http.Handler
type alice struct {
	m []middleware
}

func (a alice) Append(m middleware) alice {
	a.m = append(a.m, m)
	return a
}
func (a alice) Then(h http.Handler) http.Handler {
	for i := range a.m {
		h = a.m[len(a.m)-1-i](h)
	}
	return h
}

func init() {
	zerolog.TimestampFunc = func() time.Time {
		return time.Date(2001, time.February, 3, 4, 5, 6, 7, time.UTC)
	}
}

func Example_handler() {
	log := zerolog.New(os.Stdout).With().
		Timestamp().
		Str("role", "my-service").
		Str("host", "local-hostname").
		Logger()

	c := alice{}

	// Install the logger handler with default output on the console
	c = c.Append(hlog.NewHandler(log))

	// Install some provided extra handlers to set some request's context fields.
	// Thanks to those handlers, all our logs will come with some pre-populated fields.
	c = c.Append(hlog.RemoteAddrHandler("ip"))
	c = c.Append(hlog.UserAgentHandler("user_agent"))
	c = c.Append(hlog.RefererHandler("referer"))
	//c = c.Append(hlog.RequestIDHandler("req_id", "Request-Id"))

	// Here is your final handler
	h := c.Then(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// Get the logger from the request's context. You can safely assume it
		// will be always there: if the handler is removed, hlog.FromRequest
		// will return a no-op logger.
		hlog.FromRequest(r).Info().
			Str("user", "current user").
			Str("status", "ok").
			Msg("Something happened")
	}))
	http.Handle("/", h)

	h.ServeHTTP(httptest.NewRecorder(), &http.Request{})

	// Output: {"level":"info","role":"my-service","host":"local-hostname","user":"current user","status":"ok","time":"2001-02-03T04:05:06Z","message":"Something happened"}
}
