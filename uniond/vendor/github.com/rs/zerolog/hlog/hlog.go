// Package hlog provides a set of http.Handler helpers for zerolog.
package hlog

import (
	"context"
	"net"
	"net/http"
	"strings"
	"time"

	"github.com/rs/xid"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/hlog/internal/mutil"
	"github.com/rs/zerolog/log"
)

// FromRequest gets the logger in the request's context.
// This is a shortcut for log.Ctx(r.Context())
func FromRequest(r *http.Request) *zerolog.Logger {
	return log.Ctx(r.Context())
}

// NewHandler injects log into requests context.
func NewHandler(log zerolog.Logger) func(http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			// Create a copy of the logger (including internal context slice)
			// to prevent data race when using UpdateContext.
			l := log.With().Logger()
			r = r.WithContext(l.WithContext(r.Context()))
			next.ServeHTTP(w, r)
		})
	}
}

// URLHandler adds the requested URL as a field to the context's logger
// using fieldKey as field key.
func URLHandler(fieldKey string) func(next http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			log := zerolog.Ctx(r.Context())
			log.UpdateContext(func(c zerolog.Context) zerolog.Context {
				return c.Str(fieldKey, r.URL.String())
			})
			next.ServeHTTP(w, r)
		})
	}
}

// MethodHandler adds the request method as a field to the context's logger
// using fieldKey as field key.
func MethodHandler(fieldKey string) func(next http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			log := zerolog.Ctx(r.Context())
			log.UpdateContext(func(c zerolog.Context) zerolog.Context {
				return c.Str(fieldKey, r.Method)
			})
			next.ServeHTTP(w, r)
		})
	}
}

// RequestHandler adds the request method and URL as a field to the context's logger
// using fieldKey as field key.
func RequestHandler(fieldKey string) func(next http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			log := zerolog.Ctx(r.Context())
			log.UpdateContext(func(c zerolog.Context) zerolog.Context {
				return c.Str(fieldKey, r.Method+" "+r.URL.String())
			})
			next.ServeHTTP(w, r)
		})
	}
}

// RemoteAddrHandler adds the request's remote address as a field to the context's logger
// using fieldKey as field key.
func RemoteAddrHandler(fieldKey string) func(next http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if r.RemoteAddr != "" {
				log := zerolog.Ctx(r.Context())
				log.UpdateContext(func(c zerolog.Context) zerolog.Context {
					return c.Str(fieldKey, r.RemoteAddr)
				})
			}
			next.ServeHTTP(w, r)
		})
	}
}

func getHost(hostPort string) string {
	if hostPort == "" {
		return ""
	}

	host, _, err := net.SplitHostPort(hostPort)
	if err != nil {
		return hostPort
	}
	return host
}

// RemoteIPHandler is similar to RemoteAddrHandler, but logs only
// an IP, not a port.
func RemoteIPHandler(fieldKey string) func(next http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			ip := getHost(r.RemoteAddr)
			if ip != "" {
				log := zerolog.Ctx(r.Context())
				log.UpdateContext(func(c zerolog.Context) zerolog.Context {
					return c.Str(fieldKey, ip)
				})
			}
			next.ServeHTTP(w, r)
		})
	}
}

// UserAgentHandler adds the request's user-agent as a field to the context's logger
// using fieldKey as field key.
func UserAgentHandler(fieldKey string) func(next http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if ua := r.Header.Get("User-Agent"); ua != "" {
				log := zerolog.Ctx(r.Context())
				log.UpdateContext(func(c zerolog.Context) zerolog.Context {
					return c.Str(fieldKey, ua)
				})
			}
			next.ServeHTTP(w, r)
		})
	}
}

// RefererHandler adds the request's referer as a field to the context's logger
// using fieldKey as field key.
func RefererHandler(fieldKey string) func(next http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if ref := r.Header.Get("Referer"); ref != "" {
				log := zerolog.Ctx(r.Context())
				log.UpdateContext(func(c zerolog.Context) zerolog.Context {
					return c.Str(fieldKey, ref)
				})
			}
			next.ServeHTTP(w, r)
		})
	}
}

// ProtoHandler adds the requests protocol version as a field to the context logger
// using fieldKey as field Key.
func ProtoHandler(fieldKey string) func(next http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			log := zerolog.Ctx(r.Context())
			log.UpdateContext(func(c zerolog.Context) zerolog.Context {
				return c.Str(fieldKey, r.Proto)
			})
			next.ServeHTTP(w, r)
		})
	}
}

