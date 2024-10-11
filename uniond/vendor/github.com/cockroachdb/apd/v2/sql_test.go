// Copyright 2017 The Cockroach Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied. See the License for the specific language governing
// permissions and limitations under the License.

// +build sql

package apd

import (
	"database/sql"
	"flag"
	"testing"

	_ "github.com/lib/pq"
)

var (
	flagPostgres = flag.String("postgres", "postgres://postgres@localhost/apd?sslmode=disable", "Postgres connection string to an empty database")
)

// TestSQL tests the Scan and Value methods of Decimal.
func TestSQL(t *testing.T) {
	db, err := sql.Open("postgres", *flagPostgres)
	if err != nil {
		t.Fatal(err)
	}
	a, _, err := NewFromString("1234.567e5")
	if err != nil {
		t.Fatal(err)
	}
	if _, err := db.Exec("drop table if exists d"); err != nil {
		t.Fatal(err)
	}
	if _, err := db.Exec("create table d (v decimal)"); err != nil {
		t.Fatal(err)
	}
	if _, err := db.Exec("insert into d values ($1)", a); err != nil {
		t.Fatal(err)
	}
	if _, err := db.Exec("update d set v = v + 1e5"); err != nil {
		t.Fatal(err)
	}
	var b, c, d Decimal
	var nd NullDecimal
	if err := db.QueryRow("select v, v::text, v::int, v::float, v from d").Scan(a, &b, &c, &d, &nd); err != nil {
		t.Fatal(err)
	}
	want, _, err := NewFromString("123556700")
	if err != nil {
		t.Fatal(err)
	}
	for i, v := range []*Decimal{a, &b, &c, &d, &nd.Decimal} {
		if v.Cmp(want) != 0 {
			t.Fatalf("%d: unexpected: %s, want: %s", i, v.String(), want.String())
		}
	}

	if _, err := db.Exec("update d set v = NULL"); err != nil {
		t.Fatal(err)
	}
	if err := db.QueryRow("select v from d").Scan(&nd); err != nil {
		t.Fatal(err)
	}
	if nd.Valid {
		t.Fatal("expected null")
	}
}
