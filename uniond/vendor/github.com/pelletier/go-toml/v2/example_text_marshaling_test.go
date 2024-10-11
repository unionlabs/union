package toml_test

import (
	"fmt"
	"log"
	"strconv"

	"github.com/pelletier/go-toml/v2"
)

type customInt int

func (i *customInt) UnmarshalText(b []byte) error {
	x, err := strconv.ParseInt(string(b), 10, 32)
	if err != nil {
		return err
	}
	*i = customInt(x * 100)
	return nil
}

type doc struct {
	Value customInt
}

func ExampleUnmarshal_textUnmarshal() {
	var x doc

	data := []byte(`value  = "42"`)
	err := toml.Unmarshal(data, &x)
	if err != nil {
		log.Fatal(err)
	}
	fmt.Println(x)
	// Output:
	// {4200}
}
