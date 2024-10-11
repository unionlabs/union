// Copyright 2015 The go-ethereum Authors
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

package flags

import (
	"encoding"
	"errors"
	"flag"
	"fmt"
	"math/big"
	"os"
	"os/user"
	"path/filepath"
	"strings"
	"syscall"

	"github.com/ethereum/go-ethereum/common/math"
	"github.com/urfave/cli/v2"
)

// DirectoryString is custom type which is registered in the flags library which cli uses for
// argument parsing. This allows us to expand Value to an absolute path when
// the argument is parsed
type DirectoryString string

func (s *DirectoryString) String() string {
	return string(*s)
}

func (s *DirectoryString) Set(value string) error {
	*s = DirectoryString(expandPath(value))
	return nil
}

var (
	_ cli.Flag              = (*DirectoryFlag)(nil)
	_ cli.RequiredFlag      = (*DirectoryFlag)(nil)
	_ cli.VisibleFlag       = (*DirectoryFlag)(nil)
	_ cli.DocGenerationFlag = (*DirectoryFlag)(nil)
	_ cli.CategorizableFlag = (*DirectoryFlag)(nil)
)

// DirectoryFlag is custom cli.Flag type which expand the received string to an absolute path.
// e.g. ~/.ethereum -> /home/username/.ethereum
type DirectoryFlag struct {
	Name string

	Category    string
	DefaultText string
	Usage       string

	Required   bool
	Hidden     bool
	HasBeenSet bool

	Value DirectoryString

	Aliases []string
	EnvVars []string
}

// For cli.Flag:

func (f *DirectoryFlag) Names() []string { return append([]string{f.Name}, f.Aliases...) }
func (f *DirectoryFlag) IsSet() bool     { return f.HasBeenSet }
func (f *DirectoryFlag) String() string  { return cli.FlagStringer(f) }

// Apply called by cli library, grabs variable from environment (if in env)
// and adds variable to flag set for parsing.
func (f *DirectoryFlag) Apply(set *flag.FlagSet) error {
	for _, envVar := range f.EnvVars {
		envVar = strings.TrimSpace(envVar)
		if value, found := syscall.Getenv(envVar); found {
			f.Value.Set(value)
			f.HasBeenSet = true
			break
		}
	}
	eachName(f, func(name string) {
		set.Var(&f.Value, f.Name, f.Usage)
	})
	return nil
}

// For cli.RequiredFlag:

func (f *DirectoryFlag) IsRequired() bool { return f.Required }

// For cli.VisibleFlag:

func (f *DirectoryFlag) IsVisible() bool { return !f.Hidden }

// For cli.CategorizableFlag:

func (f *DirectoryFlag) GetCategory() string { return f.Category }

// For cli.DocGenerationFlag:

func (f *DirectoryFlag) TakesValue() bool     { return true }
func (f *DirectoryFlag) GetUsage() string     { return f.Usage }
func (f *DirectoryFlag) GetValue() string     { return f.Value.String() }
func (f *DirectoryFlag) GetEnvVars() []string { return f.EnvVars }

func (f *DirectoryFlag) GetDefaultText() string {
	if f.DefaultText != "" {
		return f.DefaultText
	}
	return f.GetValue()
}

type TextMarshaler interface {
	encoding.TextMarshaler
	encoding.TextUnmarshaler
}

// textMarshalerVal turns a TextMarshaler into a flag.Value
type textMarshalerVal struct {
	v TextMarshaler
}

func (v textMarshalerVal) String() string {
	if v.v == nil {
		return ""
	}
	text, _ := v.v.MarshalText()
	return string(text)
}

func (v textMarshalerVal) Set(s string) error {
	return v.v.UnmarshalText([]byte(s))
}

var (
	_ cli.Flag              = (*TextMarshalerFlag)(nil)
	_ cli.RequiredFlag      = (*TextMarshalerFlag)(nil)
	_ cli.VisibleFlag       = (*TextMarshalerFlag)(nil)
	_ cli.DocGenerationFlag = (*TextMarshalerFlag)(nil)
	_ cli.CategorizableFlag = (*TextMarshalerFlag)(nil)
)

// TextMarshalerFlag wraps a TextMarshaler value.
type TextMarshalerFlag struct {
	Name string

	Category    string
	DefaultText string
	Usage       string

	Required   bool
	Hidden     bool
	HasBeenSet bool

	Value TextMarshaler

	Aliases []string
	EnvVars []string
}

// For cli.Flag:

func (f *TextMarshalerFlag) Names() []string { return append([]string{f.Name}, f.Aliases...) }
func (f *TextMarshalerFlag) IsSet() bool     { return f.HasBeenSet }
func (f *TextMarshalerFlag) String() string  { return cli.FlagStringer(f) }

func (f *TextMarshalerFlag) Apply(set *flag.FlagSet) error {
	for _, envVar := range f.EnvVars {
		envVar = strings.TrimSpace(envVar)
		if value, found := syscall.Getenv(envVar); found {
			if err := f.Value.UnmarshalText([]byte(value)); err != nil {
				return fmt.Errorf("could not parse %q from environment variable %q for flag %s: %s", value, envVar, f.Name, err)
			}
			f.HasBeenSet = true
			break
		}
	}
	eachName(f, func(name string) {
		set.Var(textMarshalerVal{f.Value}, f.Name, f.Usage)
	})
	return nil
}

