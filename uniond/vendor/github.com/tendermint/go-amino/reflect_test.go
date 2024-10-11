package amino

import (
	"math/rand"
	"reflect"
	"runtime/debug"
	"testing"
	"time"

	"github.com/davecgh/go-spew/spew"
	fuzz "github.com/google/gofuzz"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/tendermint/go-amino/tests"
)

//-------------------------------------
// Non-interface Google fuzz tests

func TestCodecStruct(t *testing.T) {
	for _, ptr := range tests.StructTypes {
		rt := getTypeFromPointer(ptr)
		name := rt.Name()
		t.Run(name+":binary", func(t *testing.T) { _testCodec(t, rt, "binary") })
		t.Run(name+":json", func(t *testing.T) { _testCodec(t, rt, "json") })
	}
}

func TestCodecDef(t *testing.T) {
	for _, ptr := range tests.DefTypes {
		rt := getTypeFromPointer(ptr)
		name := rt.Name()
		t.Run(name+":binary", func(t *testing.T) { _testCodec(t, rt, "binary") })
		t.Run(name+":json", func(t *testing.T) { _testCodec(t, rt, "json") })
	}
}

func TestDeepCopyStruct(t *testing.T) {
	for _, ptr := range tests.StructTypes {
		rt := getTypeFromPointer(ptr)
		name := rt.Name()
		t.Run(name+":deepcopy", func(t *testing.T) { _testDeepCopy(t, rt) })
	}
}

func TestDeepCopyDep(t *testing.T) {
	for _, ptr := range tests.DefTypes {
		rt := getTypeFromPointer(ptr)
		name := rt.Name()
		t.Run(name+":deepcopy", func(t *testing.T) { _testDeepCopy(t, rt) })
	}
}

func _testCodec(t *testing.T, rt reflect.Type, codecType string) {

	err := error(nil)
	bz := []byte{}
	cdc := NewCodec()
	f := fuzz.New()
	rv := reflect.New(rt)
	rv2 := reflect.New(rt)
	ptr := rv.Interface()
	ptr2 := rv2.Interface()
	rnd := rand.New(rand.NewSource(10))
	f.RandSource(rnd)
	f.Funcs(fuzzFuncs...)

	defer func() {
		if r := recover(); r != nil {
			t.Fatalf("panic'd:\nreason: %v\n%s\nerr: %v\nbz: %X\nrv: %#v\nrv2: %#v\nptr: %v\nptr2: %v\n",
				r, debug.Stack(), err, bz, rv, rv2, spw(ptr), spw(ptr2),
			)
		}
	}()

	for i := 0; i < 1e4; i++ {
		f.Fuzz(ptr)

		// Reset, which makes debugging decoding easier.
		rv2 = reflect.New(rt)
		ptr2 = rv2.Interface()

		switch codecType {
		case "binary":
			bz, err = cdc.MarshalBinaryBare(ptr)
		case "json":
			bz, err = cdc.MarshalJSON(ptr)
		default:
			panic("should not happen")
		}
		require.Nil(t, err,
			"failed to marshal %v to bytes: %v\n",
			spw(ptr), err)

		switch codecType {
		case "binary":
			err = cdc.UnmarshalBinaryBare(bz, ptr2)
		case "json":
			err = cdc.UnmarshalJSON(bz, ptr2)
		default:
			panic("should not happen")
		}
		require.Nil(t, err,
			"failed to unmarshal bytes %X (%s): %v\nptr: %v\n",
			bz, bz, err, spw(ptr))

		require.Equal(t, ptr, ptr2,
			"end to end failed.\nstart: %v\nend: %v\nbytes: %X\nstring(bytes): %s\n",
			spw(ptr), spw(ptr2), bz, bz)
	}
}

func _testDeepCopy(t *testing.T, rt reflect.Type) {

	err := error(nil)
	f := fuzz.New()
	rv := reflect.New(rt)
	ptr := rv.Interface()
	rnd := rand.New(rand.NewSource(10))
	f.RandSource(rnd)
	f.Funcs(fuzzFuncs...)

	defer func() {
		if r := recover(); r != nil {
			t.Fatalf("panic'd:\nreason: %v\n%s\nerr: %v\nrv: %#v\nptr: %v\n",
				r, debug.Stack(), err, rv, spw(ptr),
			)
		}
	}()

	for i := 0; i < 1e4; i++ {
		f.Fuzz(ptr)

		ptr2 := DeepCopy(ptr)

		require.Equal(t, ptr, ptr2,
			"end to end failed.\nstart: %v\nend: %v\nbytes: %X\nstring(bytes): %s\n",
			spw(ptr), spw(ptr2))
	}
}

