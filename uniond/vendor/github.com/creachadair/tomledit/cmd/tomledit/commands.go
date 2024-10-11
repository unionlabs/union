package main

import (
	"flag"
	"fmt"
	"strings"

	"github.com/creachadair/command"
	"github.com/creachadair/tomledit"
	"github.com/creachadair/tomledit/parser"
	"github.com/creachadair/tomledit/scanner"
	"github.com/creachadair/tomledit/transform"
)

var cmdList = &command.C{
	Name:  "list",
	Usage: "<key> ...",
	Help: `List the keys of key-value mappings.

With no keys, all the key-value mappings defined in the file are listed.
Otherwise, only those mappings having the given prefix are listed.`,

	Run: func(env *command.Env) error {
		doc, err := env.Config.(*settings).loadDocument()
		if err != nil {
			return err
		}
		keys, err := parseKeys(env.Args)
		if err != nil {
			return err
		}
		doc.Scan(func(key parser.Key, _ *tomledit.Entry) bool {
			if hasPrefixIn(key, keys) {
				fmt.Println(key)
			}
			return true
		})
		return nil
	},
}

var cmdPrint = &command.C{
	Name:  "print",
	Usage: "<key>",
	Help:  "Print the value of the first definition of a key.",

	Run: func(env *command.Env) error {
		if len(env.Args) == 0 {
			return env.Usagef("missing required key argument")
		}
		key, err := parser.ParseKey(env.Args[0])
		if err != nil {
			return fmt.Errorf("parsing key: %w", err)
		}
		doc, err := env.Config.(*settings).loadDocument()
		if err != nil {
			return err
		}
		first := doc.First(key...)
		if first == nil {
			return fmt.Errorf("key %q not found", key)
		} else if first.IsSection() {
			fmt.Println(first.Section.Heading.String())
		} else {
			fmt.Println(first.KeyValue.Value.String())
		}
		return nil
	},
}

var cmdSet = &command.C{
	Name:  "set",
	Usage: "<key> <value>",
	Help:  "Set the value of an existing key.",

	Run: func(env *command.Env) error {
		if len(env.Args) != 2 {
			return env.Usagef("required arguments are <key> <value>")
		}
		key, err := parser.ParseKey(env.Args[0])
		if err != nil {
			return fmt.Errorf("parsing key: %w", err)
		}
		val, err := parseValue(env.Args[1])
		if err != nil {
			return fmt.Errorf("invalid TOML value: %w", err)
		}
		cfg := env.Config.(*settings)
		doc, err := cfg.loadDocument()
		if err != nil {
			return err
		}
		found := doc.Find(key...)
		if len(found) == 0 {
			return fmt.Errorf("key %q not found", key)
		} else if len(found) > 1 {
			return fmt.Errorf("found %d definitions of key %q", len(found), key)
		} else if !found[0].IsMapping() {
			return fmt.Errorf("%q is not a key-value mapping", key)
		}
		found[0].KeyValue.Value = val
		return cfg.saveDocument(doc)
	},
}

var cmdAdd = &command.C{
	Name: "add",
	Usage: `<table> <key> <value>
<global-key> <value>`,
	Help: `Add a key-value mapping to the specified section.

If no table name is specified, a mapping is added to the global table.
Otherwise, the mapping is added to the specified table (which must exist).
An error is reported if the key already exists, unless -replace is set.`,

	SetFlags: func(env *command.Env, fs *flag.FlagSet) {
		cfg := env.Config.(*settings)
		fs.BoolVar(&cfg.Replace, "replace", false, "Replace an existing mapping if present")
		fs.StringVar(&cfg.Text, "comment", "", "Comment text to add to the mapping")
	},

	Run: func(env *command.Env) error {
		if len(env.Args) < 2 || len(env.Args) > 3 {
			return env.Usagef("wrong number of arguments")
		}
		key, err := parser.ParseKey(env.Args[0])
		if err != nil {
			return fmt.Errorf("parsing key %q: %w", env.Args[0], err)
		}
		val, err := parseValue(env.Args[len(env.Args)-1])
		if err != nil {
			return fmt.Errorf("parsing value: %w", err)
		}
		var section parser.Key
		if len(env.Args) == 3 {
			section = key
			key, err = parser.ParseKey(env.Args[1])
			if err != nil {
				return fmt.Errorf("parsing key %q: %w", env.Args[1], err)
			}
		}

		cfg := env.Config.(*settings)
		doc, err := cfg.loadDocument()
		if err != nil {
			return err
		}
		table := transform.FindTable(doc, section...)
		if table == nil {
			return fmt.Errorf("table %q not found", section)
		}
		var block parser.Comments
		if cfg.Text != "" {
			block = parser.Comments{cfg.Text}
		}
		if !transform.InsertMapping(table.Section, &parser.KeyValue{
			Block: block,
			Name:  key,
			Value: val,
		}, cfg.Replace) {
			return fmt.Errorf("key %q exists (use -replace to replace it)", key)
		}
		return cfg.saveDocument(doc)
	},
}

func parseKeys(args []string) ([]parser.Key, error) {
	var keys []parser.Key
	for _, arg := range args {
		key, err := parser.ParseKey(arg)
		if err != nil {
			return nil, fmt.Errorf("parsing key %q: %w", arg, err)
		}
		keys = append(keys, key)
	}
	return keys, nil
}

func hasPrefixIn(needle parser.Key, keys []parser.Key) bool {
	for _, key := range keys {
		if key.IsPrefixOf(needle) {
			return true
		}
	}
	return len(keys) == 0
}

func parseValue(s string) (parser.Value, error) {
	if strings.HasPrefix(s, "@") {
		actual := `"` + string(scanner.Escape(s[1:])) + `"`
		return parser.ParseValue(actual)
	}
	return parser.ParseValue(s)
}
