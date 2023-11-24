// Copyright (C) 2022 Michael J. Fromberger. All Rights Reserved.

// Package transform defines some useful transformations on TOML documents.
package transform

import (
	"context"
	"fmt"
	"io"
	"text/tabwriter"

	"github.com/creachadair/tomledit"
)

// An Applier applies one or more transformations to a document.
// The Func, Step, and Plan types implement this interface.
type Applier interface {
	Apply(context.Context, *tomledit.Document) error
}

// A Func implements the applier interface with a function.
type Func func(context.Context, *tomledit.Document) error

// Apply applies f to doc, satisfying the Applier interface.
func (f Func) Apply(ctx context.Context, doc *tomledit.Document) error { return f(ctx, doc) }

// A Step is a single transformation in a plan.
type Step struct {
	Desc    string  // human-readable description (for logging)
	T       Applier // the transformation itself
	ErrorOK bool    // if true, ignore errors from evaluating T
}

// Apply applies the transformation step and reports its error. If ErrorOK is
// true, a non-nil error from the transformation is ignored.
func (s Step) Apply(ctx context.Context, doc *tomledit.Document) error {
	err := s.T.Apply(ctx, doc)
	if s.ErrorOK {
		return nil
	}
	return err
}

// A Plan is a sequence of transformations to be applied in order.
type Plan []Step

// Apply applies each step of p to the document in order, and reports the error
// from the first step that fails, or nil. An empty Plan always succeeds.
func (p Plan) Apply(ctx context.Context, doc *tomledit.Document) error {
	w := newLogWriter(ctx)
	defer w.close()
	for i, step := range p {
		if step.Desc != "" {
			w.log("[%d]\t%s", i+1, step.Desc)
		} else {
			w.log("[%d]\t(no description)")
		}
		if err := step.Apply(ctx, doc); err != nil {
			w.log("\t| FAILED: %v\n", err)
			return err
		}
		w.log("\t| OK\n")
	}
	return nil
}

type stepLogKey struct{}

// WithLogWriter attaches w to ctx as a logging target.
func WithLogWriter(ctx context.Context, w io.Writer) context.Context {
	return context.WithValue(ctx, stepLogKey{}, w)
}

type logWriter struct {
	log   func(string, ...interface{})
	close func() error
}

func newLogWriter(ctx context.Context) logWriter {
	if v := ctx.Value(stepLogKey{}); v != nil {
		tw := tabwriter.NewWriter(v.(io.Writer), 0, 8, 1, ' ', 0)
		return logWriter{
			log: func(msg string, args ...interface{}) {
				fmt.Fprintf(tw, msg, args...)
			},
			close: tw.Flush,
		}
	}
	return logWriter{
		log:   func(string, ...interface{}) {},
		close: func() error { return nil },
	}
}
