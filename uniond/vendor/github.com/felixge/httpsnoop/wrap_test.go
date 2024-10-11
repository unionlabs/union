package httpsnoop

import (
	"bytes"
	"io"
	"io/ioutil"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestWrap_integration(t *testing.T) {
	tests := []struct {
		Name     string
		Handler  http.Handler
		Hooks    Hooks
		WantCode int
		WantBody []byte
	}{
		{
			Name: "WriteHeader (no hook)",
			Handler: http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				w.WriteHeader(http.StatusNotFound)
			}),
			WantCode: http.StatusNotFound,
		},
		{
			Name: "WriteHeader",
			Handler: http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				w.WriteHeader(http.StatusNotFound)
			}),
			Hooks: Hooks{
				WriteHeader: func(next WriteHeaderFunc) WriteHeaderFunc {
					return func(code int) {
						if code != http.StatusNotFound {
							t.Errorf("got=%d want=%d", code, http.StatusNotFound)
						}
						next(http.StatusForbidden)
					}
				},
			},
			WantCode: http.StatusForbidden,
		},

		{
			Name: "Write (no hook)",
			Handler: http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				w.Write([]byte("foo"))
			}),
			WantCode: http.StatusOK,
			WantBody: []byte("foo"),
		},
		{
			Name: "Write (rewrite hook)",
			Handler: http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				if n, err := w.Write([]byte("foo")); err != nil {
					t.Errorf("got=%s", err)
				} else if got, want := n, len("foobar"); got != want {
					t.Errorf("got=%d want=%d", got, want)
				}
			}),
			Hooks: Hooks{
				Write: func(next WriteFunc) WriteFunc {
					return func(p []byte) (int, error) {
						if string(p) != "foo" {
							t.Errorf("%s", p)
						}
						return next([]byte("foobar"))
					}
				},
			},
			WantCode: http.StatusOK,
			WantBody: []byte("foobar"),
		},
		{
			Name: "Write (error hook)",
			Handler: http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				if n, err := w.Write([]byte("foo")); n != 0 {
					t.Errorf("got=%d want=%d", n, 0)
				} else if err != io.EOF {
					t.Errorf("got=%s want=%s", err, io.EOF)
				}
			}),
			Hooks: Hooks{
				Write: func(next WriteFunc) WriteFunc {
					return func(p []byte) (int, error) {
						if string(p) != "foo" {
							t.Errorf("%s", p)
						}
						return 0, io.EOF
					}
				},
			},
			WantCode: http.StatusOK,
		},
	}

	for _, test := range tests {
		func() {
			h := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				sw := Wrap(w, test.Hooks)
				test.Handler.ServeHTTP(sw, r)
			})
			s := httptest.NewServer(h)
			defer s.Close()
			res, err := http.Get(s.URL)
			if err != nil {
				t.Fatal(err)
			}
			defer res.Body.Close()
			gotBody, err := ioutil.ReadAll(res.Body)
			if res.StatusCode != test.WantCode {
				t.Errorf("got=%d want=%d", res.StatusCode, test.WantCode)
			} else if !bytes.Equal(gotBody, test.WantBody) {
				t.Errorf("got=%s want=%s", gotBody, test.WantBody)
			}
		}()
	}
}