//----------------------------------------
// Register/interface tests

func TestCodecBinaryRegister1(t *testing.T) {
	cdc := NewCodec()
	//cdc.RegisterInterface((*tests.Interface1)(nil), nil)
	cdc.RegisterConcrete((*tests.Concrete1)(nil), "Concrete1", nil)

	bz, err := cdc.MarshalBinaryBare(struct{ tests.Interface1 }{tests.Concrete1{}})
	assert.NotNil(t, err, "unregistered interface")
	assert.Empty(t, bz)
}

func TestCodecBinaryRegister2(t *testing.T) {
	cdc := NewCodec()
	cdc.RegisterInterface((*tests.Interface1)(nil), nil)
	cdc.RegisterConcrete((*tests.Concrete1)(nil), "Concrete1", nil)

	bz, err := cdc.MarshalBinaryBare(struct{ tests.Interface1 }{tests.Concrete1{}})
	assert.Nil(t, err, "correctly registered")
	assert.Equal(t, []byte{0xa, 0x4, 0xe3, 0xda, 0xb8, 0x33}, bz,
		"prefix bytes did not match")
}

func TestCodecBinaryRegister3(t *testing.T) {
	cdc := NewCodec()
	cdc.RegisterConcrete((*tests.Concrete1)(nil), "Concrete1", nil)
	cdc.RegisterInterface((*tests.Interface1)(nil), nil)

	bz, err := cdc.MarshalBinaryBare(struct{ tests.Interface1 }{tests.Concrete1{}})
	assert.Nil(t, err, "correctly registered")
	assert.Equal(t, []byte{0xa, 0x4, 0xe3, 0xda, 0xb8, 0x33}, bz,
		"prefix bytes did not match")
}

func TestCodecBinaryRegister4(t *testing.T) {
	cdc := NewCodec()
	cdc.RegisterConcrete((*tests.Concrete1)(nil), "Concrete1", nil)
	cdc.RegisterInterface((*tests.Interface1)(nil), &InterfaceOptions{
		AlwaysDisambiguate: true,
	})

	bz, err := cdc.MarshalBinaryBare(struct{ tests.Interface1 }{tests.Concrete1{}})
	assert.Nil(t, err, "correctly registered")
	assert.Equal(t, []byte{0xa, 0x8, 0x0, 0x12, 0xb5, 0x86, 0xe3, 0xda, 0xb8, 0x33}, bz,
		"prefix bytes did not match")
}

func TestCodecBinaryRegister5(t *testing.T) {
	cdc := NewCodec()
	//cdc.RegisterConcrete((*tests.Concrete1)(nil), "Concrete1", nil)
	cdc.RegisterInterface((*tests.Interface1)(nil), nil)

	bz, err := cdc.MarshalBinaryBare(struct{ tests.Interface1 }{tests.Concrete1{}})
	assert.NotNil(t, err, "concrete type not registered")
	assert.Empty(t, bz)
}

func TestCodecBinaryRegister6(t *testing.T) {
	cdc := NewCodec()
	cdc.RegisterInterface((*tests.Interface1)(nil), nil)
	cdc.RegisterConcrete((*tests.Concrete1)(nil), "Concrete1", nil)

	assert.Panics(t, func() {
		cdc.RegisterConcrete((*tests.Concrete2)(nil), "Concrete1", nil)
	}, "duplicate concrete name")
}

