// Copyright 2022 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package slog_test

import (
	"net/http"
	"os"
	"time"

	"golang.org/x/exp/slog"
	"golang.org/x/exp/slog/internal/testutil"
)

func ExampleGroup() {
	r, _ := http.NewRequest("GET", "localhost", nil)
	// ...

	logger := slog.New(slog.NewTextHandler(os.Stdout, &slog.HandlerOptions{ReplaceAttr: testutil.RemoveTime}))
	slog.SetDefault(logger)

	slog.Info("finished",
		slog.Group("req",
			slog.String("method", r.Method),
			slog.String("url", r.URL.String())),
		slog.Int("status", http.StatusOK),
		slog.Duration("duration", time.Second))

	// Output:
	// level=INFO msg=finished req.method=GET req.url=localhost status=200 duration=1s
}
