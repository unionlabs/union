package collections

import (
	"context"
	"errors"
)

// DefaultSequenceStart defines the default starting number of a sequence.
const DefaultSequenceStart uint64 = 0

// Sequence builds on top of an Item, and represents a monotonically increasing number.
type Sequence Item[uint64]

// NewSequence instantiates a new sequence given
// a Schema, a Prefix and humanized name for the sequence.
func NewSequence(schema *SchemaBuilder, prefix Prefix, name string) Sequence {
	return (Sequence)(NewItem(schema, prefix, name, Uint64Value))
}

// Peek returns the current sequence value, if no number
// is set then the DefaultSequenceStart is returned.
// Errors on encoding issues.
func (s Sequence) Peek(ctx context.Context) (uint64, error) {
	n, err := (Item[uint64])(s).Get(ctx)
	switch {
	case err == nil:
		return n, nil
	case errors.Is(err, ErrNotFound):
		return DefaultSequenceStart, nil
	default:
		return 0, err
	}
}

// Next returns the next sequence number, and sets the next expected sequence.
// Errors on encoding issues.
func (s Sequence) Next(ctx context.Context) (uint64, error) {
	seq, err := s.Peek(ctx)
	if err != nil {
		return 0, err
	}
	return seq, s.Set(ctx, seq+1)
}

// Set hard resets the sequence to the provided value.
// Errors on encoding issues.
func (s Sequence) Set(ctx context.Context, value uint64) error {
	return (Item[uint64])(s).Set(ctx, value)
}
