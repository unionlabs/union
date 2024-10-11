package colltest

import "testing"

type animal interface {
	name() string
}

type dog struct {
	Name        string `json:"name"`
	BarksLoudly bool   `json:"barks_loudly"`
}

type cat struct {
	Name      string `json:"name"`
	Scratches bool   `json:"scratches"`
}

func (d *cat) name() string { return d.Name }

func (d dog) name() string { return d.Name }

func TestMockValueCodec(t *testing.T) {
	t.Run("primitive type", func(t *testing.T) {
		x := MockValueCodec[string]()
		TestValueCodec(t, x, "hello")
	})

	t.Run("struct type", func(t *testing.T) {
		x := MockValueCodec[dog]()
		TestValueCodec(t, x, dog{
			Name:        "kernel",
			BarksLoudly: true,
		})
	})

	t.Run("interface type", func(t *testing.T) {
		x := MockValueCodec[animal]()
		TestValueCodec[animal](t, x, dog{
			Name:        "kernel",
			BarksLoudly: true,
		})
		TestValueCodec[animal](t, x, &cat{
			Name:      "echo",
			Scratches: true,
		})
	})
}