// HTTPVersionHandler is similar to ProtoHandler, but it does not store the "HTTP/"
// prefix in the protocol name.
func HTTPVersionHandler(fieldKey string) func(next http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			proto := strings.TrimPrefix(r.Proto, "HTTP/")
			log := zerolog.Ctx(r.Context())
			log.UpdateContext(func(c zerolog.Context) zerolog.Context {
				return c.Str(fieldKey, proto)
			})
			next.ServeHTTP(w, r)
		})
	}
}

type idKey struct{}

// IDFromRequest returns the unique id associated to the request if any.
func IDFromRequest(r *http.Request) (id xid.ID, ok bool) {
	if r == nil {
		return
	}
	return IDFromCtx(r.Context())
}

// IDFromCtx returns the unique id associated to the context if any.
func IDFromCtx(ctx context.Context) (id xid.ID, ok bool) {
	id, ok = ctx.Value(idKey{}).(xid.ID)
	return
}

// CtxWithID adds the given xid.ID to the context
func CtxWithID(ctx context.Context, id xid.ID) context.Context {
	return context.WithValue(ctx, idKey{}, id)
}

// RequestIDHandler returns a handler setting a unique id to the request which can
// be gathered using IDFromRequest(req). This generated id is added as a field to the
// logger using the passed fieldKey as field name. The id is also added as a response
// header if the headerName is not empty.
//
// The generated id is a URL safe base64 encoded mongo object-id-like unique id.
// Mongo unique id generation algorithm has been selected as a trade-off between
// size and ease of use: UUID is less space efficient and snowflake requires machine
// configuration.
func RequestIDHandler(fieldKey, headerName string) func(next http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			ctx := r.Context()
			id, ok := IDFromRequest(r)
			if !ok {
				id = xid.New()
				ctx = CtxWithID(ctx, id)
				r = r.WithContext(ctx)
			}
			if fieldKey != "" {
				log := zerolog.Ctx(ctx)
				log.UpdateContext(func(c zerolog.Context) zerolog.Context {
					return c.Str(fieldKey, id.String())
				})
			}
			if headerName != "" {
				w.Header().Set(headerName, id.String())
			}
			next.ServeHTTP(w, r)
		})
	}
}

// CustomHeaderHandler adds given header from request's header as a field to
// the context's logger using fieldKey as field key.
func CustomHeaderHandler(fieldKey, header string) func(next http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if val := r.Header.Get(header); val != "" {
				log := zerolog.Ctx(r.Context())
				log.UpdateContext(func(c zerolog.Context) zerolog.Context {
					return c.Str(fieldKey, val)
				})
			}
			next.ServeHTTP(w, r)
		})
	}
}

// EtagHandler adds Etag header from response's header as a field to
// the context's logger using fieldKey as field key.
func EtagHandler(fieldKey string) func(next http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			defer func() {
				etag := w.Header().Get("Etag")
				if etag != "" {
					etag = strings.ReplaceAll(etag, `"`, "")
					log := zerolog.Ctx(r.Context())
					log.UpdateContext(func(c zerolog.Context) zerolog.Context {
						return c.Str(fieldKey, etag)
					})
				}
			}()
			next.ServeHTTP(w, r)
		})
	}
}

func ResponseHeaderHandler(fieldKey, headerName string) func(next http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			defer func() {
				value := w.Header().Get(headerName)
				if value != "" {
					log := zerolog.Ctx(r.Context())
					log.UpdateContext(func(c zerolog.Context) zerolog.Context {
						return c.Str(fieldKey, value)
					})
				}
			}()
			next.ServeHTTP(w, r)
		})
	}
}

// AccessHandler returns a handler that call f after each request.
func AccessHandler(f func(r *http.Request, status, size int, duration time.Duration)) func(next http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			start := time.Now()
			lw := mutil.WrapWriter(w)
			defer func() {
				f(r, lw.Status(), lw.BytesWritten(), time.Since(start))
			}()
			next.ServeHTTP(lw, r)
		})
	}
}

// HostHandler adds the request's host as a field to the context's logger
// using fieldKey as field key. If trimPort is set to true, then port is
// removed from the host.
func HostHandler(fieldKey string, trimPort ...bool) func(next http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			var host string
			if len(trimPort) > 0 && trimPort[0] {
				host = getHost(r.Host)
			} else {
				host = r.Host
			}
			if host != "" {
				log := zerolog.Ctx(r.Context())
				log.UpdateContext(func(c zerolog.Context) zerolog.Context {
					return c.Str(fieldKey, host)
				})
			}
			next.ServeHTTP(w, r)
		})
	}
}
