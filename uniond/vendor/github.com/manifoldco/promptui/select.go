package promptui

import (
	"bytes"
	"fmt"
	"io"
	"os"
	"text/tabwriter"
	"text/template"

	"github.com/chzyer/readline"
	"github.com/manifoldco/promptui/list"
	"github.com/manifoldco/promptui/screenbuf"
)

// SelectedAdd is used internally inside SelectWithAdd when the add option is selected in select mode.
// Since -1 is not a possible selected index, this ensure that add mode is always unique inside
// SelectWithAdd's logic.
const SelectedAdd = -1

// Select represents a list of items used to enable selections, they can be used as search engines, menus
// or as a list of items in a cli based prompt.
type Select struct {
	// Label is the text displayed on top of the list to direct input. The IconInitial value "?" will be
	// appended automatically to the label so it does not need to be added.
	//
	// The value for Label can be a simple string or a struct that will need to be accessed by dot notation
	// inside the templates. For example, `{{ .Name }}` will display the name property of a struct.
	Label interface{}

	// Items are the items to display inside the list. It expect a slice of any kind of values, including strings.
	//
	// If using a slice of strings, promptui will use those strings directly into its base templates or the
	// provided templates. If using any other type in the slice, it will attempt to transform it into a string
	// before giving it to its templates. Custom templates will override this behavior if using the dot notation
	// inside the templates.
	//
	// For example, `{{ .Name }}` will display the name property of a struct.
	Items interface{}

	// Size is the number of items that should appear on the select before scrolling is necessary. Defaults to 5.
	Size int

	// CursorPos is the initial position of the cursor.
	CursorPos int

	// IsVimMode sets whether to use vim mode when using readline in the command prompt. Look at
	// https://godoc.org/github.com/chzyer/readline#Config for more information on readline.
	IsVimMode bool

	// HideHelp sets whether to hide help information.
	HideHelp bool

	// HideSelected sets whether to hide the text displayed after an item is successfully selected.
	HideSelected bool

	// Templates can be used to customize the select output. If nil is passed, the
	// default templates are used. See the SelectTemplates docs for more info.
	Templates *SelectTemplates

	// Keys is the set of keys used in select mode to control the command line interface. See the SelectKeys docs for
	// more info.
	Keys *SelectKeys

	// Searcher is a function that can be implemented to refine the base searching algorithm in selects.
	//
	// Search is a function that will receive the searched term and the item's index and should return a boolean
	// for whether or not the terms are alike. It is unimplemented by default and search will not work unless
	// it is implemented.
	Searcher list.Searcher

	// StartInSearchMode sets whether or not the select mode should start in search mode or selection mode.
	// For search mode to work, the Search property must be implemented.
	StartInSearchMode bool

	list *list.List

	// A function that determines how to render the cursor
	Pointer Pointer

	Stdin  io.ReadCloser
	Stdout io.WriteCloser
}

// SelectKeys defines the available keys used by select mode to enable the user to move around the list
// and trigger search mode. See the Key struct docs for more information on keys.
type SelectKeys struct {
	// Next is the key used to move to the next element inside the list. Defaults to down arrow key.
	Next Key

	// Prev is the key used to move to the previous element inside the list. Defaults to up arrow key.
	Prev Key

	// PageUp is the key used to jump back to the first element inside the list. Defaults to left arrow key.
	PageUp Key

	// PageUp is the key used to jump forward to the last element inside the list. Defaults to right arrow key.
	PageDown Key

	// Search is the key used to trigger the search mode for the list. Default to the "/" key.
	Search Key
}

// Key defines a keyboard code and a display representation for the help menu.
type Key struct {
	// Code is a rune that will be used to compare against typed keys with readline.
	// Check https://github.com/chzyer/readline for a list of codes
	Code rune

	// Display is the string that will be displayed inside the help menu to help inform the user
	// of which key to use on his keyboard for various functions.
	Display string
}

