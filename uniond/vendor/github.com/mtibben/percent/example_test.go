package percent

import (
	"fmt"
)

func Example() {
	percentEncoded := Encode("a.b/c d", "/. ")
	fmt.Println(percentEncoded)
	fmt.Println(Decode(percentEncoded))
	// Output: a%2Eb%2Fc%20d
	// a.b/c d
}
