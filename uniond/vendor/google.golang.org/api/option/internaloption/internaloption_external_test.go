// Copyright 2022 Google LLC.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package internaloption_test

import (
	"context"
	"fmt"

	"google.golang.org/api/option"
	"google.golang.org/api/option/internaloption"
)

type config struct {
	i int
}

type clientSpecificOption interface {
	option.ClientOption
	ApplyOpt(*config)
}

func WithFavoriteNumber(i int) option.ClientOption {
	return &withFavoriteNumber{i: i}
}

type withFavoriteNumber struct {
	internaloption.EmbeddableAdapter
	i int
}

func (w *withFavoriteNumber) ApplyOpt(c *config) {
	c.i = w.i
}

type Foo struct {
	i int
}

func NewFoo(ctx context.Context, opts ...option.ClientOption) (*Foo, error) {
	var conf config
	for _, opt := range opts {
		if fooOpt, ok := opt.(clientSpecificOption); ok {
			fooOpt.ApplyOpt(&conf)
		}
	}
	// Pass options to internals for dialing. All client-specific options will
	// be no-ops.
	return &Foo{i: conf.i}, nil
}

func (f *Foo) Number() int { return f.i }

func ExampleEmbeddableAdapter() {
	f, err := NewFoo(context.Background(), WithFavoriteNumber(42))
	if err != nil {
		// TODO: handle error
	}
	fmt.Println(f.Number())
	// Output: 42
}
