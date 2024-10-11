package main

import (
	"os"
	"path"
	"sort"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
)

const (
	nonTestFile       = "not_test_file.go"
	goTestFileNameOne = "first_go_file_test.go"
	goTestFileNameTwo = "second_go_file_test.go"
)

func TestGetGithubActionMatrixForTests(t *testing.T) {
	t.Run("empty dir does not fail", func(t *testing.T) {
		testingDir := t.TempDir()
		_, err := getGithubActionMatrixForTests(testingDir, "", "", nil)
		assert.NoError(t, err)
	})

	t.Run("only test functions are picked up", func(t *testing.T) {
		testingDir := t.TempDir()
		createFileWithTestSuiteAndTests(t, "FeeMiddlewareTestSuite", "TestA", "TestB", testingDir, goTestFileNameOne)

		gh, err := getGithubActionMatrixForTests(testingDir, "", "", nil)
		assert.NoError(t, err)

		expected := GithubActionTestMatrix{
			Include: []TestSuitePair{
				{
					EntryPoint: "TestFeeMiddlewareTestSuite",
					Test:       "TestA",
				},
				{
					EntryPoint: "TestFeeMiddlewareTestSuite",
					Test:       "TestB",
				},
			},
		}
		assertGithubActionTestMatricesEqual(t, expected, gh)
	})

	t.Run("all files are picked up", func(t *testing.T) {
		testingDir := t.TempDir()
		createFileWithTestSuiteAndTests(t, "FeeMiddlewareTestSuite", "TestA", "TestB", testingDir, goTestFileNameOne)
		createFileWithTestSuiteAndTests(t, "TransferTestSuite", "TestC", "TestD", testingDir, goTestFileNameTwo)

		gh, err := getGithubActionMatrixForTests(testingDir, "", "", nil)
		assert.NoError(t, err)

		expected := GithubActionTestMatrix{
			Include: []TestSuitePair{
				{
					EntryPoint: "TestTransferTestSuite",
					Test:       "TestC",
				},
				{
					EntryPoint: "TestFeeMiddlewareTestSuite",
					Test:       "TestA",
				},
				{
					EntryPoint: "TestFeeMiddlewareTestSuite",
					Test:       "TestB",
				},
				{
					EntryPoint: "TestTransferTestSuite",
					Test:       "TestD",
				},
			},
		}

		assertGithubActionTestMatricesEqual(t, expected, gh)
	})

	t.Run("single test can be specified", func(t *testing.T) {
		testingDir := t.TempDir()
		createFileWithTestSuiteAndTests(t, "FeeMiddlewareTestSuite", "TestA", "TestB", testingDir, goTestFileNameOne)
		createFileWithTestSuiteAndTests(t, "TransferTestSuite", "TestC", "TestD", testingDir, goTestFileNameTwo)

		gh, err := getGithubActionMatrixForTests(testingDir, "TestA", "TestFeeMiddlewareTestSuite", nil)
		assert.NoError(t, err)

		expected := GithubActionTestMatrix{
			Include: []TestSuitePair{
				{
					EntryPoint: "TestFeeMiddlewareTestSuite",
					Test:       "TestA",
				},
			},
		}

		assertGithubActionTestMatricesEqual(t, expected, gh)
	})

	t.Run("error if single test doesn't exist", func(t *testing.T) {
		testingDir := t.TempDir()
		createFileWithTestSuiteAndTests(t, "FeeMiddlewareTestSuite", "TestA", "TestB", testingDir, goTestFileNameOne)

		_, err := getGithubActionMatrixForTests(testingDir, "TestThatDoesntExist", "TestFeeMiddlewareTestSuite", nil)
		assert.Error(t, err)
	})

	t.Run("non test files are not picked up", func(t *testing.T) {
		testingDir := t.TempDir()
		createFileWithTestSuiteAndTests(t, "FeeMiddlewareTestSuite", "TestA", "TestB", testingDir, nonTestFile)

		gh, err := getGithubActionMatrixForTests(testingDir, "", "", nil)
		assert.NoError(t, err)
		assert.Empty(t, gh.Include)
	})

	t.Run("fails when there are multiple suite runs", func(t *testing.T) {
		testingDir := t.TempDir()
		createFileWithTestSuiteAndTests(t, "FeeMiddlewareTestSuite", "TestA", "TestB", testingDir, nonTestFile)

		fileWithTwoSuites := `package foo
func SuiteOne(t *testing.T) {
	suite.Run(t, new(FeeMiddlewareTestSuite))
}

func SuiteTwo(t *testing.T) {
	suite.Run(t, new(FeeMiddlewareTestSuite))
}

type FeeMiddlewareTestSuite struct {}
`

		err := os.WriteFile(path.Join(testingDir, goTestFileNameOne), []byte(fileWithTwoSuites), os.FileMode(0o777))
		assert.NoError(t, err)

		_, err = getGithubActionMatrixForTests(testingDir, "", "", nil)
		assert.Error(t, err)
	})
}

func assertGithubActionTestMatricesEqual(t *testing.T, expected, actual GithubActionTestMatrix) {
	t.Helper()
	// sort by both suite and test as the order of the end result does not matter as
	// all tests will be run.
	sort.SliceStable(expected.Include, func(i, j int) bool {
		memberI := expected.Include[i]
		memberJ := expected.Include[j]
		if memberI.EntryPoint == memberJ.EntryPoint {
			return memberI.Test < memberJ.Test
		}
		return memberI.EntryPoint < memberJ.EntryPoint
	})

	sort.SliceStable(actual.Include, func(i, j int) bool {
		memberI := actual.Include[i]
		memberJ := actual.Include[j]
		if memberI.EntryPoint == memberJ.EntryPoint {
			return memberI.Test < memberJ.Test
		}
		return memberI.EntryPoint < memberJ.EntryPoint
	})
	assert.Equal(t, expected.Include, actual.Include)
}

func goTestFileContents(suiteName, fnName1, fnName2 string) string {
	replacedSuiteName := strings.ReplaceAll(`package foo

func TestSuiteName(t *testing.T) {
	suite.Run(t, new(SuiteName))
}

type SuiteName struct {}

func (s *SuiteName) fnName1() {}
func (s *SuiteName) fnName2() {}

func (s *SuiteName) suiteHelper() {}

func helper() {}
`, "SuiteName", suiteName)

	replacedFn1Name := strings.ReplaceAll(replacedSuiteName, "fnName1", fnName1)
	return strings.ReplaceAll(replacedFn1Name, "fnName2", fnName2)
}

func createFileWithTestSuiteAndTests(t *testing.T, suiteName, fn1Name, fn2Name, dir, filename string) {
	t.Helper()
	goFileContents := goTestFileContents(suiteName, fn1Name, fn2Name)
	err := os.WriteFile(path.Join(dir, filename), []byte(goFileContents), os.FileMode(0o777))
	assert.NoError(t, err)
}