// SelectTemplates allow a select list to be customized following stdlib
// text/template syntax. Custom state, colors and background color are available for use inside
// the templates and are documented inside the Variable section of the docs.
//
// Examples
//
// text/templates use a special notation to display programmable content. Using the double bracket notation,
// the value can be printed with specific helper functions. For example
//
// This displays the value given to the template as pure, unstylized text. Structs are transformed to string
// with this notation.
// 	'{{ . }}'
//
// This displays the name property of the value colored in cyan
// 	'{{ .Name | cyan }}'
//
// This displays the label property of value colored in red with a cyan background-color
// 	'{{ .Label | red | cyan }}'
//
// See the doc of text/template for more info: https://golang.org/pkg/text/template/
//
// Notes
//
// Setting any of these templates will remove the icons from the default templates. They must
// be added back in each of their specific templates. The styles.go constants contains the default icons.
type SelectTemplates struct {
	// Label is a text/template for the main command line label. Defaults to printing the label as it with
	// the IconInitial.
	Label string

	// Active is a text/template for when an item is currently active within the list.
	Active string

	// Inactive is a text/template for when an item is not currently active inside the list. This
	// template is used for all items unless they are active or selected.
	Inactive string

	// Selected is a text/template for when an item was successfully selected.
	Selected string

	// Details is a text/template for when an item current active to show
	// additional information. It can have multiple lines.
	//
	// Detail will always be displayed for the active element and thus can be used to display additional
	// information on the element beyond its label.
	//
	// promptui will not trim spaces and tabs will be displayed if the template is indented.
	Details string

	// Help is a text/template for displaying instructions at the top. By default
	// it shows keys for movement and search.
	Help string

	// FuncMap is a map of helper functions that can be used inside of templates according to the text/template
	// documentation.
	//
	// By default, FuncMap contains the color functions used to color the text in templates. If FuncMap
	// is overridden, the colors functions must be added in the override from promptui.FuncMap to work.
	FuncMap template.FuncMap

	label    *template.Template
	active   *template.Template
	inactive *template.Template
	selected *template.Template
	details  *template.Template
	help     *template.Template
}

// SearchPrompt is the prompt displayed in search mode.
var SearchPrompt = "Search: "

// Run executes the select list. It displays the label and the list of items, asking the user to chose any
// value within to list. Run will keep the prompt alive until it has been canceled from
// the command prompt or it has received a valid value. It will return the value and an error if any
// occurred during the select's execution.
func (s *Select) Run() (int, string, error) {
	return s.RunCursorAt(s.CursorPos, 0)
}

// RunCursorAt executes the select list, initializing the cursor to the given
// position. Invalid cursor positions will be clamped to valid values.  It
// displays the label and the list of items, asking the user to chose any value
// within to list. Run will keep the prompt alive until it has been canceled
// from the command prompt or it has received a valid value. It will return
// the value and an error if any occurred during the select's execution.
func (s *Select) RunCursorAt(cursorPos, scroll int) (int, string, error) {
	if s.Size == 0 {
		s.Size = 5
	}

	l, err := list.New(s.Items, s.Size)
	if err != nil {
		return 0, "", err
	}
	l.Searcher = s.Searcher

	s.list = l

	s.setKeys()

	err = s.prepareTemplates()
	if err != nil {
		return 0, "", err
	}
	return s.innerRun(cursorPos, scroll, ' ')
}

