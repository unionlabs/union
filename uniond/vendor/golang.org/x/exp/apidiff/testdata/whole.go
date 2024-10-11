package p

// Whole-package interface satisfaction

// old
type WI1 interface {
	M1()
	m1()
}

type WI2 interface {
	M2()
	m2()
}

type WS1 int

func (WS1) M1() {}
func (WS1) m1() {}

type WS2 int

func (WS2) M2() {}
func (WS2) m2() {}

// new
type WI1 interface {
	M1()
	m()
}

type WS1 int

func (WS1) M1() {}

// i WS1: no longer implements WI1
//func (WS1) m1() {}

type WI2 interface {
	M2()
	m2()
	// i WS2: no longer implements WI2
	m3()
}

type WS2 int

func (WS2) M2() {}
func (WS2) m2() {}
