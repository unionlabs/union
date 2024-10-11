// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package trace

import (
	"testing"
)

func BenchmarkTraceStateParse(b *testing.B) {
	for _, test := range testcases {
		b.Run(test.name, func(b *testing.B) {
			b.ResetTimer()
			b.ReportAllocs()
			for i := 0; i < b.N; i++ {
				_, _ = ParseTraceState(test.in)
			}
		})
	}
}

func BenchmarkTraceStateString(b *testing.B) {
	for _, test := range testcases {
		if len(test.tracestate.list) == 0 {
			continue
		}
		b.Run(test.name, func(b *testing.B) {
			b.ResetTimer()
			b.ReportAllocs()
			for i := 0; i < b.N; i++ {
				_ = test.tracestate.String()
			}
		})
	}
}

func BenchmarkTraceStateInsert(b *testing.B) {
	for _, test := range insertTestcase {
		b.Run(test.name, func(b *testing.B) {
			b.ResetTimer()
			b.ReportAllocs()
			for i := 0; i < b.N; i++ {
				_, _ = test.tracestate.Insert(test.key, test.value)
			}
		})
	}
}