func TestCodecBinaryRegister7(t *testing.T) {
	cdc := NewCodec()
	cdc.RegisterInterface((*tests.Interface1)(nil), nil)
	cdc.RegisterConcrete((*tests.Concrete1)(nil), "Concrete1", nil)
	cdc.RegisterConcrete((*tests.Concrete2)(nil), "Concrete2", nil)

	{ // test tests.Concrete1, no conflict.
		bz, err := cdc.MarshalBinaryBare(struct{ tests.Interface1 }{tests.Concrete1{}})
		assert.Nil(t, err, "correctly registered")
		assert.Equal(t, []byte{0xa, 0x4, 0xe3, 0xda, 0xb8, 0x33}, bz,
			"disfix bytes did not match")
	}

	{ // test tests.Concrete2, no conflict
		bz, err := cdc.MarshalBinaryBare(struct{ tests.Interface1 }{tests.Concrete2{}})
		assert.Nil(t, err, "correctly registered")
		assert.Equal(t, []byte{0xa, 0x4, 0x6a, 0x9, 0xca, 0x1}, bz,
			"disfix bytes did not match")
	}
}

// Serialize and deserialize a non-nil interface value.
func TestCodecBinaryRegister8(t *testing.T) {
	cdc := NewCodec()
	cdc.RegisterInterface((*tests.Interface1)(nil), nil)
	cdc.RegisterConcrete(tests.Concrete3{}, "Concrete3", nil)

	assert.Panics(t, func() {
		cdc.RegisterConcrete(tests.Concrete2{}, "Concrete3", nil)
	}, "duplicate concrete name")

	var c3 tests.Concrete3
	copy(c3[:], []byte("0123"))

	bz, err := cdc.MarshalBinaryBare(c3)
	assert.Nil(t, err)
	assert.Equal(t, []byte{0x53, 0x37, 0x21, 0x1, 0x4, 0x30, 0x31, 0x32, 0x33}, bz,
		"Concrete3 incorrectly serialized")

	var i1 tests.Interface1
	err = cdc.UnmarshalBinaryBare(bz, &i1)
	assert.Nil(t, err)
	assert.Equal(t, c3, i1)
}

// Like TestCodecBinaryRegister8, but JSON.
func TestCodecJSONRegister8(t *testing.T) {
	cdc := NewCodec()
	cdc.RegisterInterface((*tests.Interface1)(nil), nil)
	cdc.RegisterConcrete(tests.Concrete3{}, "Concrete3", nil)

	var c3 tests.Concrete3
	copy(c3[:], []byte("0123"))

	// NOTE: We don't wrap c3...
	// But that's OK, JSON still writes the disfix bytes by default.
	bz, err := cdc.MarshalJSON(c3)
	assert.Nil(t, err)
	assert.Equal(t, []byte(`{"type":"Concrete3","value":"MDEyMw=="}`),
		bz, "Concrete3 incorrectly serialized")

	var i1 tests.Interface1
	err = cdc.UnmarshalJSON(bz, &i1)
	assert.Nil(t, err)
	assert.Equal(t, c3, i1)
}

// Like TestCodecBinaryRegister8, but serialize the concrete value directly.
func TestCodecBinaryRegister9(t *testing.T) {
	cdc := NewCodec()
	cdc.RegisterInterface((*tests.Interface1)(nil), nil)
	cdc.RegisterConcrete(tests.Concrete3{}, "Concrete3", nil)

	assert.Panics(t, func() {
		cdc.RegisterConcrete(tests.Concrete2{}, "Concrete3", nil)
	}, "duplicate concrete name")

	var c3 tests.Concrete3
	copy(c3[:], []byte("0123"))

	bz, err := cdc.MarshalBinaryBare(c3)
	assert.Nil(t, err)
	assert.Equal(t, []byte{0x53, 0x37, 0x21, 0x1, 0x4, 0x30, 0x31, 0x32, 0x33}, bz,
		"Concrete3 incorrectly serialized")

	var i1 tests.Interface1
	err = cdc.UnmarshalBinaryBare(bz, &i1)
	assert.Nil(t, err)
	assert.Equal(t, c3, i1)
}

