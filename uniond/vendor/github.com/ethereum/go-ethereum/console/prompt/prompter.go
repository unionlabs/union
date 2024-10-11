// Copyright 2016 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package prompt

import (
	"fmt"
	"strings"

	"github.com/peterh/liner"
)

// Stdin holds the stdin line reader (also using stdout for printing prompts).
// Only this reader may be used for input because it keeps an internal buffer.
var Stdin = newTerminalPrompter()

// UserPrompter defines the methods needed by the console to prompt the user for
// various types of inputs.
type UserPrompter interface {
	// PromptInput displays the given prompt to the user and requests some textual
	// data to be entered, returning the input of the user.
	PromptInput(prompt string) (string, error)

	// PromptPassword displays the given prompt to the user and requests some textual
	// data to be entered, but one which must not be echoed out into the terminal.
	// The method returns the input provided by the user.
	PromptPassword(prompt string) (string, error)

	// PromptConfirm displays the given prompt to the user and requests a boolean
	// choice to be made, returning that choice.
	PromptConfirm(prompt string) (bool, error)

	// SetHistory sets the input scrollback history that the prompter will allow
	// the user to scroll back to.
	SetHistory(history []string)

	// AppendHistory appends an entry to the scrollback history. It should be called
	// if and only if the prompt to append was a valid command.
	AppendHistory(command string)

	// ClearHistory clears the entire history
	ClearHistory()

	// SetWordCompleter sets the completion function that the prompter will call to
	// fetch completion candidates when the user presses tab.
	SetWordCompleter(completer WordCompleter)
}

// WordCompleter takes the currently edited line with the cursor position and
// returns the completion candidates for the partial word to be completed. If
// the line is "Hello, wo!!!" and the cursor is before the first '!', ("Hello,
// wo!!!", 9) is passed to the completer which may returns ("Hello, ", {"world",
// "Word"}, "!!!") to have "Hello, world!!!".
type WordCompleter func(line string, pos int) (string, []string, string)

// terminalPrompter is a UserPrompter backed by the liner package. It supports
// prompting the user for various input, among others for non-echoing password
// input.
type terminalPrompter struct {
	*liner.State
	warned     bool
	supported  bool
	normalMode liner.ModeApplier
	rawMode    liner.ModeApplier
}

// newTerminalPrompter creates a liner based user input prompter working off the
// standard input and output streams.
func newTerminalPrompter() *terminalPrompter {
	p := new(terminalPrompter)
	// Get the original mode before calling NewLiner.
	// This is usually regular "cooked" mode where characters echo.
	normalMode, _ := liner.TerminalMode()
	// Turn on liner. It switches to raw mode.
	p.State = liner.NewLiner()
	rawMode, err := liner.TerminalMode()
	if err != nil || !liner.TerminalSupported() {
		p.supported = false
	} else {
		p.supported = true
		p.normalMode = normalMode
		p.rawMode = rawMode
		// Switch back to normal mode while we're not prompting.
		normalMode.ApplyMode()
	}
	p.SetCtrlCAborts(true)
	p.SetTabCompletionStyle(liner.TabPrints)
	p.SetMultiLineMode(true)
	return p
}

// PromptInput displays the given prompt to the user and requests some textual
// data to be entered, returning the input of the user.
func (p *terminalPrompter) PromptInput(prompt string) (string, error) {
	if p.supported {
		p.rawMode.ApplyMode()
		defer p.normalMode.ApplyMode()
	} else {
		// liner tries to be smart about printing the prompt
		// and doesn't print anything if input is redirected.
		// Un-smart it by printing the prompt always.
		fmt.Print(prompt)
		prompt = ""
		defer fmt.Println()
	}
	return p.State.Prompt(prompt)
}

// PromptPassword displays the given prompt to the user and requests some textual
// data to be entered, but one which must not be echoed out into the terminal.
// The method returns the input provided by the user.
func (p *terminalPrompter) PromptPassword(prompt string) (passwd string, err error) {
	if p.supported {
		p.rawMode.ApplyMode()
		defer p.normalMode.ApplyMode()
		return p.State.PasswordPrompt(prompt)
	}
	if !p.warned {
		fmt.Println("!! Unsupported terminal, password will be echoed.")
		p.warned = true
	}
	// Just as in Prompt, handle printing the prompt here instead of relying on liner.
	fmt.Print(prompt)
	passwd, err = p.State.Prompt("")
	fmt.Println()
	return passwd, err
}

// PromptConfirm displays the given prompt to the user and requests a boolean
// choice to be made, returning that choice.
func (p *terminalPrompter) PromptConfirm(prompt string) (bool, error) {
	input, err := p.Prompt(prompt + " [y/n] ")
	if len(input) > 0 && strings.EqualFold(input[:1], "y") {
		return true, nil
	}
	return false, err
}

// SetHistory sets the input scrollback history that the prompter will allow
// the user to scroll back to.
func (p *terminalPrompter) SetHistory(history []string) {
	p.State.ReadHistory(strings.NewReader(strings.Join(history, "\n")))
}

// AppendHistory appends an entry to the scrollback history.
func (p *terminalPrompter) AppendHistory(command string) {
	p.State.AppendHistory(command)
}

// ClearHistory clears the entire history
func (p *terminalPrompter) ClearHistory() {
	p.State.ClearHistory()
}

// SetWordCompleter sets the completion function that the prompter will call to
// fetch completion candidates when the user presses tab.
func (p *terminalPrompter) SetWordCompleter(completer WordCompleter) {
	p.State.SetWordCompleter(liner.WordCompleter(completer))
}
