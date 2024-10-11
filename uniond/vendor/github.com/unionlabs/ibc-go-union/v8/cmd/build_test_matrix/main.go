package main

import (
	"encoding/json"
	"fmt"
	"go/ast"
	"go/parser"
	"go/token"
	"io/fs"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

const (
	testNamePrefix     = "Test"
	testFileNameSuffix = "_test.go"
	e2eTestDirectory   = "e2e"
	// testEntryPointEnv specifes a single test function to run if provided.
	testEntryPointEnv = "TEST_ENTRYPOINT"
	// testExclusionsEnv is a comma separated list of test function names that will not be included
	// in the results of this script.
	testExclusionsEnv = "TEST_EXCLUSIONS"
	// testNameEnv if provided returns a single test entry so that only one test is actually run.
	testNameEnv = "TEST_NAME"
)

// GithubActionTestMatrix represents
type GithubActionTestMatrix struct {
	Include []TestSuitePair `json:"include"`
}

type TestSuitePair struct {
	Test       string `json:"test"`
	EntryPoint string `json:"entrypoint"`
}

func main() {
	githubActionMatrix, err := getGithubActionMatrixForTests(e2eTestDirectory, getTestToRun(), getTestEntrypointToRun(), getExcludedTestFunctions())
	if err != nil {
		fmt.Printf("error generating github action json: %s", err)
		os.Exit(1)
	}

	ghBytes, err := json.Marshal(githubActionMatrix)
	if err != nil {
		fmt.Printf("error marshalling github action json: %s", err)
		os.Exit(1)
	}
	fmt.Println(string(ghBytes))
}

// getTestEntrypointToRun returns the specified test function to run if present, otherwise
// it returns an empty string which will result in running all test suites.
func getTestEntrypointToRun() string {
	testSuite, ok := os.LookupEnv(testEntryPointEnv)
	if !ok {
		return ""
	}
	return testSuite
}

// getTestToRun returns the specified test function to run if present.
// If specified, only this test will be run.
func getTestToRun() string {
	testName, ok := os.LookupEnv(testNameEnv)
	if !ok {
		return ""
	}
	return testName
}

// getExcludedTestFunctions returns a list of test functions that we don't want to run.
func getExcludedTestFunctions() []string {
	exclusions, ok := os.LookupEnv(testExclusionsEnv)
	if !ok {
		return nil
	}
	return strings.Split(exclusions, ",")
}

func contains(s string, items []string) bool {
	for _, elem := range items {
		if elem == s {
			return true
		}
	}
	return false
}

// getGithubActionMatrixForTests returns a json string representing the contents that should go in the matrix
// field in a github action workflow. This string can be used with `fromJSON(str)` to dynamically build
// the workflow matrix to include all E2E tests under the e2eRootDirectory directory.
func getGithubActionMatrixForTests(e2eRootDirectory, testName string, suite string, excludedItems []string) (GithubActionTestMatrix, error) {
	testSuiteMapping := map[string][]string{}
	fset := token.NewFileSet()
	err := filepath.Walk(e2eRootDirectory, func(path string, info fs.FileInfo, err error) error {
		if err != nil {
			return fmt.Errorf("error walking e2e directory: %s", err)
		}

		// only look at test files
		if !strings.HasSuffix(path, testFileNameSuffix) {
			return nil
		}

		f, err := parser.ParseFile(fset, path, nil, 0)
		if err != nil {
			return fmt.Errorf("failed parsing file: %s", err)
		}

		suiteNameForFile, testCases, err := extractSuiteAndTestNames(f)
		if err != nil {
			return fmt.Errorf("failed extracting test suite name and test cases: %s", err)
		}

		if testName != "" && contains(testName, testCases) {
			testCases = []string{testName}
		}

		if contains(suiteNameForFile, excludedItems) {
			return nil
		}

		if suite == "" || suiteNameForFile == suite {
			testSuiteMapping[suiteNameForFile] = testCases
		}

		return nil
	})
	if err != nil {
		return GithubActionTestMatrix{}, err
	}

	gh := GithubActionTestMatrix{
		Include: []TestSuitePair{},
	}

	for testSuiteName, testCases := range testSuiteMapping {
		for _, testCaseName := range testCases {
			gh.Include = append(gh.Include, TestSuitePair{
				Test:       testCaseName,
				EntryPoint: testSuiteName,
			})
		}
	}
	// Sort the test cases by name so that the order is consistent.
	sort.SliceStable(gh.Include, func(i, j int) bool {
		return gh.Include[i].Test < gh.Include[j].Test
	})

	if testName != "" && len(gh.Include) != 1 {
		return GithubActionTestMatrix{}, fmt.Errorf("expected exactly 1 test in the output matrix but got %d", len(gh.Include))
	}

	return gh, nil
}

// extractSuiteAndTestNames extracts the name of the test suite function as well
// as all tests associated with it in the same file.
func extractSuiteAndTestNames(file *ast.File) (string, []string, error) {
	var suiteNameForFile string
	var testCases []string

	for _, d := range file.Decls {
		if f, ok := d.(*ast.FuncDecl); ok {
			functionName := f.Name.Name
			if isTestSuiteMethod(f) {
				if suiteNameForFile != "" {
					return "", nil, fmt.Errorf("found a second test function: %s when %s was already found", f.Name.Name, suiteNameForFile)
				}
				suiteNameForFile = functionName
				continue
			}
			if isTestFunction(f) {
				testCases = append(testCases, functionName)
			}
		}
	}
	if suiteNameForFile == "" {
		return "", nil, fmt.Errorf("file %s had no test suite test case", file.Name.Name)
	}
	return suiteNameForFile, testCases, nil
}

// isTestSuiteMethod returns true if the function is a test suite function.
// e.g. func TestFeeMiddlewareTestSuite(t *testing.T) { ... }
func isTestSuiteMethod(f *ast.FuncDecl) bool {
	return strings.HasPrefix(f.Name.Name, testNamePrefix) && len(f.Type.Params.List) == 1
}

// isTestFunction returns true if the function name starts with "Test" and has no parameters.
// as test suite functions do not accept a *testing.T.
func isTestFunction(f *ast.FuncDecl) bool {
	return strings.HasPrefix(f.Name.Name, testNamePrefix) && len(f.Type.Params.List) == 0
}