// Like TestCodecBinaryRegister8 but read into concrete var.
func TestCodecBinaryRegister10(t *testing.T) {
	cdc := NewCodec()
	cdc.RegisterInterface((*tests.Interface1)(nil), nil)
	cdc.RegisterConcrete(tests.Concrete3{}, "Concrete3", nil)

	var c3a tests.Concrete3
	copy(c3a[:], []byte("0123"))

	bz, err := cdc.MarshalBinaryBare(c3a)
	assert.Nil(t, err)
	assert.Equal(t, []byte{0x53, 0x37, 0x21, 0x1, 0x4, 0x30, 0x31, 0x32, 0x33}, bz,
		"Concrete3 incorrectly serialized")

	var c3b tests.Concrete3
	err = cdc.UnmarshalBinaryBare(bz, &c3b)
	assert.Nil(t, err)
	assert.Equal(t, c3a, c3b)
}

func TestCodecBinaryStructFieldNilInterface(t *testing.T) {
	cdc := NewCodec()
	cdc.RegisterInterface((*tests.Interface1)(nil), nil)
	cdc.RegisterConcrete((*tests.InterfaceFieldsStruct)(nil), "interfaceFields", nil)

	i1 := &tests.InterfaceFieldsStruct{F1: new(tests.InterfaceFieldsStruct), F2: nil}
	bz, err := cdc.MarshalBinaryLengthPrefixed(i1)
	assert.Nil(t, err, "unexpected error")

	i2 := new(tests.InterfaceFieldsStruct)
	err = cdc.UnmarshalBinaryLengthPrefixed(bz, i2)

	assert.Nil(t, err, "unexpected error")
	require.Equal(t, i1, i2, "i1 and i2 should be the same after decoding")
}

//----------------------------------------
// Misc.

func spw(o interface{}) string {
	return spew.Sprintf("%#v", o)
}

