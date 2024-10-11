package main

import (
	"fmt"

	"github.com/muesli/termenv"
)

func main() {
	restoreConsole, err := termenv.EnableVirtualTerminalProcessing(termenv.DefaultOutput())
	if err != nil {
		panic(err)
	}
	defer restoreConsole()

	// Basic ANSI colors 0 - 15
	fmt.Println(termenv.String("Basic ANSI colors").Bold())

	p := termenv.ANSI
	for i := int64(0); i < 16; i++ {
		if i%8 == 0 {
			fmt.Println()
		}

		// background color
		bg := p.Color(fmt.Sprintf("%d", i))
		c := termenv.ConvertToRGB(bg)

		out := termenv.String(fmt.Sprintf(" %2d %s ", i, c.Hex()))

		// apply colors
		if i < 5 {
			out = out.Foreground(p.Color("7"))
		} else {
			out = out.Foreground(p.Color("0"))
		}
		out = out.Background(bg)

		fmt.Print(out.String()[:])
	}
	fmt.Printf("\n\n")

	// Extended ANSI colors 16-231
	fmt.Println(termenv.String("Extended ANSI colors").Bold())

	p = termenv.ANSI256
	for i := int64(16); i < 232; i++ {
		if (i-16)%6 == 0 {
			fmt.Println()
		}

		// background color
		bg := p.Color(fmt.Sprintf("%d", i))
		c := termenv.ConvertToRGB(bg)

		out := termenv.String(fmt.Sprintf(" %3d %s ", i, c.Hex()))

		// apply colors
		if i < 28 {
			out = out.Foreground(p.Color("7"))
		} else {
			out = out.Foreground(p.Color("0"))
		}
		out = out.Background(bg)

		fmt.Print(out.String()[:])
	}
	fmt.Printf("\n\n")

	// Grayscale ANSI colors 232-255
	fmt.Println(termenv.String("Extended ANSI Grayscale").Bold())

	p = termenv.ANSI256
	for i := int64(232); i < 256; i++ {
		if (i-232)%6 == 0 {
			fmt.Println()
		}

		// background color
		bg := p.Color(fmt.Sprintf("%d", i))
		c := termenv.ConvertToRGB(bg)

		out := termenv.String(fmt.Sprintf(" %3d %s ", i, c.Hex()))

		// apply colors
		if i < 244 {
			out = out.Foreground(p.Color("7"))
		} else {
			out = out.Foreground(p.Color("0"))
		}
		out = out.Background(bg)

		fmt.Print(out.String()[:])
	}
	fmt.Printf("\n\n")
}
