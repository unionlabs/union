// Copyright 2014 Unknwon
//
// Licensed under the Apache License, Version 2.0 (the "License"): you may
// not use this file except in compliance with the License. You may obtain
// a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations
// under the License.

package ini

import (
	"bytes"
	"fmt"
	"strings"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

type testNested struct {
	Cities      []string `delim:"|"`
	Visits      []time.Time
	Years       []int
	Numbers     []int64
	Ages        []uint
	Populations []uint64
	Coordinates []float64
	Flags       []bool
	Note        string
	Unused      int `ini:"-"`
}

type TestEmbeded struct {
	GPA float64
}

type testStruct struct {
	Name           string `ini:"NAME"`
	Age            int
	Male           bool
	Money          float64
	Born           time.Time
	Time           time.Duration `ini:"Duration"`
	OldVersionTime time.Duration
	Others         testNested
	OthersPtr      *testNested
	NilPtr         *testNested
	*TestEmbeded   `ini:"grade"`
	Unused         int `ini:"-"`
	Unsigned       uint
	Omitted        bool     `ini:"omitthis,omitempty"`
	Shadows        []string `ini:",allowshadow"`
	ShadowInts     []int    `ini:"Shadows,allowshadow"`
	BoolPtr        *bool
	BoolPtrNil     *bool
	FloatPtr       *float64
	FloatPtrNil    *float64
	IntPtr         *int
	IntPtrNil      *int
	UintPtr        *uint
	UintPtrNil     *uint
	StringPtr      *string
	StringPtrNil   *string
	TimePtr        *time.Time
	TimePtrNil     *time.Time
	DurationPtr    *time.Duration
	DurationPtrNil *time.Duration
}

type testInterface struct {
	Address    string
	ListenPort int
	PrivateKey string
}

type testPeer struct {
	PublicKey    string
	PresharedKey string
	AllowedIPs   []string `delim:","`
}

type testNonUniqueSectionsStruct struct {
	Interface testInterface
	Peer      []testPeer `ini:",nonunique"`
}

type BaseStruct struct {
	Base bool
}

type testExtend struct {
	BaseStruct `ini:",extends"`
	Extend     bool
}

const confDataStruct = `
NAME = Unknwon
Age = 21
Male = true
Money = 1.25
Born = 1993-10-07T20:17:05Z
Duration = 2h45m
OldVersionTime = 30
Unsigned = 3
omitthis = true
Shadows = 1, 2
Shadows = 3, 4
BoolPtr = false
FloatPtr = 0
IntPtr = 0
UintPtr = 0
StringPtr = ""
TimePtr = 0001-01-01T00:00:00Z
DurationPtr = 0s

[Others]
Cities = HangZhou|Boston
Visits = 1993-10-07T20:17:05Z, 1993-10-07T20:17:05Z
Years = 1993,1994
Numbers = 10010,10086
Ages = 18,19
Populations = 12345678,98765432
Coordinates = 192.168,10.11
Flags       = true,false
Note = Hello world!

[OthersPtr]
Cities = HangZhou|Boston
Visits = 1993-10-07T20:17:05Z, 1993-10-07T20:17:05Z
Years = 1993,1994
Numbers = 10010,10086
Ages = 18,19
Populations = 12345678,98765432
Coordinates = 192.168,10.11
Flags       = true,false
Note = Hello world!

[grade]
GPA = 2.8

[foo.bar]
Here = there
When = then

[extended]
Base = true
Extend = true
`

const confNonUniqueSectionDataStruct = `[Interface]
Address    = 10.2.0.1/24
ListenPort = 34777
PrivateKey = privServerKey

[Peer]
PublicKey    = pubClientKey
PresharedKey = psKey
AllowedIPs   = 10.2.0.2/32,fd00:2::2/128

[Peer]
PublicKey    = pubClientKey2
PresharedKey = psKey2
AllowedIPs   = 10.2.0.3/32,fd00:2::3/128
`

type unsupport struct {
	Byte byte
}

type unsupport2 struct {
	Others struct {
		Cities byte
	}
}

type Unsupport3 struct {
	Cities byte
}

type unsupport4 struct {
	*Unsupport3 `ini:"Others"`
}

type defaultValue struct {
	Name     string
	Age      int
	Male     bool
	Optional *bool
	Money    float64
	Born     time.Time
	Cities   []string
}

type fooBar struct {
	Here, When string
}

const invalidDataConfStruct = `
Name = 
Age = age
Male = 123
Money = money
Born = nil
Cities = 
`

func Test_MapToStruct(t *testing.T) {
	t.Run("map to struct", func(t *testing.T) {
		t.Run("map file to struct", func(t *testing.T) {
			ts := new(testStruct)
			assert.NoError(t, MapTo(ts, []byte(confDataStruct)))

			assert.Equal(t, "Unknwon", ts.Name)
			assert.Equal(t, 21, ts.Age)
			assert.True(t, ts.Male)
			assert.Equal(t, 1.25, ts.Money)
			assert.Equal(t, uint(3), ts.Unsigned)

			ti, err := time.Parse(time.RFC3339, "1993-10-07T20:17:05Z")
			require.NoError(t, err)
			assert.Equal(t, ti.String(), ts.Born.String())

			dur, err := time.ParseDuration("2h45m")
			require.NoError(t, err)
			assert.Equal(t, dur.Seconds(), ts.Time.Seconds())

			assert.Equal(t, 30*time.Second, ts.OldVersionTime*time.Second)

			assert.Equal(t, "HangZhou,Boston", strings.Join(ts.Others.Cities, ","))
			assert.Equal(t, ti.String(), ts.Others.Visits[0].String())
			assert.Equal(t, "[1993 1994]", fmt.Sprint(ts.Others.Years))
			assert.Equal(t, "[10010 10086]", fmt.Sprint(ts.Others.Numbers))
			assert.Equal(t, "[18 19]", fmt.Sprint(ts.Others.Ages))
			assert.Equal(t, "[12345678 98765432]", fmt.Sprint(ts.Others.Populations))
			assert.Equal(t, "[192.168 10.11]", fmt.Sprint(ts.Others.Coordinates))
			assert.Equal(t, "[true false]", fmt.Sprint(ts.Others.Flags))
			assert.Equal(t, "Hello world!", ts.Others.Note)
			assert.Equal(t, 2.8, ts.TestEmbeded.GPA)

			assert.Equal(t, "HangZhou,Boston", strings.Join(ts.OthersPtr.Cities, ","))
			assert.Equal(t, ti.String(), ts.OthersPtr.Visits[0].String())
			assert.Equal(t, "[1993 1994]", fmt.Sprint(ts.OthersPtr.Years))
			assert.Equal(t, "[10010 10086]", fmt.Sprint(ts.OthersPtr.Numbers))
			assert.Equal(t, "[18 19]", fmt.Sprint(ts.OthersPtr.Ages))
			assert.Equal(t, "[12345678 98765432]", fmt.Sprint(ts.OthersPtr.Populations))
			assert.Equal(t, "[192.168 10.11]", fmt.Sprint(ts.OthersPtr.Coordinates))
			assert.Equal(t, "[true false]", fmt.Sprint(ts.OthersPtr.Flags))
			assert.Equal(t, "Hello world!", ts.OthersPtr.Note)

			assert.Nil(t, ts.NilPtr)

			assert.Equal(t, false, *ts.BoolPtr)
			assert.Nil(t, ts.BoolPtrNil)
			assert.Equal(t, float64(0), *ts.FloatPtr)
			assert.Nil(t, ts.FloatPtrNil)
			assert.Equal(t, 0, *ts.IntPtr)
			assert.Nil(t, ts.IntPtrNil)
			assert.Equal(t, uint(0), *ts.UintPtr)
			assert.Nil(t, ts.UintPtrNil)
			assert.Equal(t, "", *ts.StringPtr)
			assert.Nil(t, ts.StringPtrNil)
			assert.NotNil(t, *ts.TimePtr)
			assert.Nil(t, ts.TimePtrNil)
			assert.Equal(t, time.Duration(0), *ts.DurationPtr)
			assert.Nil(t, ts.DurationPtrNil)
		})

		t.Run("map section to struct", func(t *testing.T) {
			foobar := new(fooBar)
			f, err := Load([]byte(confDataStruct))
			require.NoError(t, err)

			assert.NoError(t, f.Section("foo.bar").MapTo(foobar))
			assert.Equal(t, "there", foobar.Here)
			assert.Equal(t, "then", foobar.When)
		})

		t.Run("map to non-pointer struct", func(t *testing.T) {
			f, err := Load([]byte(confDataStruct))
			require.NoError(t, err)
			require.NotNil(t, f)

			assert.Error(t, f.MapTo(testStruct{}))
		})

		t.Run("map to unsupported type", func(t *testing.T) {
			f, err := Load([]byte(confDataStruct))
			require.NoError(t, err)
			require.NotNil(t, f)

			f.NameMapper = func(raw string) string {
				if raw == "Byte" {
					return "NAME"
				}
				return raw
			}
			assert.Error(t, f.MapTo(&unsupport{}))
			assert.Error(t, f.MapTo(&unsupport2{}))
			assert.Error(t, f.MapTo(&unsupport4{}))
		})

		t.Run("map to omitempty field", func(t *testing.T) {
			ts := new(testStruct)
			assert.NoError(t, MapTo(ts, []byte(confDataStruct)))

			assert.Equal(t, true, ts.Omitted)
		})

		t.Run("map with shadows", func(t *testing.T) {
			f, err := LoadSources(LoadOptions{AllowShadows: true}, []byte(confDataStruct))
			require.NoError(t, err)
			ts := new(testStruct)
			assert.NoError(t, f.MapTo(ts))

			assert.Equal(t, "1 2 3 4", strings.Join(ts.Shadows, " "))
			assert.Equal(t, "[1 2 3 4]", fmt.Sprintf("%v", ts.ShadowInts))
		})

		t.Run("map from invalid data source", func(t *testing.T) {
			assert.Error(t, MapTo(&testStruct{}, "hi"))
		})

		t.Run("map to wrong types and gain default values", func(t *testing.T) {
			f, err := Load([]byte(invalidDataConfStruct))
			require.NoError(t, err)

			ti, err := time.Parse(time.RFC3339, "1993-10-07T20:17:05Z")
			require.NoError(t, err)
			dv := &defaultValue{"Joe", 10, true, nil, 1.25, ti, []string{"HangZhou", "Boston"}}
			assert.NoError(t, f.MapTo(dv))
			assert.Equal(t, "Joe", dv.Name)
			assert.Equal(t, 10, dv.Age)
			assert.True(t, dv.Male)
			assert.Equal(t, 1.25, dv.Money)
			assert.Equal(t, ti.String(), dv.Born.String())
			assert.Equal(t, "HangZhou,Boston", strings.Join(dv.Cities, ","))
		})

		t.Run("map to extended base", func(t *testing.T) {
			f, err := Load([]byte(confDataStruct))
			require.NoError(t, err)
			require.NotNil(t, f)
			te := testExtend{}
			assert.NoError(t, f.Section("extended").MapTo(&te))
			assert.True(t, te.Base)
			assert.True(t, te.Extend)
		})
	})

	t.Run("map to struct in strict mode", func(t *testing.T) {
		f, err := Load([]byte(`
name=bruce
age=a30`))
		require.NoError(t, err)

		type Strict struct {
			Name string `ini:"name"`
			Age  int    `ini:"age"`
		}
		s := new(Strict)

		assert.Error(t, f.Section("").StrictMapTo(s))
	})

	t.Run("map slice in strict mode", func(t *testing.T) {
		f, err := Load([]byte(`
names=alice, bruce`))
		require.NoError(t, err)

		type Strict struct {
			Names []string `ini:"names"`
		}
		s := new(Strict)

		assert.NoError(t, f.Section("").StrictMapTo(s))
		assert.Equal(t, "[alice bruce]", fmt.Sprint(s.Names))
	})
}

func Test_MapToStructNonUniqueSections(t *testing.T) {
	t.Run("map to struct non unique", func(t *testing.T) {
		t.Run("map file to struct non unique", func(t *testing.T) {
			f, err := LoadSources(LoadOptions{AllowNonUniqueSections: true}, []byte(confNonUniqueSectionDataStruct))
			require.NoError(t, err)
			ts := new(testNonUniqueSectionsStruct)

			assert.NoError(t, f.MapTo(ts))

			assert.Equal(t, "10.2.0.1/24", ts.Interface.Address)
			assert.Equal(t, 34777, ts.Interface.ListenPort)
			assert.Equal(t, "privServerKey", ts.Interface.PrivateKey)

			assert.Equal(t, "pubClientKey", ts.Peer[0].PublicKey)
			assert.Equal(t, "psKey", ts.Peer[0].PresharedKey)
			assert.Equal(t, "10.2.0.2/32", ts.Peer[0].AllowedIPs[0])
			assert.Equal(t, "fd00:2::2/128", ts.Peer[0].AllowedIPs[1])

			assert.Equal(t, "pubClientKey2", ts.Peer[1].PublicKey)
			assert.Equal(t, "psKey2", ts.Peer[1].PresharedKey)
			assert.Equal(t, "10.2.0.3/32", ts.Peer[1].AllowedIPs[0])
			assert.Equal(t, "fd00:2::3/128", ts.Peer[1].AllowedIPs[1])
		})

		t.Run("map non unique section to struct", func(t *testing.T) {
			newPeer := new(testPeer)
			newPeerSlice := make([]testPeer, 0)

			f, err := LoadSources(LoadOptions{AllowNonUniqueSections: true}, []byte(confNonUniqueSectionDataStruct))
			require.NoError(t, err)

			// try only first one
			assert.NoError(t, f.Section("Peer").MapTo(newPeer))
			assert.Equal(t, "pubClientKey", newPeer.PublicKey)
			assert.Equal(t, "psKey", newPeer.PresharedKey)
			assert.Equal(t, "10.2.0.2/32", newPeer.AllowedIPs[0])
			assert.Equal(t, "fd00:2::2/128", newPeer.AllowedIPs[1])

			// try all
			assert.NoError(t, f.Section("Peer").MapTo(&newPeerSlice))
			assert.Equal(t, "pubClientKey", newPeerSlice[0].PublicKey)
			assert.Equal(t, "psKey", newPeerSlice[0].PresharedKey)
			assert.Equal(t, "10.2.0.2/32", newPeerSlice[0].AllowedIPs[0])
			assert.Equal(t, "fd00:2::2/128", newPeerSlice[0].AllowedIPs[1])

			assert.Equal(t, "pubClientKey2", newPeerSlice[1].PublicKey)
			assert.Equal(t, "psKey2", newPeerSlice[1].PresharedKey)
			assert.Equal(t, "10.2.0.3/32", newPeerSlice[1].AllowedIPs[0])
			assert.Equal(t, "fd00:2::3/128", newPeerSlice[1].AllowedIPs[1])
		})

		t.Run("map non unique sections with subsections to struct", func(t *testing.T) {
			iniFile, err := LoadSources(LoadOptions{AllowNonUniqueSections: true}, strings.NewReader(`
[Section]
FieldInSubSection = 1
FieldInSubSection2 = 2
FieldInSection = 3

[Section]
FieldInSubSection = 4
FieldInSubSection2 = 5
FieldInSection = 6
`))
			require.NoError(t, err)

			type SubSection struct {
				FieldInSubSection string `ini:"FieldInSubSection"`
			}
			type SubSection2 struct {
				FieldInSubSection2 string `ini:"FieldInSubSection2"`
			}

			type Section struct {
				SubSection     `ini:"Section"`
				SubSection2    `ini:"Section"`
				FieldInSection string `ini:"FieldInSection"`
			}

			type File struct {
				Sections []Section `ini:"Section,nonunique"`
			}

			f := new(File)
			err = iniFile.MapTo(f)
			require.NoError(t, err)

			assert.Equal(t, "1", f.Sections[0].FieldInSubSection)
			assert.Equal(t, "2", f.Sections[0].FieldInSubSection2)
			assert.Equal(t, "3", f.Sections[0].FieldInSection)

			assert.Equal(t, "4", f.Sections[1].FieldInSubSection)
			assert.Equal(t, "5", f.Sections[1].FieldInSubSection2)
			assert.Equal(t, "6", f.Sections[1].FieldInSection)
		})
	})
}

func Test_ReflectFromStruct(t *testing.T) {
	t.Run("reflect from struct", func(t *testing.T) {
		type Embeded struct {
			Dates       []time.Time `delim:"|" comment:"Time data"`
			Places      []string
			Years       []int
			Numbers     []int64
			Ages        []uint
			Populations []uint64
			Coordinates []float64
			Flags       []bool
			None        []int
		}
		type Author struct {
			Name      string `ini:"NAME"`
			Male      bool
			Optional  *bool
			Age       int `comment:"Author's age"`
			Height    uint
			GPA       float64
			Date      time.Time
			NeverMind string `ini:"-"`
			ignored   string
			*Embeded  `ini:"infos" comment:"Embeded section"`
		}

		ti, err := time.Parse(time.RFC3339, "1993-10-07T20:17:05Z")
		require.NoError(t, err)
		a := &Author{"Unknwon", true, nil, 21, 100, 2.8, ti, "", "ignored",
			&Embeded{
				[]time.Time{ti, ti},
				[]string{"HangZhou", "Boston"},
				[]int{1993, 1994},
				[]int64{10010, 10086},
				[]uint{18, 19},
				[]uint64{12345678, 98765432},
				[]float64{192.168, 10.11},
				[]bool{true, false},
				[]int{},
			}}
		cfg := Empty()
		assert.NoError(t, ReflectFrom(cfg, a))

		var buf bytes.Buffer
		_, err = cfg.WriteTo(&buf)
		require.NoError(t, err)
		assert.Equal(t, `NAME     = Unknwon
Male     = true
Optional = 
; Author's age
Age      = 21
Height   = 100
GPA      = 2.8
Date     = 1993-10-07T20:17:05Z

; Embeded section
[infos]
; Time data
Dates       = 1993-10-07T20:17:05Z|1993-10-07T20:17:05Z
Places      = HangZhou,Boston
Years       = 1993,1994
Numbers     = 10010,10086
Ages        = 18,19
Populations = 12345678,98765432
Coordinates = 192.168,10.11
Flags       = true,false
None        = 
`,
			buf.String(),
		)

		t.Run("reflect from non-point struct", func(t *testing.T) {
			assert.Error(t, ReflectFrom(cfg, Author{}))
		})

		t.Run("reflect from struct with omitempty", func(t *testing.T) {
			cfg := Empty()
			type SpecialStruct struct {
				FirstName  string    `ini:"first_name"`
				LastName   string    `ini:"last_name,omitempty"`
				JustOmitMe string    `ini:"omitempty"`
				LastLogin  time.Time `ini:"last_login,omitempty"`
				LastLogin2 time.Time `ini:",omitempty"`
				NotEmpty   int       `ini:"omitempty"`
				Number     int64     `ini:",omitempty"`
				Ages       uint      `ini:",omitempty"`
				Population uint64    `ini:",omitempty"`
				Coordinate float64   `ini:",omitempty"`
				Flag       bool      `ini:",omitempty"`
				Note       *string   `ini:",omitempty"`
			}
			special := &SpecialStruct{
				FirstName: "John",
				LastName:  "Doe",
				NotEmpty:  9,
			}

			assert.NoError(t, ReflectFrom(cfg, special))

			var buf bytes.Buffer
			_, err = cfg.WriteTo(&buf)
			assert.Equal(t, `first_name = John
last_name  = Doe
omitempty  = 9
`,
				buf.String(),
			)
		})

		t.Run("reflect from struct with non-anonymous structure pointer", func(t *testing.T) {
			cfg := Empty()
			type Rpc struct {
				Enable  bool   `ini:"enable"`
				Type    string `ini:"type"`
				Address string `ini:"addr"`
				Name    string `ini:"name"`
			}
			type Cfg struct {
				Rpc *Rpc `ini:"rpc"`
			}

			config := &Cfg{
				Rpc: &Rpc{
					Enable:  true,
					Type:    "type",
					Address: "address",
					Name:    "name",
				},
			}
			assert.NoError(t, cfg.ReflectFrom(config))

			var buf bytes.Buffer
			_, err = cfg.WriteTo(&buf)
			assert.Equal(t, `[rpc]
enable = true
type   = type
addr   = address
name   = name
`,
				buf.String(),
			)
		})
	})
}

func Test_ReflectFromStructNonUniqueSections(t *testing.T) {
	t.Run("reflect from struct with non unique sections", func(t *testing.T) {
		nonUnique := &testNonUniqueSectionsStruct{
			Interface: testInterface{
				Address:    "10.2.0.1/24",
				ListenPort: 34777,
				PrivateKey: "privServerKey",
			},
			Peer: []testPeer{
				{
					PublicKey:    "pubClientKey",
					PresharedKey: "psKey",
					AllowedIPs:   []string{"10.2.0.2/32,fd00:2::2/128"},
				},
				{
					PublicKey:    "pubClientKey2",
					PresharedKey: "psKey2",
					AllowedIPs:   []string{"10.2.0.3/32,fd00:2::3/128"},
				},
			},
		}

		cfg := Empty(LoadOptions{
			AllowNonUniqueSections: true,
		})

		assert.NoError(t, ReflectFrom(cfg, nonUnique))

		var buf bytes.Buffer
		_, err := cfg.WriteTo(&buf)
		require.NoError(t, err)
		assert.Equal(t, confNonUniqueSectionDataStruct, buf.String())

		// note: using ReflectFrom from should overwrite the existing sections
		err = cfg.Section("Peer").ReflectFrom([]*testPeer{
			{
				PublicKey:    "pubClientKey3",
				PresharedKey: "psKey3",
				AllowedIPs:   []string{"10.2.0.4/32,fd00:2::4/128"},
			},
			{
				PublicKey:    "pubClientKey4",
				PresharedKey: "psKey4",
				AllowedIPs:   []string{"10.2.0.5/32,fd00:2::5/128"},
			},
		})

		require.NoError(t, err)

		buf = bytes.Buffer{}
		_, err = cfg.WriteTo(&buf)
		require.NoError(t, err)
		assert.Equal(t, `[Interface]
Address    = 10.2.0.1/24
ListenPort = 34777
PrivateKey = privServerKey

[Peer]
PublicKey    = pubClientKey3
PresharedKey = psKey3
AllowedIPs   = 10.2.0.4/32,fd00:2::4/128

[Peer]
PublicKey    = pubClientKey4
PresharedKey = psKey4
AllowedIPs   = 10.2.0.5/32,fd00:2::5/128
`,
			buf.String(),
		)

		// note: using ReflectFrom from should overwrite the existing sections
		err = cfg.Section("Peer").ReflectFrom(&testPeer{
			PublicKey:    "pubClientKey5",
			PresharedKey: "psKey5",
			AllowedIPs:   []string{"10.2.0.6/32,fd00:2::6/128"},
		})

		require.NoError(t, err)

		buf = bytes.Buffer{}
		_, err = cfg.WriteTo(&buf)
		require.NoError(t, err)
		assert.Equal(t, `[Interface]
Address    = 10.2.0.1/24
ListenPort = 34777
PrivateKey = privServerKey

[Peer]
PublicKey    = pubClientKey5
PresharedKey = psKey5
AllowedIPs   = 10.2.0.6/32,fd00:2::6/128
`,
			buf.String(),
		)
	})
}

// Inspired by https://github.com/go-ini/ini/issues/196
func TestMapToAndReflectFromStructWithShadows(t *testing.T) {
	t.Run("map to struct and then reflect with shadows should generate original config content", func(t *testing.T) {
		type include struct {
			Paths []string `ini:"path,omitempty,allowshadow"`
		}

		cfg, err := LoadSources(LoadOptions{
			AllowShadows: true,
		}, []byte(`
[include]
path = /tmp/gpm-profiles/test5.profile
path = /tmp/gpm-profiles/test1.profile`))
		require.NoError(t, err)

		sec := cfg.Section("include")
		inc := new(include)
		err = sec.MapTo(inc)
		require.NoError(t, err)

		err = sec.ReflectFrom(inc)
		require.NoError(t, err)

		var buf bytes.Buffer
		_, err = cfg.WriteTo(&buf)
		require.NoError(t, err)
		assert.Equal(t, `[include]
path = /tmp/gpm-profiles/test5.profile
path = /tmp/gpm-profiles/test1.profile
`,
			buf.String(),
		)

		t.Run("reflect from struct with shadows", func(t *testing.T) {
			cfg := Empty(LoadOptions{
				AllowShadows: true,
			})
			type ShadowStruct struct {
				StringArray      []string    `ini:"sa,allowshadow"`
				EmptyStringArrat []string    `ini:"empty,omitempty,allowshadow"`
				Allowshadow      []string    `ini:"allowshadow,allowshadow"`
				Dates            []time.Time `ini:",allowshadow"`
				Places           []string    `ini:",allowshadow"`
				Years            []int       `ini:",allowshadow"`
				Numbers          []int64     `ini:",allowshadow"`
				Ages             []uint      `ini:",allowshadow"`
				Populations      []uint64    `ini:",allowshadow"`
				Coordinates      []float64   `ini:",allowshadow"`
				Flags            []bool      `ini:",allowshadow"`
				None             []int       `ini:",allowshadow"`
			}

			shadow := &ShadowStruct{
				StringArray: []string{"s1", "s2"},
				Allowshadow: []string{"s3", "s4"},
				Dates: []time.Time{time.Date(2020, 9, 12, 00, 00, 00, 651387237, time.UTC),
					time.Date(2020, 9, 12, 00, 00, 00, 651387237, time.UTC)},
				Places:      []string{"HangZhou", "Boston"},
				Years:       []int{1993, 1994},
				Numbers:     []int64{10010, 10086},
				Ages:        []uint{18, 19},
				Populations: []uint64{12345678, 98765432},
				Coordinates: []float64{192.168, 10.11},
				Flags:       []bool{true, false},
				None:        []int{},
			}

			assert.NoError(t, ReflectFrom(cfg, shadow))

			var buf bytes.Buffer
			_, err := cfg.WriteTo(&buf)
			require.NoError(t, err)
			assert.Equal(t, `sa          = s1
sa          = s2
allowshadow = s3
allowshadow = s4
Dates       = 2020-09-12T00:00:00Z
Places      = HangZhou
Places      = Boston
Years       = 1993
Years       = 1994
Numbers     = 10010
Numbers     = 10086
Ages        = 18
Ages        = 19
Populations = 12345678
Populations = 98765432
Coordinates = 192.168
Coordinates = 10.11
Flags       = true
Flags       = false
None        = 
`,
				buf.String(),
			)
		})
	})
}

type testMapper struct {
	PackageName string
}

func Test_NameGetter(t *testing.T) {
	t.Run("test name mappers", func(t *testing.T) {
		assert.NoError(t, MapToWithMapper(&testMapper{}, TitleUnderscore, []byte("packag_name=ini")))

		cfg, err := Load([]byte("PACKAGE_NAME=ini"))
		require.NoError(t, err)
		require.NotNil(t, cfg)

		cfg.NameMapper = SnackCase
		tg := new(testMapper)
		assert.NoError(t, cfg.MapTo(tg))
		assert.Equal(t, "ini", tg.PackageName)
	})
}

type testDurationStruct struct {
	Duration time.Duration `ini:"Duration"`
}

func Test_Duration(t *testing.T) {
	t.Run("duration less than 16m50s", func(t *testing.T) {
		ds := new(testDurationStruct)
		assert.NoError(t, MapTo(ds, []byte("Duration=16m49s")))

		dur, err := time.ParseDuration("16m49s")
		require.NoError(t, err)
		assert.Equal(t, dur.Seconds(), ds.Duration.Seconds())
	})
}

type Employer struct {
	Name  string
	Title string
}

type Employers []*Employer

func (es Employers) ReflectINIStruct(f *File) error {
	for _, e := range es {
		f.Section(e.Name).Key("Title").SetValue(e.Title)
	}
	return nil
}

// Inspired by https://github.com/go-ini/ini/issues/199
func Test_StructReflector(t *testing.T) {
	t.Run("reflect with StructReflector interface", func(t *testing.T) {
		p := &struct {
			FirstName string
			Employer  Employers
		}{
			FirstName: "Andrew",
			Employer: []*Employer{
				{
					Name:  `Employer "VMware"`,
					Title: "Staff II Engineer",
				},
				{
					Name:  `Employer "EMC"`,
					Title: "Consultant Engineer",
				},
			},
		}

		f := Empty()
		assert.NoError(t, f.ReflectFrom(p))

		var buf bytes.Buffer
		_, err := f.WriteTo(&buf)
		require.NoError(t, err)

		assert.Equal(t, `FirstName = Andrew

[Employer "VMware"]
Title = Staff II Engineer

[Employer "EMC"]
Title = Consultant Engineer
`,
			buf.String(),
		)
	})
}
