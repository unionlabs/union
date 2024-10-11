package main

import (
	"fmt"
	"go/ast"
	"go/parser"
	"go/token"
	"os"
	"path"
	"path/filepath"
	"regexp"
	"strings"

	"github.com/urfave/cli/v2"
)

// Regex to find Python's "def".
var reg1 = regexp.MustCompile(`def\s(.*)\(.*`)

// checkNumRows defines whether tool should check that the spec comment is the last comment of the block, so not only
// it matches the reference snippet, but it also has the same number of rows.
const checkNumRows = false

func check(cliCtx *cli.Context) error {
	// Obtain reference snippets.
	defs, err := parseSpecs()
	if err != nil {
		return err
	}

	// Walk the path, and process all contained Golang files.
	fileWalker := func(path string, info os.FileInfo, err error) error {
		if info == nil {
			return fmt.Errorf("invalid input dir %q", path)
		}
		if !strings.HasSuffix(info.Name(), ".go") {
			return nil
		}
		return inspectFile(path, defs)
	}
	return filepath.Walk(cliCtx.String(dirFlag.Name), fileWalker)
}

func inspectFile(path string, defs map[string][]string) error {
	// Parse source files, and check the pseudo code.
	fset := token.NewFileSet()
	file, err := parser.ParseFile(fset, path, nil, parser.ParseComments)
	if err != nil {
		return err
	}

	ast.Inspect(file, func(node ast.Node) bool {
		stmt, ok := node.(*ast.CommentGroup)
		if !ok {
			return true
		}
		// Ignore comment groups that do not have python pseudo-code.
		chunk := stmt.Text()
		if !reg1.MatchString(chunk) {
			return true
		}

		// Trim the chunk, so that it starts from Python's "def".
		loc := reg1.FindStringIndex(chunk)
		chunk = chunk[loc[0]:]

		// Find out Python function name.
		defName, defBody := parseDefChunk(chunk)
		if defName == "" {
			fmt.Printf("%s: cannot parse comment pseudo code\n", fset.Position(node.Pos()))
			return false
		}

		// Calculate differences with reference implementation.
		refDefs, ok := defs[defName]
		if !ok {
			fmt.Printf("%s: %q is not found in spec docs\n", fset.Position(node.Pos()), defName)
			return false
		}
		if !matchesRefImplementation(defName, refDefs, defBody, fset.Position(node.Pos())) {
			fmt.Printf("%s: %q code does not match reference implementation in specs\n", fset.Position(node.Pos()), defName)
			return false
		}

		return true
	})

	return nil
}

// parseSpecs parses input spec docs into map of function name -> array of function bodies
// (single entity may have several definitions).
func parseSpecs() (map[string][]string, error) {
	loadSpecsFile := func(sb *strings.Builder, specFilePath string) error {
		chunk, err := specFS.ReadFile(specFilePath)
		if err != nil {
			return fmt.Errorf("cannot read specs file: %w", err)
		}
		_, err = sb.Write(chunk)
		if err != nil {
			return fmt.Errorf("cannot copy specs file: %w", err)
		}
		return nil
	}

	// Traverse all spec files, and aggregate them within as single string.
	var sb strings.Builder
	for dirName, fileNames := range specDirs {
		for _, fileName := range fileNames {
			if err := loadSpecsFile(&sb, path.Join("data", dirName, fileName)); err != nil {
				return nil, err
			}
		}
	}

	// Load file with extra definitions (this allows us to use pseudo-code that is not from specs).
	if err := loadSpecsFile(&sb, path.Join("data", "extra.md")); err != nil {
		return nil, err
	}

	// Parse docs into function name -> array of function bodies map.
	chunks := strings.Split(strings.ReplaceAll(sb.String(), "```python", ""), "```")
	defs := make(map[string][]string, len(chunks))
	for _, chunk := range chunks {
		defName, defBody := parseDefChunk(chunk)
		if defName == "" {
			continue
		}
		defs[defName] = append(defs[defName], defBody)
	}
	return defs, nil
}

// parseDefChunk extract function name and function body from a Python's "def" chunk.
func parseDefChunk(chunk string) (string, string) {
	chunk = strings.TrimLeft(chunk, "\n")
	if chunk == "" {
		return "", ""
	}
	chunkLines := strings.Split(chunk, "\n")
	// Ignore all snippets, that do not define functions.
	if chunkLines[0][:4] != "def " {
		return "", ""
	}
	defMatches := reg1.FindStringSubmatch(chunkLines[0])
	if len(defMatches) < 2 {
		return "", ""
	}
	return strings.Trim(defMatches[1], " "), chunk
}

// matchesRefImplementation compares input string to reference code snippets (there might be multiple implementations).
func matchesRefImplementation(defName string, refDefs []string, input string, pos token.Position) bool {
	for _, refDef := range refDefs {
		refDefLines := strings.Split(strings.TrimRight(refDef, "\n"), "\n")
		inputLines := strings.Split(strings.TrimRight(input, "\n"), "\n")

		matchesPerfectly := true
		for i := 0; i < len(refDefLines); i++ {
			a, b := strings.Trim(refDefLines[i], " "), strings.Trim(inputLines[i], " ")
			if a != b {
				matchesPerfectly = false
				break
			}
		}
		// Mark potential issues, when there's some more comments in our code (which might be ok, as we are not required
		// to put specs comments as the last one in the doc block).
		if checkNumRows && len(refDefLines) != len(inputLines) {
			fmt.Printf("%s: %q potentially has issues (comment is longer than reference implementation)\n", pos, defName)
		}
		if matchesPerfectly {
			return true
		}
	}
	return false
}
