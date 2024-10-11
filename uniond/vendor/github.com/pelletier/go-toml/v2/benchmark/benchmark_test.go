package benchmark_test

import (
	"bytes"
	"io/ioutil"
	"testing"
	"time"

	"github.com/pelletier/go-toml/v2"
	"github.com/stretchr/testify/require"
)

func TestUnmarshalSimple(t *testing.T) {
	doc := []byte(`A = "hello"`)
	d := struct {
		A string
	}{}

	err := toml.Unmarshal(doc, &d)
	if err != nil {
		panic(err)
	}
}

func BenchmarkUnmarshal(b *testing.B) {
	b.Run("SimpleDocument", func(b *testing.B) {
		doc := []byte(`A = "hello"`)

		b.Run("struct", func(b *testing.B) {
			b.SetBytes(int64(len(doc)))
			b.ReportAllocs()
			b.ResetTimer()

			for i := 0; i < b.N; i++ {
				d := struct {
					A string
				}{}

				err := toml.Unmarshal(doc, &d)
				if err != nil {
					panic(err)
				}
			}
		})

		b.Run("map", func(b *testing.B) {
			b.SetBytes(int64(len(doc)))
			b.ReportAllocs()
			b.ResetTimer()

			for i := 0; i < b.N; i++ {
				d := map[string]interface{}{}
				err := toml.Unmarshal(doc, &d)
				if err != nil {
					panic(err)
				}
			}
		})
	})

	b.Run("ReferenceFile", func(b *testing.B) {
		bytes, err := ioutil.ReadFile("benchmark.toml")
		if err != nil {
			b.Fatal(err)
		}

		b.Run("struct", func(b *testing.B) {
			b.SetBytes(int64(len(bytes)))
			b.ReportAllocs()
			b.ResetTimer()
			for i := 0; i < b.N; i++ {
				d := benchmarkDoc{}
				err := toml.Unmarshal(bytes, &d)
				if err != nil {
					panic(err)
				}
			}
		})

		b.Run("map", func(b *testing.B) {
			b.SetBytes(int64(len(bytes)))
			b.ReportAllocs()
			b.ResetTimer()
			for i := 0; i < b.N; i++ {
				d := map[string]interface{}{}
				err := toml.Unmarshal(bytes, &d)
				if err != nil {
					panic(err)
				}
			}
		})
	})

	b.Run("HugoFrontMatter", func(b *testing.B) {
		b.SetBytes(int64(len(hugoFrontMatterbytes)))
		b.ReportAllocs()
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			d := map[string]interface{}{}
			err := toml.Unmarshal(hugoFrontMatterbytes, &d)
			if err != nil {
				panic(err)
			}
		}
	})
}

func marshal(v interface{}) ([]byte, error) {
	var b bytes.Buffer
	enc := toml.NewEncoder(&b)
	err := enc.Encode(v)
	return b.Bytes(), err
}

func BenchmarkMarshal(b *testing.B) {
	b.Run("SimpleDocument", func(b *testing.B) {
		doc := []byte(`A = "hello"`)

		b.Run("struct", func(b *testing.B) {
			d := struct {
				A string
			}{}

			err := toml.Unmarshal(doc, &d)
			if err != nil {
				panic(err)
			}

			b.ReportAllocs()
			b.ResetTimer()

			var out []byte

			for i := 0; i < b.N; i++ {
				out, err = marshal(d)
				if err != nil {
					panic(err)
				}
			}

			b.SetBytes(int64(len(out)))
		})

		b.Run("map", func(b *testing.B) {
			d := map[string]interface{}{}
			err := toml.Unmarshal(doc, &d)
			if err != nil {
				panic(err)
			}

			b.ReportAllocs()
			b.ResetTimer()

			var out []byte

			for i := 0; i < b.N; i++ {
				out, err = marshal(d)
				if err != nil {
					panic(err)
				}
			}

			b.SetBytes(int64(len(out)))
		})
	})

	b.Run("ReferenceFile", func(b *testing.B) {
		bytes, err := ioutil.ReadFile("benchmark.toml")
		if err != nil {
			b.Fatal(err)
		}

		b.Run("struct", func(b *testing.B) {
			d := benchmarkDoc{}
			err := toml.Unmarshal(bytes, &d)
			if err != nil {
				panic(err)
			}
			b.ReportAllocs()
			b.ResetTimer()

			var out []byte

			for i := 0; i < b.N; i++ {
				out, err = marshal(d)
				if err != nil {
					panic(err)
				}
			}

			b.SetBytes(int64(len(out)))
		})

		b.Run("map", func(b *testing.B) {
			d := map[string]interface{}{}
			err := toml.Unmarshal(bytes, &d)
			if err != nil {
				panic(err)
			}

			b.ReportAllocs()
			b.ResetTimer()

			var out []byte
			for i := 0; i < b.N; i++ {
				out, err = marshal(d)
				if err != nil {
					panic(err)
				}
			}

			b.SetBytes(int64(len(out)))
		})
	})

	b.Run("HugoFrontMatter", func(b *testing.B) {
		d := map[string]interface{}{}
		err := toml.Unmarshal(hugoFrontMatterbytes, &d)
		if err != nil {
			panic(err)
		}

		b.ReportAllocs()
		b.ResetTimer()

		var out []byte

		for i := 0; i < b.N; i++ {
			out, err = marshal(d)
			if err != nil {
				panic(err)
			}
		}

		b.SetBytes(int64(len(out)))
	})
}

