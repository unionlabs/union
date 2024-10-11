package subtest_test

import (
	"io"
	"net/http"
	"strings"
	"testing"

	"gotest.tools/v3/assert"
	"gotest.tools/v3/x/subtest"
)

var t = &testing.T{}

func ExampleRun_tableTest() {
	var testcases = []struct {
		data     io.Reader
		expected int
	}{
		{
			data:     strings.NewReader("invalid input"),
			expected: 400,
		},
		{
			data:     strings.NewReader("valid input"),
			expected: 200,
		},
	}

	for _, tc := range testcases {
		subtest.Run(t, "test-service-call", func(t subtest.TestContext) {
			// startFakeService can shutdown using t.AddCleanup
			url := startFakeService(t)

			req, err := http.NewRequest("POST", url, tc.data)
			assert.NilError(t, err)
			req = req.WithContext(t.Ctx())

			client := newClient(t)
			resp, err := client.Do(req)
			assert.NilError(t, err)
			assert.Equal(t, resp.StatusCode, tc.expected)
		})
	}
}

func startFakeService(t subtest.TestContext) string {
	return "url"
}

func newClient(_ subtest.TestContext) *http.Client {
	return &http.Client{}
}

func ExampleRun_testSuite() {
	// do suite setup before subtests

	subtest.Run(t, "test-one", func(t subtest.TestContext) {
		assert.Equal(t, 1, 1)
	})
	subtest.Run(t, "test-two", func(t subtest.TestContext) {
		assert.Equal(t, 2, 2)
	})

	// do suite teardown after subtests
}
