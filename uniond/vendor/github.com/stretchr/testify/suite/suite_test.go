package suite

import (
	"bytes"
	"errors"
	"flag"
	"io/ioutil"
	"math/rand"
	"os"
	"os/exec"
	"strings"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// SuiteRequireTwice is intended to test the usage of suite.Require in two
// different tests
type SuiteRequireTwice struct{ Suite }

// TestSuiteRequireTwice checks for regressions of issue #149 where
// suite.requirements was not initialized in suite.SetT()
// A regression would result on these tests panicking rather than failing.
func TestSuiteRequireTwice(t *testing.T) {
	ok := testing.RunTests(
		allTestsFilter,
		[]testing.InternalTest{{
			Name: t.Name() + "/SuiteRequireTwice",
			F: func(t *testing.T) {
				suite := new(SuiteRequireTwice)
				Run(t, suite)
			},
		}},
	)
	assert.False(t, ok)
}

func (s *SuiteRequireTwice) TestRequireOne() {
	r := s.Require()
	r.Equal(1, 2)
}

func (s *SuiteRequireTwice) TestRequireTwo() {
	r := s.Require()
	r.Equal(1, 2)
}

type panickingSuite struct {
	Suite
	panicInSetupSuite    bool
	panicInSetupTest     bool
	panicInBeforeTest    bool
	panicInTest          bool
	panicInAfterTest     bool
	panicInTearDownTest  bool
	panicInTearDownSuite bool
}

func (s *panickingSuite) SetupSuite() {
	if s.panicInSetupSuite {
		panic("oops in setup suite")
	}
}

func (s *panickingSuite) SetupTest() {
	if s.panicInSetupTest {
		panic("oops in setup test")
	}
}

func (s *panickingSuite) BeforeTest(_, _ string) {
	if s.panicInBeforeTest {
		panic("oops in before test")
	}
}

func (s *panickingSuite) Test() {
	if s.panicInTest {
		panic("oops in test")
	}
}

func (s *panickingSuite) AfterTest(_, _ string) {
	if s.panicInAfterTest {
		panic("oops in after test")
	}
}

func (s *panickingSuite) TearDownTest() {
	if s.panicInTearDownTest {
		panic("oops in tear down test")
	}
}

func (s *panickingSuite) TearDownSuite() {
	if s.panicInTearDownSuite {
		panic("oops in tear down suite")
	}
}

func TestSuiteRecoverPanic(t *testing.T) {
	ok := true
	panickingTests := []testing.InternalTest{
		{
			Name: t.Name() + "/InSetupSuite",
			F:    func(t *testing.T) { Run(t, &panickingSuite{panicInSetupSuite: true}) },
		},
		{
			Name: t.Name() + "/InSetupTest",
			F:    func(t *testing.T) { Run(t, &panickingSuite{panicInSetupTest: true}) },
		},
		{
			Name: t.Name() + "InBeforeTest",
			F:    func(t *testing.T) { Run(t, &panickingSuite{panicInBeforeTest: true}) },
		},
		{
			Name: t.Name() + "/InTest",
			F:    func(t *testing.T) { Run(t, &panickingSuite{panicInTest: true}) },
		},
		{
			Name: t.Name() + "/InAfterTest",
			F:    func(t *testing.T) { Run(t, &panickingSuite{panicInAfterTest: true}) },
		},
		{
			Name: t.Name() + "/InTearDownTest",
			F:    func(t *testing.T) { Run(t, &panickingSuite{panicInTearDownTest: true}) },
		},
		{
			Name: t.Name() + "/InTearDownSuite",
			F:    func(t *testing.T) { Run(t, &panickingSuite{panicInTearDownSuite: true}) },
		},
	}

	require.NotPanics(t, func() {
		ok = testing.RunTests(allTestsFilter, panickingTests)
	})

	assert.False(t, ok)
}

// This suite is intended to store values to make sure that only
// testing-suite-related methods are run.  It's also a fully
// functional example of a testing suite, using setup/teardown methods
// and a helper method that is ignored by testify.  To make this look
// more like a real world example, all tests in the suite perform some
// type of assertion.
type SuiteTester struct {
	// Include our basic suite logic.
	Suite

	// Keep counts of how many times each method is run.
	SetupSuiteRunCount      int
	TearDownSuiteRunCount   int
	SetupTestRunCount       int
	TearDownTestRunCount    int
	TestOneRunCount         int
	TestTwoRunCount         int
	TestSubtestRunCount     int
	NonTestMethodRunCount   int
	SetupSubTestRunCount    int
	TearDownSubTestRunCount int

	SetupSubTestNames    []string
	TearDownSubTestNames []string

	SuiteNameBefore []string
	TestNameBefore  []string

	SuiteNameAfter []string
	TestNameAfter  []string

	TimeBefore []time.Time
	TimeAfter  []time.Time
}

// The SetupSuite method will be run by testify once, at the very
// start of the testing suite, before any tests are run.
func (suite *SuiteTester) SetupSuite() {
	suite.SetupSuiteRunCount++
}

func (suite *SuiteTester) BeforeTest(suiteName, testName string) {
	suite.SuiteNameBefore = append(suite.SuiteNameBefore, suiteName)
	suite.TestNameBefore = append(suite.TestNameBefore, testName)
	suite.TimeBefore = append(suite.TimeBefore, time.Now())
}

func (suite *SuiteTester) AfterTest(suiteName, testName string) {
	suite.SuiteNameAfter = append(suite.SuiteNameAfter, suiteName)
	suite.TestNameAfter = append(suite.TestNameAfter, testName)
	suite.TimeAfter = append(suite.TimeAfter, time.Now())
}

// The TearDownSuite method will be run by testify once, at the very
// end of the testing suite, after all tests have been run.
func (suite *SuiteTester) TearDownSuite() {
	suite.TearDownSuiteRunCount++
}

// The SetupTest method will be run before every test in the suite.
func (suite *SuiteTester) SetupTest() {
	suite.SetupTestRunCount++
}

// The TearDownTest method will be run after every test in the suite.
func (suite *SuiteTester) TearDownTest() {
	suite.TearDownTestRunCount++
}

// Every method in a testing suite that begins with "Test" will be run
// as a test.  TestOne is an example of a test.  For the purposes of
// this example, we've included assertions in the tests, since most
// tests will issue assertions.
func (suite *SuiteTester) TestOne() {
	beforeCount := suite.TestOneRunCount
	suite.TestOneRunCount++
	assert.Equal(suite.T(), suite.TestOneRunCount, beforeCount+1)
	suite.Equal(suite.TestOneRunCount, beforeCount+1)
}

// TestTwo is another example of a test.
func (suite *SuiteTester) TestTwo() {
	beforeCount := suite.TestTwoRunCount
	suite.TestTwoRunCount++
	assert.NotEqual(suite.T(), suite.TestTwoRunCount, beforeCount)
	suite.NotEqual(suite.TestTwoRunCount, beforeCount)
}

func (suite *SuiteTester) TestSkip() {
	suite.T().Skip()
}

// NonTestMethod does not begin with "Test", so it will not be run by
// testify as a test in the suite.  This is useful for creating helper
// methods for your tests.
func (suite *SuiteTester) NonTestMethod() {
	suite.NonTestMethodRunCount++
}

func (suite *SuiteTester) TestSubtest() {
	suite.TestSubtestRunCount++

	for _, t := range []struct {
		testName string
	}{
		{"first"},
		{"second"},
	} {
		suiteT := suite.T()
		suite.Run(t.testName, func() {
			// We should get a different *testing.T for subtests, so that
			// go test recognizes them as proper subtests for output formatting
			// and running individual subtests
			subTestT := suite.T()
			suite.NotEqual(subTestT, suiteT)
		})
		suite.Equal(suiteT, suite.T())
	}
}

func (suite *SuiteTester) TearDownSubTest() {
	suite.TearDownSubTestNames = append(suite.TearDownSubTestNames, suite.T().Name())
	suite.TearDownSubTestRunCount++
}

func (suite *SuiteTester) SetupSubTest() {
	suite.SetupSubTestNames = append(suite.SetupSubTestNames, suite.T().Name())
	suite.SetupSubTestRunCount++
}

type SuiteSkipTester struct {
	// Include our basic suite logic.
	Suite

	// Keep counts of how many times each method is run.
	SetupSuiteRunCount    int
	TearDownSuiteRunCount int
}

func (suite *SuiteSkipTester) SetupSuite() {
	suite.SetupSuiteRunCount++
	suite.T().Skip()
}

func (suite *SuiteSkipTester) TestNothing() {
	// SetupSuite is only called when at least one test satisfies
	// test filter. For this suite to be set up (and then tore down)
	// it is necessary to add at least one test method.
}

func (suite *SuiteSkipTester) TearDownSuite() {
	suite.TearDownSuiteRunCount++
}

// TestRunSuite will be run by the 'go test' command, so within it, we
// can run our suite using the Run(*testing.T, TestingSuite) function.
func TestRunSuite(t *testing.T) {
	suiteTester := new(SuiteTester)
	Run(t, suiteTester)

	// Normally, the test would end here.  The following are simply
	// some assertions to ensure that the Run function is working as
	// intended - they are not part of the example.

	// The suite was only run once, so the SetupSuite and TearDownSuite
	// methods should have each been run only once.
	assert.Equal(t, 1, suiteTester.SetupSuiteRunCount)
	assert.Equal(t, 1, suiteTester.TearDownSuiteRunCount)

	assert.Len(t, suiteTester.SuiteNameAfter, 4)
	assert.Len(t, suiteTester.SuiteNameBefore, 4)
	assert.Len(t, suiteTester.TestNameAfter, 4)
	assert.Len(t, suiteTester.TestNameBefore, 4)

	assert.Contains(t, suiteTester.TestNameAfter, "TestOne")
	assert.Contains(t, suiteTester.TestNameAfter, "TestTwo")
	assert.Contains(t, suiteTester.TestNameAfter, "TestSkip")
	assert.Contains(t, suiteTester.TestNameAfter, "TestSubtest")

	assert.Contains(t, suiteTester.TestNameBefore, "TestOne")
	assert.Contains(t, suiteTester.TestNameBefore, "TestTwo")
	assert.Contains(t, suiteTester.TestNameBefore, "TestSkip")
	assert.Contains(t, suiteTester.TestNameBefore, "TestSubtest")

	assert.Contains(t, suiteTester.SetupSubTestNames, "TestRunSuite/TestSubtest/first")
	assert.Contains(t, suiteTester.SetupSubTestNames, "TestRunSuite/TestSubtest/second")

	assert.Contains(t, suiteTester.TearDownSubTestNames, "TestRunSuite/TestSubtest/first")
	assert.Contains(t, suiteTester.TearDownSubTestNames, "TestRunSuite/TestSubtest/second")

	for _, suiteName := range suiteTester.SuiteNameAfter {
		assert.Equal(t, "SuiteTester", suiteName)
	}

	for _, suiteName := range suiteTester.SuiteNameBefore {
		assert.Equal(t, "SuiteTester", suiteName)
	}

	for _, when := range suiteTester.TimeAfter {
		assert.False(t, when.IsZero())
	}

	for _, when := range suiteTester.TimeBefore {
		assert.False(t, when.IsZero())
	}

	// There are four test methods (TestOne, TestTwo, TestSkip, and TestSubtest), so
	// the SetupTest and TearDownTest methods (which should be run once for
	// each test) should have been run four times.
	assert.Equal(t, 4, suiteTester.SetupTestRunCount)
	assert.Equal(t, 4, suiteTester.TearDownTestRunCount)

	// Each test should have been run once.
	assert.Equal(t, 1, suiteTester.TestOneRunCount)
	assert.Equal(t, 1, suiteTester.TestTwoRunCount)
	assert.Equal(t, 1, suiteTester.TestSubtestRunCount)

	assert.Equal(t, 2, suiteTester.TearDownSubTestRunCount)
	assert.Equal(t, 2, suiteTester.SetupSubTestRunCount)

	// Methods that don't match the test method identifier shouldn't
	// have been run at all.
	assert.Equal(t, 0, suiteTester.NonTestMethodRunCount)

	suiteSkipTester := new(SuiteSkipTester)
	Run(t, suiteSkipTester)

	// The suite was only run once, so the SetupSuite and TearDownSuite
	// methods should have each been run only once, even though SetupSuite
	// called Skip()
	assert.Equal(t, 1, suiteSkipTester.SetupSuiteRunCount)
	assert.Equal(t, 1, suiteSkipTester.TearDownSuiteRunCount)

}

// This suite has no Test... methods. It's setup and teardown must be skipped.
type SuiteSetupSkipTester struct {
	Suite

	setUp    bool
	toreDown bool
}

func (s *SuiteSetupSkipTester) SetupSuite() {
	s.setUp = true
}

func (s *SuiteSetupSkipTester) NonTestMethod() {

}

func (s *SuiteSetupSkipTester) TearDownSuite() {
	s.toreDown = true
}

func TestSkippingSuiteSetup(t *testing.T) {
	suiteTester := new(SuiteSetupSkipTester)
	Run(t, suiteTester)
	assert.False(t, suiteTester.setUp)
	assert.False(t, suiteTester.toreDown)
}

func TestSuiteGetters(t *testing.T) {
	suite := new(SuiteTester)
	suite.SetT(t)
	assert.NotNil(t, suite.Assert())
	assert.Equal(t, suite.Assertions, suite.Assert())
	assert.NotNil(t, suite.Require())
	assert.Equal(t, suite.require, suite.Require())
}

type SuiteLoggingTester struct {
	Suite
}

func (s *SuiteLoggingTester) TestLoggingPass() {
	s.T().Log("TESTLOGPASS")
}

func (s *SuiteLoggingTester) TestLoggingFail() {
	s.T().Log("TESTLOGFAIL")
	assert.NotNil(s.T(), nil) // expected to fail
}

type StdoutCapture struct {
	oldStdout *os.File
	readPipe  *os.File
}

func (sc *StdoutCapture) StartCapture() {
	sc.oldStdout = os.Stdout
	sc.readPipe, os.Stdout, _ = os.Pipe()
}

func (sc *StdoutCapture) StopCapture() (string, error) {
	if sc.oldStdout == nil || sc.readPipe == nil {
		return "", errors.New("StartCapture not called before StopCapture")
	}
	os.Stdout.Close()
	os.Stdout = sc.oldStdout
	bytes, err := ioutil.ReadAll(sc.readPipe)
	if err != nil {
		return "", err
	}
	return string(bytes), nil
}

func TestSuiteLogging(t *testing.T) {
	suiteLoggingTester := new(SuiteLoggingTester)
	capture := StdoutCapture{}
	internalTest := testing.InternalTest{
		Name: t.Name() + "/SuiteLoggingTester",
		F: func(subT *testing.T) {
			Run(subT, suiteLoggingTester)
		},
	}
	capture.StartCapture()
	testing.RunTests(allTestsFilter, []testing.InternalTest{internalTest})
	output, err := capture.StopCapture()
	require.NoError(t, err, "Got an error trying to capture stdout and stderr!")
	require.NotEmpty(t, output, "output content must not be empty")

	// Failed tests' output is always printed
	assert.Contains(t, output, "TESTLOGFAIL")

	if testing.Verbose() {
		// In verbose mode, output from successful tests is also printed
		assert.Contains(t, output, "TESTLOGPASS")
	} else {
		assert.NotContains(t, output, "TESTLOGPASS")
	}
}

type CallOrderSuite struct {
	Suite
	callOrder []string
}

func (s *CallOrderSuite) call(method string) {
	time.Sleep(time.Duration(rand.Intn(300)) * time.Millisecond)
	s.callOrder = append(s.callOrder, method)
}

func TestSuiteCallOrder(t *testing.T) {
	Run(t, new(CallOrderSuite))
}
func (s *CallOrderSuite) SetupSuite() {
	s.call("SetupSuite")
}

func (s *CallOrderSuite) TearDownSuite() {
	s.call("TearDownSuite")
	assert.Equal(s.T(), "SetupSuite;SetupTest;Test A;SetupSubTest;SubTest A1;TearDownSubTest;SetupSubTest;SubTest A2;TearDownSubTest;TearDownTest;SetupTest;Test B;SetupSubTest;SubTest B1;TearDownSubTest;SetupSubTest;SubTest B2;TearDownSubTest;TearDownTest;TearDownSuite", strings.Join(s.callOrder, ";"))
}
func (s *CallOrderSuite) SetupTest() {
	s.call("SetupTest")
}

func (s *CallOrderSuite) TearDownTest() {
	s.call("TearDownTest")
}

func (s *CallOrderSuite) SetupSubTest() {
	s.call("SetupSubTest")
}

func (s *CallOrderSuite) TearDownSubTest() {
	s.call("TearDownSubTest")
}

func (s *CallOrderSuite) Test_A() {
	s.call("Test A")
	s.Run("SubTest A1", func() {
		s.call("SubTest A1")
	})
	s.Run("SubTest A2", func() {
		s.call("SubTest A2")
	})
}

func (s *CallOrderSuite) Test_B() {
	s.call("Test B")
	s.Run("SubTest B1", func() {
		s.call("SubTest B1")
	})
	s.Run("SubTest B2", func() {
		s.call("SubTest B2")
	})
}

type suiteWithStats struct {
	Suite
	wasCalled bool
	stats     *SuiteInformation
}

func (s *suiteWithStats) HandleStats(suiteName string, stats *SuiteInformation) {
	s.wasCalled = true
	s.stats = stats
}

func (s *suiteWithStats) TestSomething() {
	s.Equal(1, 1)
}

func (s *suiteWithStats) TestPanic() {
	panic("oops")
}

func TestSuiteWithStats(t *testing.T) {
	suiteWithStats := new(suiteWithStats)

	suiteSuccess := testing.RunTests(allTestsFilter, []testing.InternalTest{
		{
			Name: t.Name() + "/suiteWithStats",
			F: func(t *testing.T) {
				Run(t, suiteWithStats)
			},
		},
	})
	require.False(t, suiteSuccess, "suiteWithStats should report test failure because of panic in TestPanic")

	assert.True(t, suiteWithStats.wasCalled)
	assert.NotZero(t, suiteWithStats.stats.Start)
	assert.NotZero(t, suiteWithStats.stats.End)
	assert.False(t, suiteWithStats.stats.Passed())

	testStats := suiteWithStats.stats.TestStats

	assert.NotZero(t, testStats["TestSomething"].Start)
	assert.NotZero(t, testStats["TestSomething"].End)
	assert.True(t, testStats["TestSomething"].Passed)

	assert.NotZero(t, testStats["TestPanic"].Start)
	assert.NotZero(t, testStats["TestPanic"].End)
	assert.False(t, testStats["TestPanic"].Passed)
}

// FailfastSuite will test the behavior when running with the failfast flag
// It logs calls in the callOrder slice which we then use to assert the correct calls were made
type FailfastSuite struct {
	Suite
	callOrder []string
}

func (s *FailfastSuite) call(method string) {
	s.callOrder = append(s.callOrder, method)
}

func TestFailfastSuite(t *testing.T) {
	// This test suite is run twice. Once normally and once with the -failfast flag by TestFailfastSuiteFailFastOn
	// If you need to debug it run this test directly with the failfast flag set on/off as you need
	failFast := flag.Lookup("test.failfast").Value.(flag.Getter).Get().(bool)
	s := new(FailfastSuite)
	ok := testing.RunTests(
		allTestsFilter,
		[]testing.InternalTest{{
			Name: t.Name() + "/FailfastSuite",
			F: func(t *testing.T) {
				Run(t, s)
			},
		}},
	)
	assert.False(t, ok)
	if failFast {
		// Test A Fails and because we are running with failfast Test B never runs and we proceed straight to TearDownSuite
		assert.Equal(t, "SetupSuite;SetupTest;Test A Fails;TearDownTest;TearDownSuite", strings.Join(s.callOrder, ";"))
	} else {
		// Test A Fails and because we are running without failfast we continue and run Test B and then proceed to TearDownSuite
		assert.Equal(t, "SetupSuite;SetupTest;Test A Fails;TearDownTest;SetupTest;Test B Passes;TearDownTest;TearDownSuite", strings.Join(s.callOrder, ";"))
	}
}
func TestFailfastSuiteFailFastOn(t *testing.T) {
	// To test this with failfast on (and isolated from other intended test failures in our test suite) we launch it in its own process
	cmd := exec.Command("go", "test", "-v", "-race", "-run", "TestFailfastSuite", "-failfast")
	var out bytes.Buffer
	cmd.Stdout = &out
	t.Log("Running go test -v -race -run TestFailfastSuite -failfast")
	err := cmd.Run()
	t.Log(out.String())
	if err != nil {
		t.Log(err)
		t.Fail()
	}
}
func (s *FailfastSuite) SetupSuite() {
	s.call("SetupSuite")
}

func (s *FailfastSuite) TearDownSuite() {
	s.call("TearDownSuite")
}
func (s *FailfastSuite) SetupTest() {
	s.call("SetupTest")
}

func (s *FailfastSuite) TearDownTest() {
	s.call("TearDownTest")
}

func (s *FailfastSuite) Test_A_Fails() {
	s.call("Test A Fails")
	s.T().Error("Test A meant to fail")
}

func (s *FailfastSuite) Test_B_Passes() {
	s.call("Test B Passes")
	s.Require().True(true)
}

type subtestPanicSuite struct {
	Suite
	inTearDownSuite   bool
	inTearDownTest    bool
	inTearDownSubTest bool
}

func (s *subtestPanicSuite) TearDownSuite() {
	s.inTearDownSuite = true
}

func (s *subtestPanicSuite) TearDownTest() {
	s.inTearDownTest = true
}

func (s *subtestPanicSuite) TearDownSubTest() {
	s.inTearDownSubTest = true
}

func (s *subtestPanicSuite) TestSubtestPanic() {
	ok := s.Run("subtest", func() {
		panic("panic")
	})
	s.False(ok, "subtest failure is expected")
}

func TestSubtestPanic(t *testing.T) {
	suite := new(subtestPanicSuite)
	ok := testing.RunTests(
		allTestsFilter,
		[]testing.InternalTest{{
			Name: t.Name() + "/subtestPanicSuite",
			F: func(t *testing.T) {
				Run(t, suite)
			},
		}},
	)
	assert.False(t, ok, "TestSubtestPanic/subtest should make the testsuite fail")
	assert.True(t, suite.inTearDownSubTest)
	assert.True(t, suite.inTearDownTest)
	assert.True(t, suite.inTearDownSuite)
}

type unInitializedSuite struct {
	Suite
}

// TestUnInitializedSuites asserts the behavior of the suite methods when the
// suite is not initialized
func TestUnInitializedSuites(t *testing.T) {
	t.Run("should panic on Require", func(t *testing.T) {
		suite := new(unInitializedSuite)

		assert.Panics(t, func() {
			suite.Require().True(true)
		})
	})

	t.Run("should panic on Assert", func(t *testing.T) {
		suite := new(unInitializedSuite)

		assert.Panics(t, func() {
			suite.Assert().True(true)
		})
	})
}