var fuzzFuncs = []interface{}{
	func(i **int8, c fuzz.Continue) {
		// Prefer nil instead of zero, for deep equality.
		// (go-amino decoder will always prefer nil).
		var i_ int8
		c.Fuzz(&i_)
		if i_ == 0 {
			*i = nil
		} else {
			*i = &i_
		}
	},
	func(i **int16, c fuzz.Continue) {
		// Prefer nil instead of zero, for deep equality.
		// (go-amino decoder will always prefer nil).
		var i_ int16
		c.Fuzz(&i_)
		if i_ == 0 {
			*i = nil
		} else {
			*i = &i_
		}
	},
	func(i **int32, c fuzz.Continue) {
		// Prefer nil instead of zero, for deep equality.
		// (go-amino decoder will always prefer nil).
		var i_ int32
		c.Fuzz(&i_)
		if i_ == 0 {
			*i = nil
		} else {
			*i = &i_
		}
	},
	func(i **int64, c fuzz.Continue) {
		// Prefer nil instead of zero, for deep equality.
		// (go-amino decoder will always prefer nil).
		var i_ int64
		c.Fuzz(&i_)
		if i_ == 0 {
			*i = nil
		} else {
			*i = &i_
		}
	},
	func(i **int, c fuzz.Continue) {
		// Prefer nil instead of zero, for deep equality.
		// (go-amino decoder will always prefer nil).
		var i_ int
		c.Fuzz(&i_)
		if i_ == 0 {
			*i = nil
		} else {
			*i = &i_
		}
	},
	func(ui **uint8, c fuzz.Continue) {
		// Prefer nil instead of zero, for deep equality.
		// (go-amino decoder will always prefer nil).
		var ui_ uint8
		c.Fuzz(&ui_)
		if ui_ == 0 {
			*ui = nil
		} else {
			*ui = &ui_
		}
	},
	func(ptr ***uint8, c fuzz.Continue) {
		// Prefer nil instead of zero, for deep equality.
		// (go-amino decoder will always prefer nil).
		var ui_ uint8
		c.Fuzz(&ui_)
		if ui_ == 0 {
			*ptr = nil
		} else {
			*ptr = new(*uint8)
			**ptr = new(uint8)
			***ptr = ui_
		}
	},
	func(ptr ****uint8, c fuzz.Continue) {
		// Prefer nil instead of zero, for deep equality.
		// (go-amino decoder will always prefer nil).
		var ui_ uint8
		c.Fuzz(&ui_)
		if ui_ == 0 {
			*ptr = nil
		} else {
			*ptr = new(**uint8)
			**ptr = new(*uint8)
			***ptr = new(uint8)
			****ptr = ui_
		}
	},
	func(ui **uint16, c fuzz.Continue) {
		// Prefer nil instead of zero, for deep equality.
		// (go-amino decoder will always prefer nil).
		var ui_ uint16
		c.Fuzz(&ui_)
		if ui_ == 0 {
			*ui = nil
		} else {
			*ui = &ui_
		}
	},
	func(ui **uint32, c fuzz.Continue) {
		// Prefer nil instead of zero, for deep equality.
		// (go-amino decoder will always prefer nil).
		var ui_ uint32
		c.Fuzz(&ui_)
		if ui_ == 0 {
			*ui = nil
		} else {
			*ui = &ui_
		}
	},
	func(ui **uint64, c fuzz.Continue) {
		// Prefer nil instead of zero, for deep equality.
		// (go-amino decoder will always prefer nil).
		var ui_ uint64
		c.Fuzz(&ui_)
		if ui_ == 0 {
			*ui = nil
		} else {
			*ui = &ui_
		}
	},
	func(ui **uint, c fuzz.Continue) {
		// Prefer nil instead of zero, for deep equality.
		// (go-amino decoder will always prefer nil).
		var ui_ uint
		c.Fuzz(&ui_)
		if ui_ == 0 {
			*ui = nil
		} else {
			*ui = &ui_
		}
	},
	func(s **string, c fuzz.Continue) {
		// Prefer nil instead of zero, for deep equality.
		// (go-amino decoder will always prefer nil).
		s_ := randString(c)
		if len(s_) == 0 {
			*s = nil
		} else {
			*s = &s_
		}
	},
	func(bz **[]byte, c fuzz.Continue) {
		// Prefer nil instead of zero, for deep equality.
		// (go-amino decoder will always prefer nil).
		var bz_ []byte
		c.Fuzz(&bz_)
		if len(bz_) == 0 {
			*bz = nil
		} else {
			*bz = &bz_
		}
	},
	func(tyme *time.Time, c fuzz.Continue) {
		// Set time.Unix(_,_) to wipe .wal
		switch c.Intn(4) {
		case 0:
			ns := c.Int63n(10)
			*tyme = time.Unix(0, ns)
		case 1:
			ns := c.Int63n(1e10)
			*tyme = time.Unix(0, ns)
		case 2:
			const maxSeconds = 4611686018 // (1<<63 - 1) / 1e9
			s := c.Int63n(maxSeconds)
			ns := c.Int63n(1e10)
			*tyme = time.Unix(s, ns)
		case 3:
			s := c.Int63n(10)
			ns := c.Int63n(1e10)
			*tyme = time.Unix(s, ns)
		}
		// Strip timezone and monotonic for deep equality.
		// Also set to UTC.
		*tyme = tyme.Truncate(0).UTC()
	},
	func(esz *[]*tests.EmptyStruct, c fuzz.Continue) {
		n := c.Intn(4)
		switch n {
		case 0:
			// Prefer nil over empty slice.
			*esz = nil
		default:
			// Slice of empty struct pointers should be nil,
			// since we don't set amino:"empty_elements".
			*esz = make([]*tests.EmptyStruct, n)
			for i := 0; i < n; i++ {
				(*esz)[i] = nil
			}
		}
	},
}

//----------------------------------------
// From https://github.com/google/gofuzz/blob/master/fuzz.go
// (Apache2.0 License)
// TODO move to tmlibs/common/random.go?

type charRange struct {
	first, last rune
}

// choose returns a random unicode character from the given range, using the
// given randomness source.
func (r *charRange) choose(rand fuzz.Continue) rune {
	count := int64(r.last - r.first)
	return r.first + rune(rand.Int63n(count))
}

var unicodeRanges = []charRange{
	{' ', '~'},           // ASCII characters
	{'\u00a0', '\u02af'}, // Multi-byte encoded characters
	{'\u4e00', '\u9fff'}, // Common CJK (even longer encodings)
}

// randString makes a random string up to 20 characters long. The returned string
// may include a variety of (valid) UTF-8 encodings.
func randString(r fuzz.Continue) string {
	n := r.Intn(19) + 1
	runes := make([]rune, n)
	for i := range runes {
		runes[i] = unicodeRanges[r.Intn(len(unicodeRanges))].choose(r)
	}
	return string(runes)
}
