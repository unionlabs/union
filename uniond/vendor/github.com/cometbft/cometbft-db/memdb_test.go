package db

import (
	"testing"
)

func BenchmarkMemDBRangeScans1M(b *testing.B) {
	db := NewMemDB()
	defer db.Close()

	benchmarkRangeScans(b, db, int64(1e6))
}

func BenchmarkMemDBRangeScans10M(b *testing.B) {
	db := NewMemDB()
	defer db.Close()

	benchmarkRangeScans(b, db, int64(10e6))
}

func BenchmarkMemDBRandomReadsWrites(b *testing.B) {
	db := NewMemDB()
	defer db.Close()

	benchmarkRandomReadsWrites(b, db)
}
