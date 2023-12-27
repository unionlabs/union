package debug

import (
	"bytes"
	"fmt"
	"io"
	"net/http"
	"net/http/httptrace"
	"net/http/httputil"
)

// Transport implements http.RoundTripper and can be used to wrap other HTTP
// transports for debugging, normally http.DefaultTransport.
type Transport struct {
	http.RoundTripper
	Output io.Writer
	// Dump controls whether to dump HTTP request and responses.
	Dump bool
	// Trace enables usage of net/http/httptrace.
	Trace bool
}

func (t *Transport) RoundTrip(req *http.Request) (*http.Response, error) {
	var buf bytes.Buffer
	if t.Dump {
		b, err := httputil.DumpRequestOut(req, true)
		if err != nil {
			panic(err)
		}
		_, err = buf.Write(ensureTrailingNewline(b))
		if err != nil {
			panic(err)
		}
	}
	if t.Trace {
		trace := &httptrace.ClientTrace{
			DNSDone: func(di httptrace.DNSDoneInfo) {
				fmt.Fprintf(&buf, "* DNS %v → %v\n", req.Host, di.Addrs)
			},
			GotConn: func(ci httptrace.GotConnInfo) {
				fmt.Fprintf(&buf, "* Connection local=%v remote=%v", ci.Conn.LocalAddr(), ci.Conn.RemoteAddr())
				if ci.Reused {
					fmt.Fprint(&buf, " (reused)")
				}
				if ci.WasIdle {
					fmt.Fprintf(&buf, " (idle %v)", ci.IdleTime)
				}
				fmt.Fprintln(&buf)
			},
		}
		req = req.WithContext(httptrace.WithClientTrace(req.Context(), trace))
	}
	resp, err := t.RoundTripper.RoundTrip(req)
	if err != nil {
		return nil, err
	}
	if t.Dump {
		b, err := httputil.DumpResponse(resp, true)
		if err != nil {
			panic(err)
		}
		_, err = buf.Write(ensureTrailingNewline(b))
		if err != nil {
			panic(err)
		}
	}
	_, err = io.Copy(t.Output, &buf)
	if err != nil {
		panic(err)
	}
	return resp, nil
}

func ensureTrailingNewline(b []byte) []byte {
	if len(b) > 0 && b[len(b)-1] != '\n' {
		b = append(b, '\n')
	}
	return b
}
