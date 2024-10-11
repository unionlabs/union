// This is a small example using readline to read a password
// and check it's strength while typing using the zxcvbn library.
// Depending on the strength the prompt is colored nicely to indicate strength.
//
// This file is licensed under the WTFPL:
//
//         DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
//                     Version 2, December 2004
//
//  Copyright (C) 2004 Sam Hocevar <sam@hocevar.net>
//
//  Everyone is permitted to copy and distribute verbatim or modified
//  copies of this license document, and changing it is allowed as long
//  as the name is changed.
//
//             DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
//    TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION
//
//   0. You just DO WHAT THE FUCK YOU WANT TO.
package main

import (
	"fmt"

	"github.com/chzyer/readline"
)

const (
	Cyan          = 36
	Green         = 32
	Magenta       = 35
	Red           = 31
	Yellow        = 33
	BackgroundRed = 41
)

// Reset sequence
var ColorResetEscape = "\033[0m"

// ColorResetEscape translates a ANSI color number to a color escape.
func ColorEscape(color int) string {
	return fmt.Sprintf("\033[0;%dm", color)
}

// Colorize the msg using ANSI color escapes
func Colorize(msg string, color int) string {
	return ColorEscape(color) + msg + ColorResetEscape
}

func createStrengthPrompt(password []rune) string {
	symbol, color := "", Red

	switch {
	case len(password) <= 1:
		symbol = "✗"
		color = Red
	case len(password) <= 3:
		symbol = "⚡"
		color = Magenta
	case len(password) <= 5:
		symbol = "⚠"
		color = Yellow
	default:
		symbol = "✔"
		color = Green
	}

	prompt := Colorize(symbol, color)
	prompt += Colorize(" ENT", Cyan)

	prompt += Colorize(" New Password: ", color)
	return prompt
}

func main() {
	rl, err := readline.New("")
	if err != nil {
		return
	}
	defer rl.Close()

	setPasswordCfg := rl.GenPasswordConfig()
	setPasswordCfg.SetListener(func(line []rune, pos int, key rune) (newLine []rune, newPos int, ok bool) {
		rl.SetPrompt(createStrengthPrompt(line))
		rl.Refresh()
		return nil, 0, false
	})

	pswd, err := rl.ReadPasswordWithConfig(setPasswordCfg)
	if err != nil {
		return
	}

	fmt.Println("Your password was:", string(pswd))
}
