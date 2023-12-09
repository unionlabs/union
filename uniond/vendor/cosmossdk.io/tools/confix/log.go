package confix

import (
	"context"
	"io"

	"github.com/creachadair/tomledit/transform"
)

// WithLogWriter returns a child of ctx with a logger attached that sends
// output to w. This is a convenience wrapper for transform.WithLogWriter.
func WithLogWriter(ctx context.Context, w io.Writer) context.Context {
	return transform.WithLogWriter(ctx, w)
}