func (s *Select) innerRun(cursorPos, scroll int, top rune) (int, string, error) {
	c := &readline.Config{
		Stdin:  s.Stdin,
		Stdout: s.Stdout,
	}
	err := c.Init()
	if err != nil {
		return 0, "", err
	}

	c.Stdin = readline.NewCancelableStdin(c.Stdin)

	if s.IsVimMode {
		c.VimMode = true
	}

	c.HistoryLimit = -1
	c.UniqueEditLine = true

	rl, err := readline.NewEx(c)
	if err != nil {
		return 0, "", err
	}

	rl.Write([]byte(hideCursor))
	sb := screenbuf.New(rl)

	cur := NewCursor("", s.Pointer, false)

	canSearch := s.Searcher != nil
	searchMode := s.StartInSearchMode
	s.list.SetCursor(cursorPos)
	s.list.SetStart(scroll)

	c.SetListener(func(line []rune, pos int, key rune) ([]rune, int, bool) {
		switch {
		case key == KeyEnter:
			return nil, 0, true
		case key == s.Keys.Next.Code || (key == 'j' && !searchMode):
			s.list.Next()
		case key == s.Keys.Prev.Code || (key == 'k' && !searchMode):
			s.list.Prev()
		case key == s.Keys.Search.Code:
			if !canSearch {
				break
			}

			if searchMode {
				searchMode = false
				cur.Replace("")
				s.list.CancelSearch()
			} else {
				searchMode = true
			}
		case key == KeyBackspace || key == KeyCtrlH:
			if !canSearch || !searchMode {
				break
			}

			cur.Backspace()
			if len(cur.Get()) > 0 {
				s.list.Search(cur.Get())
			} else {
				s.list.CancelSearch()
			}
		case key == s.Keys.PageUp.Code || (key == 'h' && !searchMode):
			s.list.PageUp()
		case key == s.Keys.PageDown.Code || (key == 'l' && !searchMode):
			s.list.PageDown()
		default:
			if canSearch && searchMode {
				cur.Update(string(line))
				s.list.Search(cur.Get())
			}
		}

		if searchMode {
			header := SearchPrompt + cur.Format()
			sb.WriteString(header)
		} else if !s.HideHelp {
			help := s.renderHelp(canSearch)
			sb.Write(help)
		}

		label := render(s.Templates.label, s.Label)
		sb.Write(label)

		items, idx := s.list.Items()
		last := len(items) - 1

		for i, item := range items {
			page := " "

			switch i {
			case 0:
				if s.list.CanPageUp() {
					page = "↑"
				} else {
					page = string(top)
				}
			case last:
				if s.list.CanPageDown() {
					page = "↓"
				}
			}

			output := []byte(page + " ")

			if i == idx {
				output = append(output, render(s.Templates.active, item)...)
			} else {
				output = append(output, render(s.Templates.inactive, item)...)
			}

			sb.Write(output)
		}

		if idx == list.NotFound {
			sb.WriteString("")
			sb.WriteString("No results")
		} else {
			active := items[idx]

			details := s.renderDetails(active)
			for _, d := range details {
				sb.Write(d)
			}
		}

		sb.Flush()

		return nil, 0, true
	})

	for {
		_, err = rl.Readline()

		if err != nil {
			switch {
			case err == readline.ErrInterrupt, err.Error() == "Interrupt":
				err = ErrInterrupt
			case err == io.EOF:
				err = ErrEOF
			}
			break
		}

		_, idx := s.list.Items()
		if idx != list.NotFound {
			break
		}

	}

	if err != nil {
		if err.Error() == "Interrupt" {
			err = ErrInterrupt
		}
		sb.Reset()
		sb.WriteString("")
		sb.Flush()
		rl.Write([]byte(showCursor))
		rl.Close()
		return 0, "", err
	}

	items, idx := s.list.Items()
	item := items[idx]

	if s.HideSelected {
		clearScreen(sb)
	} else {
		sb.Reset()
		sb.Write(render(s.Templates.selected, item))
		sb.Flush()
	}

	rl.Write([]byte(showCursor))
	rl.Close()

	return s.list.Index(), fmt.Sprintf("%v", item), err
}

// ScrollPosition returns the current scroll position.
func (s *Select) ScrollPosition() int {
	return s.list.Start()
}

func (s *Select) prepareTemplates() error {
	tpls := s.Templates
	if tpls == nil {
		tpls = &SelectTemplates{}
	}

	if tpls.FuncMap == nil {
		tpls.FuncMap = FuncMap
	}

	if tpls.Label == "" {
		tpls.Label = fmt.Sprintf("%s {{.}}: ", IconInitial)
	}

	tpl, err := template.New("").Funcs(tpls.FuncMap).Parse(tpls.Label)
	if err != nil {
		return err
	}

	tpls.label = tpl

	if tpls.Active == "" {
		tpls.Active = fmt.Sprintf("%s {{ . | underline }}", IconSelect)
	}

	tpl, err = template.New("").Funcs(tpls.FuncMap).Parse(tpls.Active)
	if err != nil {
		return err
	}

	tpls.active = tpl

	if tpls.Inactive == "" {
		tpls.Inactive = "  {{.}}"
	}

	tpl, err = template.New("").Funcs(tpls.FuncMap).Parse(tpls.Inactive)
	if err != nil {
		return err
	}

	tpls.inactive = tpl

	if tpls.Selected == "" {
		tpls.Selected = fmt.Sprintf(`{{ "%s" | green }} {{ . | faint }}`, IconGood)
	}

	tpl, err = template.New("").Funcs(tpls.FuncMap).Parse(tpls.Selected)
	if err != nil {
		return err
	}
	tpls.selected = tpl

	if tpls.Details != "" {
		tpl, err = template.New("").Funcs(tpls.FuncMap).Parse(tpls.Details)
		if err != nil {
			return err
		}

		tpls.details = tpl
	}

	if tpls.Help == "" {
		tpls.Help = fmt.Sprintf(`{{ "Use the arrow keys to navigate:" | faint }} {{ .NextKey | faint }} ` +
			`{{ .PrevKey | faint }} {{ .PageDownKey | faint }} {{ .PageUpKey | faint }} ` +
			`{{ if .Search }} {{ "and" | faint }} {{ .SearchKey | faint }} {{ "toggles search" | faint }}{{ end }}`)
	}

	tpl, err = template.New("").Funcs(tpls.FuncMap).Parse(tpls.Help)
	if err != nil {
		return err
	}

	tpls.help = tpl

	s.Templates = tpls

	return nil
}