type benchmarkDoc struct {
	Table struct {
		Key      string
		Subtable struct {
			Key string
		}
		Inline struct {
			Name struct {
				First string
				Last  string
			}
			Point struct {
				X int64
				Y int64
			}
		}
	}
	String struct {
		Basic struct {
			Basic string
		}
		Multiline struct {
			Key1      string
			Key2      string
			Key3      string
			Continued struct {
				Key1 string
				Key2 string
				Key3 string
			}
		}
		Literal struct {
			Winpath   string
			Winpath2  string
			Quoted    string
			Regex     string
			Multiline struct {
				Regex2 string
				Lines  string
			}
		}
	}
	Integer struct {
		Key1        int64
		Key2        int64
		Key3        int64
		Key4        int64
		Underscores struct {
			Key1 int64
			Key2 int64
			Key3 int64
		}
	}
	Float struct {
		Fractional struct {
			Key1 float64
			Key2 float64
			Key3 float64
		}
		Exponent struct {
			Key1 float64
			Key2 float64
			Key3 float64
		}
		Both struct {
			Key float64
		}
		Underscores struct {
			Key1 float64
			Key2 float64
		}
	}
	Boolean struct {
		True  bool
		False bool
	}
	Datetime struct {
		Key1 time.Time
		Key2 time.Time
		Key3 time.Time
	}
	Array struct {
		Key1 []int64
		Key2 []string
		Key3 [][]int64
		// TODO: Key4 not supported by go-toml's Unmarshal
		Key4 []interface{}
		Key5 []int64
		Key6 []int64
	}
	Products []struct {
		Name  string
		Sku   int64
		Color string
	}
	Fruit []struct {
		Name     string
		Physical struct {
			Color string
			Shape string
		}
		Variety []struct {
			Name string
		}
	}
}