// For cli.RequiredFlag:

func (f *TextMarshalerFlag) IsRequired() bool { return f.Required }

// For cli.VisibleFlag:

func (f *TextMarshalerFlag) IsVisible() bool { return !f.Hidden }

// For cli.CategorizableFlag:

func (f *TextMarshalerFlag) GetCategory() string { return f.Category }

// For cli.DocGenerationFlag:

func (f *TextMarshalerFlag) TakesValue() bool     { return true }
func (f *TextMarshalerFlag) GetUsage() string     { return f.Usage }
func (f *TextMarshalerFlag) GetEnvVars() []string { return f.EnvVars }

func (f *TextMarshalerFlag) GetValue() string {
	t, err := f.Value.MarshalText()
	if err != nil {
		return "(ERR: " + err.Error() + ")"
	}
	return string(t)
}

func (f *TextMarshalerFlag) GetDefaultText() string {
	if f.DefaultText != "" {
		return f.DefaultText
	}
	return f.GetValue()
}

// GlobalTextMarshaler returns the value of a TextMarshalerFlag from the global flag set.
func GlobalTextMarshaler(ctx *cli.Context, name string) TextMarshaler {
	val := ctx.Generic(name)
	if val == nil {
		return nil
	}
	return val.(textMarshalerVal).v
}

var (
	_ cli.Flag              = (*BigFlag)(nil)
	_ cli.RequiredFlag      = (*BigFlag)(nil)
	_ cli.VisibleFlag       = (*BigFlag)(nil)
	_ cli.DocGenerationFlag = (*BigFlag)(nil)
	_ cli.CategorizableFlag = (*BigFlag)(nil)
)

// BigFlag is a command line flag that accepts 256 bit big integers in decimal or
// hexadecimal syntax.
type BigFlag struct {
	Name string

	Category    string
	DefaultText string
	Usage       string

	Required   bool
	Hidden     bool
	HasBeenSet bool

	Value *big.Int

	Aliases []string
	EnvVars []string
}

// For cli.Flag:

func (f *BigFlag) Names() []string { return append([]string{f.Name}, f.Aliases...) }
func (f *BigFlag) IsSet() bool     { return f.HasBeenSet }
func (f *BigFlag) String() string  { return cli.FlagStringer(f) }

func (f *BigFlag) Apply(set *flag.FlagSet) error {
	for _, envVar := range f.EnvVars {
		envVar = strings.TrimSpace(envVar)
		if value, found := syscall.Getenv(envVar); found {
			if _, ok := f.Value.SetString(value, 10); !ok {
				return fmt.Errorf("could not parse %q from environment variable %q for flag %s", value, envVar, f.Name)
			}
			f.HasBeenSet = true
			break
		}
	}
	eachName(f, func(name string) {
		f.Value = new(big.Int)
		set.Var((*bigValue)(f.Value), f.Name, f.Usage)
	})

	return nil
}

// For cli.RequiredFlag:

func (f *BigFlag) IsRequired() bool { return f.Required }

// For cli.VisibleFlag:

func (f *BigFlag) IsVisible() bool { return !f.Hidden }

// For cli.CategorizableFlag:

func (f *BigFlag) GetCategory() string { return f.Category }

// For cli.DocGenerationFlag:

func (f *BigFlag) TakesValue() bool     { return true }
func (f *BigFlag) GetUsage() string     { return f.Usage }
func (f *BigFlag) GetValue() string     { return f.Value.String() }
func (f *BigFlag) GetEnvVars() []string { return f.EnvVars }

func (f *BigFlag) GetDefaultText() string {
	if f.DefaultText != "" {
		return f.DefaultText
	}
	return f.GetValue()
}

// bigValue turns *big.Int into a flag.Value
type bigValue big.Int

func (b *bigValue) String() string {
	if b == nil {
		return ""
	}
	return (*big.Int)(b).String()
}

func (b *bigValue) Set(s string) error {
	intVal, ok := math.ParseBig256(s)
	if !ok {
		return errors.New("invalid integer syntax")
	}
	*b = (bigValue)(*intVal)
	return nil
}

// GlobalBig returns the value of a BigFlag from the global flag set.
func GlobalBig(ctx *cli.Context, name string) *big.Int {
	val := ctx.Generic(name)
	if val == nil {
		return nil
	}
	return (*big.Int)(val.(*bigValue))
}

// Expands a file path
// 1. replace tilde with users home dir
// 2. expands embedded environment variables
// 3. cleans the path, e.g. /a/b/../c -> /a/c
// Note, it has limitations, e.g. ~someuser/tmp will not be expanded
func expandPath(p string) string {
	// Named pipes are not file paths on windows, ignore
	if strings.HasPrefix(p, `\\.\pipe`) {
		return p
	}
	if strings.HasPrefix(p, "~/") || strings.HasPrefix(p, "~\\") {
		if home := HomeDir(); home != "" {
			p = home + p[1:]
		}
	}
	return filepath.Clean(os.ExpandEnv(p))
}

func HomeDir() string {
	if home := os.Getenv("HOME"); home != "" {
		return home
	}
	if usr, err := user.Current(); err == nil {
		return usr.HomeDir
	}
	return ""
}

func eachName(f cli.Flag, fn func(string)) {
	for _, name := range f.Names() {
		name = strings.Trim(name, " ")
		fn(name)
	}
}