// SelectWithAdd represents a list for selecting a single item inside a list of items with the possibility to
// add new items to the list.
type SelectWithAdd struct {
	// Label is the text displayed on top of the list to direct input. The IconInitial value "?" will be
	// appended automatically to the label so it does not need to be added.
	Label string

	// Items are the items to display inside the list. Each item will be listed individually with the
	// AddLabel as the first item of the list.
	Items []string

	// AddLabel is the label used for the first item of the list that enables adding a new item.
	// Selecting this item in the list displays the add item prompt using promptui/prompt.
	AddLabel string

	// Validate is an optional function that fill be used against the entered value in the prompt to validate it.
	// If the value is valid, it is returned to the callee to be added in the list.
	Validate ValidateFunc

	// IsVimMode sets whether to use vim mode when using readline in the command prompt. Look at
	// https://godoc.org/github.com/chzyer/readline#Config for more information on readline.
	IsVimMode bool

	// a function that defines how to render the cursor
	Pointer Pointer

	// HideHelp sets whether to hide help information.
	HideHelp bool
}

// Run executes the select list. Its displays the label and the list of items, asking the user to chose any
// value within to list or add his own. Run will keep the prompt alive until it has been canceled from
// the command prompt or it has received a valid value.
//
// If the addLabel is selected in the list, this function will return a -1 index with the added label and no error.
// Otherwise, it will return the index and the value of the selected item. In any case, if an error is triggered, it
// will also return the error as its third return value.
func (sa *SelectWithAdd) Run() (int, string, error) {
	if len(sa.Items) > 0 {
		newItems := append([]string{sa.AddLabel}, sa.Items...)

		list, err := list.New(newItems, 5)
		if err != nil {
			return 0, "", err
		}

		s := Select{
			Label:     sa.Label,
			Items:     newItems,
			IsVimMode: sa.IsVimMode,
			HideHelp:  sa.HideHelp,
			Size:      5,
			list:      list,
			Pointer:   sa.Pointer,
		}
		s.setKeys()

		err = s.prepareTemplates()
		if err != nil {
			return 0, "", err
		}

		selected, value, err := s.innerRun(1, 0, '+')
		if err != nil || selected != 0 {
			return selected - 1, value, err
		}

		// XXX run through terminal for windows
		os.Stdout.Write([]byte(upLine(1) + "\r" + clearLine))
	}

	p := Prompt{
		Label:     sa.AddLabel,
		Validate:  sa.Validate,
		IsVimMode: sa.IsVimMode,
		Pointer:   sa.Pointer,
	}
	value, err := p.Run()
	return SelectedAdd, value, err
}

func (s *Select) setKeys() {
	if s.Keys != nil {
		return
	}
	s.Keys = &SelectKeys{
		Prev:     Key{Code: KeyPrev, Display: KeyPrevDisplay},
		Next:     Key{Code: KeyNext, Display: KeyNextDisplay},
		PageUp:   Key{Code: KeyBackward, Display: KeyBackwardDisplay},
		PageDown: Key{Code: KeyForward, Display: KeyForwardDisplay},
		Search:   Key{Code: '/', Display: "/"},
	}
}

func (s *Select) renderDetails(item interface{}) [][]byte {
	if s.Templates.details == nil {
		return nil
	}

	var buf bytes.Buffer

	w := tabwriter.NewWriter(&buf, 0, 0, 8, ' ', 0)

	err := s.Templates.details.Execute(w, item)
	if err != nil {
		fmt.Fprintf(w, "%v", item)
	}

	w.Flush()

	output := buf.Bytes()

	return bytes.Split(output, []byte("\n"))
}

func (s *Select) renderHelp(b bool) []byte {
	keys := struct {
		NextKey     string
		PrevKey     string
		PageDownKey string
		PageUpKey   string
		Search      bool
		SearchKey   string
	}{
		NextKey:     s.Keys.Next.Display,
		PrevKey:     s.Keys.Prev.Display,
		PageDownKey: s.Keys.PageDown.Display,
		PageUpKey:   s.Keys.PageUp.Display,
		SearchKey:   s.Keys.Search.Display,
		Search:      b,
	}

	return render(s.Templates.help, keys)
}

func render(tpl *template.Template, data interface{}) []byte {
	var buf bytes.Buffer
	err := tpl.Execute(&buf, data)
	if err != nil {
		return []byte(fmt.Sprintf("%v", data))
	}
	return buf.Bytes()
}

func clearScreen(sb *screenbuf.ScreenBuf) {
	sb.Reset()
	sb.Clear()
	sb.Flush()
}
