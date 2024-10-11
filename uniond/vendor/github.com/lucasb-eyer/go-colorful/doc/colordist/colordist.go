package main

import "fmt"
import "github.com/lucasb-eyer/go-colorful"

func main() {
	c1a := colorful.Color{150.0 / 255.0, 10.0 / 255.0, 150.0 / 255.0}
	c1b := colorful.Color{53.0 / 255.0, 10.0 / 255.0, 150.0 / 255.0}
	c2a := colorful.Color{10.0 / 255.0, 150.0 / 255.0, 50.0 / 255.0}
	c2b := colorful.Color{99.9 / 255.0, 150.0 / 255.0, 10.0 / 255.0}

	fmt.Printf("DistanceRgb:       c1: %v\tand c2: %v\n", c1a.DistanceRgb(c1b), c2a.DistanceRgb(c2b))
	fmt.Printf("DistanceLab:       c1: %v\tand c2: %v\n", c1a.DistanceLab(c1b), c2a.DistanceLab(c2b))
	fmt.Printf("DistanceLuv:       c1: %v\tand c2: %v\n", c1a.DistanceLuv(c1b), c2a.DistanceLuv(c2b))
	fmt.Printf("DistanceCIE76:     c1: %v\tand c2: %v\n", c1a.DistanceCIE76(c1b), c2a.DistanceCIE76(c2b))
	fmt.Printf("DistanceCIE94:     c1: %v\tand c2: %v\n", c1a.DistanceCIE94(c1b), c2a.DistanceCIE94(c2b))
	fmt.Printf("DistanceCIEDE2000: c1: %v\tand c2: %v\n", c1a.DistanceCIEDE2000(c1b), c2a.DistanceCIEDE2000(c2b))
}