func TestUnmarshalReferenceFile(t *testing.T) {
	bytes, err := ioutil.ReadFile("benchmark.toml")
	require.NoError(t, err)
	d := benchmarkDoc{}
	err = toml.Unmarshal(bytes, &d)
	require.NoError(t, err)

	expected := benchmarkDoc{
		Table: struct {
			Key      string
			Subtable struct{ Key string }
			Inline   struct {
				Name struct {
					First string
					Last  string
				}
				Point struct {
					X int64
					Y int64
				}
			}
		}{
			Key: "value",
			Subtable: struct{ Key string }{
				Key: "another value",
			},
			// note: x.y.z.w is purposefully missing
			Inline: struct {
				Name struct {
					First string
					Last  string
				}
				Point struct {
					X int64
					Y int64
				}
			}{
				Name: struct {
					First string
					Last  string
				}{
					First: "Tom",
					Last:  "Preston-Werner",
				},
				Point: struct {
					X int64
					Y int64
				}{
					X: 1,
					Y: 2,
				},
			},
		},
		String: struct {
			Basic     struct{ Basic string }
			Multiline struct {
				Key1      string
				Key2      string
				Key3      string
				Continued struct {
					Key1 string
					Key2 string
					Key3 string
				}
			}
			Literal struct {
				Winpath   string
				Winpath2  string
				Quoted    string
				Regex     string
				Multiline struct {
					Regex2 string
					Lines  string
				}
			}
		}{
			Basic: struct{ Basic string }{
				Basic: "I'm a string. \"You can quote me\". Name\tJos\u00E9\nLocation\tSF.",
			},
			Multiline: struct {
				Key1      string
				Key2      string
				Key3      string
				Continued struct {
					Key1 string
					Key2 string
					Key3 string
				}
			}{
				Key1: "One\nTwo",
				Key2: "One\nTwo",
				Key3: "One\nTwo",

				Continued: struct {
					Key1 string
					Key2 string
					Key3 string
				}{
					Key1: `The quick brown fox jumps over the lazy dog.`,
					Key2: `The quick brown fox jumps over the lazy dog.`,
					Key3: `The quick brown fox jumps over the lazy dog.`,
				},
			},
			Literal: struct {
				Winpath   string
				Winpath2  string
				Quoted    string
				Regex     string
				Multiline struct {
					Regex2 string
					Lines  string
				}
			}{
				Winpath:  `C:\Users\nodejs\templates`,
				Winpath2: `\\ServerX\admin$\system32\`,
				Quoted:   `Tom "Dubs" Preston-Werner`,
				Regex:    `<\i\c*\s*>`,

				Multiline: struct {
					Regex2 string
					Lines  string
				}{
					Regex2: `I [dw]on't need \d{2} apples`,
					Lines: `The first newline is
trimmed in raw strings.
   All other whitespace
   is preserved.
`,
				},
			},
		},
		Integer: struct {
			Key1        int64
			Key2        int64
			Key3        int64
			Key4        int64
			Underscores struct {
				Key1 int64
				Key2 int64
				Key3 int64
			}
		}{
			Key1: 99,
			Key2: 42,
			Key3: 0,
			Key4: -17,

			Underscores: struct {
				Key1 int64
				Key2 int64
				Key3 int64
			}{
				Key1: 1000,
				Key2: 5349221,
				Key3: 12345,
			},
		},
		Float: struct {
			Fractional struct {
				Key1 float64
				Key2 float64
				Key3 float64
			}
			Exponent struct {
				Key1 float64
				Key2 float64
				Key3 float64
			}
			Both        struct{ Key float64 }
			Underscores struct {
				Key1 float64
				Key2 float64
			}
		}{
			Fractional: struct {
				Key1 float64
				Key2 float64
				Key3 float64
			}{
				Key1: 1.0,
				Key2: 3.1415,
				Key3: -0.01,
			},
			Exponent: struct {
				Key1 float64
				Key2 float64
				Key3 float64
			}{
				Key1: 5e+22,
				Key2: 1e6,
				Key3: -2e-2,
			},
			Both: struct{ Key float64 }{
				Key: 6.626e-34,
			},
			Underscores: struct {
				Key1 float64
				Key2 float64
			}{
				Key1: 9224617.445991228313,
				Key2: 1e100,
			},
		},
		Boolean: struct {
			True  bool
			False bool
		}{
			True:  true,
			False: false,
		},
		Datetime: struct {
			Key1 time.Time
			Key2 time.Time
			Key3 time.Time
		}{
			Key1: time.Date(1979, 5, 27, 7, 32, 0, 0, time.UTC),
			Key2: time.Date(1979, 5, 27, 0, 32, 0, 0, time.FixedZone("", -7*3600)),
			Key3: time.Date(1979, 5, 27, 0, 32, 0, 999999000, time.FixedZone("", -7*3600)),
		},
		Array: struct {
			Key1 []int64
			Key2 []string
			Key3 [][]int64
			Key4 []interface{}
			Key5 []int64
			Key6 []int64
		}{
			Key1: []int64{1, 2, 3},
			Key2: []string{"red", "yellow", "green"},
			Key3: [][]int64{{1, 2}, {3, 4, 5}},
			Key4: []interface{}{
				[]interface{}{int64(1), int64(2)},
				[]interface{}{"a", "b", "c"},
			},
			Key5: []int64{1, 2, 3},
			Key6: []int64{1, 2},
		},
		Products: []struct {
			Name  string
			Sku   int64
			Color string
		}{
			{
				Name: "Hammer",
				Sku:  738594937,
			},
			{},
			{
				Name:  "Nail",
				Sku:   284758393,
				Color: "gray",
			},
		},
		Fruit: []struct {
			Name     string
			Physical struct {
				Color string
				Shape string
			}
			Variety []struct{ Name string }
		}{
			{
				Name: "apple",
				Physical: struct {
					Color string
					Shape string
				}{
					Color: "red",
					Shape: "round",
				},
				Variety: []struct{ Name string }{
					{Name: "red delicious"},
					{Name: "granny smith"},
				},
			},
			{
				Name: "banana",
				Variety: []struct{ Name string }{
					{Name: "plantain"},
				},
			},
		},
	}

	require.Equal(t, expected, d)
}

var hugoFrontMatterbytes = []byte(`
categories = ["Development", "VIM"]
date = "2012-04-06"
description = "spf13-vim is a cross platform distribution of vim plugins and resources for Vim."
slug = "spf13-vim-3-0-release-and-new-website"
tags = [".vimrc", "plugins", "spf13-vim", "vim"]
title = "spf13-vim 3.0 release and new website"
include_toc = true
show_comments = false

[[cascade]]
  background = "yosemite.jpg"
  [cascade._target]
    kind = "page"
    lang = "en"
    path = "/blog/**"

[[cascade]]
  background = "goldenbridge.jpg"
  [cascade._target]
    kind = "section"
`)
