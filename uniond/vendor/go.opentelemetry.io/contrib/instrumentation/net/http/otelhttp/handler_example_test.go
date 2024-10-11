// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0
package otelhttp_test

import (
	"context"
	"fmt"
	"io"
	"log"
	"net/http"
	"strings"

	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/trace"

	"go.opentelemetry.io/contrib/instrumentation/net/http/otelhttp"
)

func ExampleNewHandler() {
	/* curl -v -d "a painting" http://localhost:7777/hello/bob/ross
	...
	* upload completely sent off: 10 out of 10 bytes
	< HTTP/1.1 200 OK
	< Traceparent: 00-76ae040ee5753f38edf1c2bd9bd128bd-dd394138cfd7a3dc-01
	< Date: Fri, 04 Oct 2019 02:33:08 GMT
	< Content-Length: 45
	< Content-Type: text/plain; charset=utf-8
	<
	Hello, bob/ross!
	You sent me this:
	a painting
	*/

	figureOutName := func(ctx context.Context, s string) (string, error) {
		pp := strings.SplitN(s, "/", 2)
		var err error
		switch pp[1] {
		case "":
			err = fmt.Errorf("expected /hello/:name in %q", s)
		default:
			trace.SpanFromContext(ctx).SetAttributes(attribute.String("name", pp[1]))
		}
		return pp[1], err
	}

	var mux http.ServeMux
	mux.Handle("/hello/",
		otelhttp.WithRouteTag("/hello/:name", http.HandlerFunc(
			func(w http.ResponseWriter, r *http.Request) {
				ctx := r.Context()
				labeler, _ := otelhttp.LabelerFromContext(ctx)

				var name string
				// Wrap another function in its own span
				if err := func(ctx context.Context) error {
					ctx, span := trace.SpanFromContext(ctx).TracerProvider().Tracer("exampleTracer").Start(ctx, "figureOutName")
					defer span.End()

					var err error
					name, err = figureOutName(ctx, r.URL.Path[1:])
					return err
				}(ctx); err != nil {
					log.Println("error figuring out name: ", err)
					http.Error(w, err.Error(), http.StatusInternalServerError)
					labeler.Add(attribute.Bool("error", true))
					return
				}

				d, err := io.ReadAll(r.Body)
				if err != nil {
					log.Println("error reading body: ", err)
					w.WriteHeader(http.StatusBadRequest)
					labeler.Add(attribute.Bool("error", true))
					return
				}

				n, err := io.WriteString(w, "Hello, "+name+"!\nYou sent me this:\n"+string(d))
				if err != nil {
					log.Printf("error writing reply after %d bytes: %s", n, err)
					labeler.Add(attribute.Bool("error", true))
				}
			}),
		),
	)

	if err := http.ListenAndServe(":7777", //nolint:gosec // Ignoring G114: Use of net/http serve function that has no support for setting timeouts.
		otelhttp.NewHandler(&mux, "server",
			otelhttp.WithMessageEvents(otelhttp.ReadEvents, otelhttp.WriteEvents),
		),
	); err != nil {
		log.Fatal(err)
	}
}
