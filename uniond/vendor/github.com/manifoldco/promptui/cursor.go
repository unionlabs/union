package promptui

import (
	"fmt"
	"strings"
)

// Pointer is A specific type that translates a given set of runes into a given
// set of runes pointed at by the cursor.
type Pointer func(to []rune) []rune

func defaultCursor(ignored []rune) []rune {
	return []rune("\u2588")
}

func blockCursor(input []rune) []rune {
	return []rune(fmt.Sprintf("\\e[7m%s\\e[0m", string(input)))
}

func pipeCursor(input []rune) []rune {
	marker := []rune("|")
	out := []rune{}
	out = append(out, marker...)
	out = append(out, input...)
	return out
}

var (
	// DefaultCursor is a big square block character. Obscures whatever was
	// input.
	DefaultCursor Pointer = defaultCursor
	// BlockCursor is a cursor which highlights a character by inverting colors
	// on it.
	BlockCursor Pointer = blockCursor
	// PipeCursor is a pipe character "|" which appears before the input
	// character.
	PipeCursor Pointer = pipeCursor
)

// Cursor tracks the state associated with the movable cursor
// The strategy is to keep the prompt, input pristine except for requested
// modifications. The insertion of the cursor happens during a `format` call
// and we read in new input via an `Update` call
type Cursor struct {
	// shows where the user inserts/updates text
	Cursor Pointer
	// what the user entered, and what we will echo back to them, after
	// insertion of the cursor and prefixing with the prompt
	input []rune
	// Put the cursor before this slice
	Position int
	erase    bool
}

// NewCursor create a new cursor, with the DefaultCursor, the specified input,
// and position at the end of the specified starting input.
func NewCursor(startinginput string, pointer Pointer, eraseDefault bool) Cursor {
	if pointer == nil {
		pointer = defaultCursor
	}
	cur := Cursor{Cursor: pointer, Position: len(startinginput), input: []rune(startinginput), erase: eraseDefault}
	if eraseDefault {
		cur.Start()
	} else {
		cur.End()
	}
	return cur
}

func (c *Cursor) String() string {
	return fmt.Sprintf(
		"Cursor: %s, input %s, Position %d",
		string(c.Cursor([]rune(""))), string(c.input), c.Position)
}

// End is a convenience for c.Place(len(c.input)) so you don't have to know how I
// indexed.
func (c *Cursor) End() {
	c.Place(len(c.input))
}

// Start is convenience for c.Place(0) so you don't have to know how I
// indexed.
func (c *Cursor) Start() {
	c.Place(0)
}

// ensures we are in bounds.
func (c *Cursor) correctPosition() {
	if c.Position > len(c.input) {
		c.Position = len(c.input)
	}

	if c.Position < 0 {
		c.Position = 0
	}
}

// insert the cursor rune array into r before the provided index
func format(a []rune, c *Cursor) string {
	i := c.Position
	var b []rune

	out := make([]rune, 0)
	if i < len(a) {
		b = c.Cursor(a[i : i+1])
		out = append(out, a[:i]...)   // does not include i
		out = append(out, b...)       // add the cursor
		out = append(out, a[i+1:]...) // add the rest after i
	} else {
		b = c.Cursor([]rune{})
		out = append(out, a...)
		out = append(out, b...)
	}
	return string(out)
}

// Format renders the input with the Cursor appropriately positioned.
func (c *Cursor) Format() string {
	r := c.input
	// insert the cursor
	return format(r, c)
}

// FormatMask replaces all input runes with the mask rune.
func (c *Cursor) FormatMask(mask rune) string {
	if mask == ' ' {
		return format([]rune{}, c)
	}

	r := make([]rune, len(c.input))
	for i := range r {
		r[i] = mask
	}
	return format(r, c)
}

// Update inserts newinput into the input []rune in the appropriate place.
// The cursor is moved to the end of the inputed sequence.
func (c *Cursor) Update(newinput string) {
	a := c.input
	b := []rune(newinput)
	i := c.Position
	a = append(a[:i], append(b, a[i:]...)...)
	c.input = a
	c.Move(len(b))
}

// Get returns a copy of the input
func (c *Cursor) Get() string {
	return string(c.input)
}

// GetMask returns a mask string with length equal to the input
func (c *Cursor) GetMask(mask rune) string {
	return strings.Repeat(string(mask), len(c.input))
}

// Replace replaces the previous input with whatever is specified, and moves the
// cursor to the end position
func (c *Cursor) Replace(input string) {
	c.input = []rune(input)
	c.End()
}

// Place moves the cursor to the absolute array index specified by position
func (c *Cursor) Place(position int) {
	c.Position = position
	c.correctPosition()
}

// Move moves the cursor over in relative terms, by shift indices.
func (c *Cursor) Move(shift int) {
	// delete the current cursor
	c.Position = c.Position + shift
	c.correctPosition()
}

// Backspace removes the rune that precedes the cursor
//
// It handles being at the beginning or end of the row, and moves the cursor to
// the appropriate position.
func (c *Cursor) Backspace() {
	a := c.input
	i := c.Position
	if i == 0 {
		// Shrug
		return
	}
	if i == len(a) {
		c.input = a[:i-1]
	} else {
		c.input = append(a[:i-1], a[i:]...)
	}
	// now it's pointing to the i+1th element
	c.Move(-1)
}

// Listen is a readline Listener that updates internal cursor state appropriately.
func (c *Cursor) Listen(line []rune, pos int, key rune) ([]rune, int, bool) {
	if line != nil {
		// no matter what, update our internal representation.
		c.Update(string(line))
	}

	switch key {
	case 0: // empty
	case KeyEnter:
		return []rune(c.Get()), c.Position, false
	case KeyBackspace, KeyCtrlH:
		if c.erase {
			c.erase = false
			c.Replace("")
		}
		c.Backspace()
	case KeyForward:
		// the user wants to edit the default, despite how we set it up. Let
		// them.
		c.erase = false
		c.Move(1)
	case KeyBackward:
		c.Move(-1)
	default:
		if c.erase {
			c.erase = false
			c.Replace("")
			c.Update(string(key))
		}
	}

	return []rune(c.Get()), c.Position, true
}
