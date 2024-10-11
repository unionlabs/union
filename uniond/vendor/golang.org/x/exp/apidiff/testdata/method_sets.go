package p

// old
type SM struct {
	embedm
	Embedm
}

func (SM) V1() {}
func (SM) V2() {}
func (SM) V3() {}
func (SM) V4() {}
func (SM) v()  {}

func (*SM) P1() {}
func (*SM) P2() {}
func (*SM) P3() {}
func (*SM) P4() {}
func (*SM) p()  {}

type embedm int

func (embedm) EV1()  {}
func (embedm) EV2()  {}
func (embedm) EV3()  {}
func (*embedm) EP1() {}
func (*embedm) EP2() {}
func (*embedm) EP3() {}

type Embedm struct {
	A int
}

func (Embedm) FV()  {}
func (*Embedm) FP() {}

type RepeatEmbedm struct {
	Embedm
}

// new
type SM struct {
	embedm2
	embedm3
	Embedm
	// i SM.A: changed from int to bool
}

// c SMa: added
type SMa = SM

func (SM) V1() {} // OK: same

// func (SM) V2() {}
// i SM.V2: removed

// i SM.V3: changed from func() to func(int)
func (SM) V3(int) {}

// c SM.V5: added
func (SM) V5() {}

func (SM) v(int) {} // OK: unexported method change
func (SM) v2()   {} // OK: unexported method added

func (*SM) P1() {} // OK: same
//func (*SM) P2() {}
// i (*SM).P2: removed

// i (*SM).P3: changed from func() to func(int)
func (*SMa) P3(int) {}

// c (*SM).P5: added
func (*SM) P5() {}

// func (*SM) p() {}  // OK: unexported method removed

// Changing from a value to a pointer receiver or vice versa
// just looks like adding and removing a method.

// i SM.V4: removed
// i (*SM).V4: changed from func() to func(int)
func (*SM) V4(int) {}

// c SM.P4: added
// P4 is not removed from (*SM) because value methods
// are in the pointer method set.
func (SM) P4() {}

type embedm2 int

// i embedm.EV1: changed from func() to func(int)
func (embedm2) EV1(int) {}

// i embedm.EV2, method set of SM: removed
// i embedm.EV2, method set of *SM: removed

// i (*embedm).EP2, method set of *SM: removed
func (*embedm2) EP1() {}

type embedm3 int

func (embedm3) EV3()  {} // OK: compatible with old embedm.EV3
func (*embedm3) EP3() {} // OK: compatible with old (*embedm).EP3

type Embedm struct {
	// i Embedm.A: changed from int to bool
	A bool
}

// i Embedm.FV: changed from func() to func(int)
func (Embedm) FV(int) {}
func (*Embedm) FP()   {}

type RepeatEmbedm struct {
	// i RepeatEmbedm.A: changed from int to bool
	Embedm
}
