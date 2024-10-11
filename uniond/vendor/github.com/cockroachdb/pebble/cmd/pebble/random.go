// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package main

import (
	"strconv"
	"strings"
	"time"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/randvar"
	"github.com/cockroachdb/pebble/internal/rate"
)

type rateFlag struct {
	randvar.Flag
	fluctuateDuration time.Duration
	spec              string
}

func newRateFlag(spec string) *rateFlag {
	f := &rateFlag{}
	if err := f.Set(spec); err != nil {
		panic(err)
	}
	return f
}

func (f *rateFlag) String() string {
	return f.spec
}

// Type implements the Flag.Value interface.
func (f *rateFlag) Type() string {
	return "ratevar"
}

// Set implements the Flag.Value interface.
func (f *rateFlag) Set(spec string) error {
	if spec == "" {
		if err := f.Flag.Set("0"); err != nil {
			return err
		}
		f.fluctuateDuration = time.Duration(0)
		f.spec = spec
		return nil
	}

	parts := strings.Split(spec, "/")
	if len(parts) == 0 || len(parts) > 2 {
		return errors.Errorf("invalid ratevar spec: %s", errors.Safe(spec))
	}
	if err := f.Flag.Set(parts[0]); err != nil {
		return err
	}
	// Don't fluctuate by default.
	f.fluctuateDuration = time.Duration(0)
	if len(parts) == 2 {
		fluctuateDurationFloat, err := strconv.ParseFloat(parts[1], 64)
		if err != nil {
			return err
		}
		f.fluctuateDuration = time.Duration(fluctuateDurationFloat) * time.Second
	}
	f.spec = spec
	return nil
}

func (f *rateFlag) newRateLimiter() *rate.Limiter {
	if f.spec == "" {
		return nil
	}
	rng := randvar.NewRand()
	limiter := rate.NewLimiter(float64(f.Uint64(rng)), 1)
	if f.fluctuateDuration != 0 {
		go func(limiter *rate.Limiter) {
			ticker := time.NewTicker(f.fluctuateDuration)
			for range ticker.C {
				limiter.SetRate(float64(f.Uint64(rng)))
			}
		}(limiter)
	}
	return limiter
}

func wait(l *rate.Limiter) {
	if l != nil {
		l.Wait(1)
	}
}
