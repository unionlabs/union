package circonus

import (
	"errors"
	"fmt"
	"io/ioutil"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/hashicorp/go-metrics"
)

func TestNewCirconusSink(t *testing.T) {

	// test with invalid config (nil)
	expectedError := errors.New("invalid check manager configuration (no API token AND no submission url)")
	_, err := NewCirconusSink(nil)
	if err == nil || !strings.Contains(err.Error(), expectedError.Error()) {
		t.Errorf("Expected an '%#v' error, got '%#v'", expectedError, err)
	}

	// test w/submission url and w/o token
	cfg := &Config{}
	cfg.CheckManager.Check.SubmissionURL = "http://127.0.0.1:43191/"
	_, err = NewCirconusSink(cfg)
	if err != nil {
		t.Errorf("Expected no error, got '%v'", err)
	}

	// note: a test with a valid token is *not* done as it *will* create a
	//       check resulting in testing the api more than the circonus sink
	// see circonus-gometrics/checkmgr/checkmgr_test.go for testing of api token
}

func TestFlattenKey(t *testing.T) {
	var testKeys = []struct {
		input    []string
		expected string
	}{
		{[]string{"a", "b", "c"}, "a`b`c"},
		{[]string{"a-a", "b_b", "c/c"}, "a-a`b_b`c/c"},
		{[]string{"spaces must", "flatten", "to", "underscores"}, "spaces_must`flatten`to`underscores"},
	}

	c := &CirconusSink{}

	for _, test := range testKeys {
		if actual := c.flattenKey(test.input); actual != test.expected {
			t.Fatalf("Flattening %v failed, expected '%s' got '%s'", test.input, test.expected, actual)
		}
	}
}

func fakeBroker(q chan string) *httptest.Server {
	handler := func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(200)
		w.Header().Set("Content-Type", "application/json")
		defer r.Body.Close()
		body, err := ioutil.ReadAll(r.Body)
		if err != nil {
			q <- err.Error()
			fmt.Fprintln(w, err.Error())
		} else {
			q <- string(body)
			fmt.Fprintln(w, `{"stats":1}`)
		}
	}

	return httptest.NewServer(http.HandlerFunc(handler))
}

func TestSetGauge(t *testing.T) {
	q := make(chan string)

	server := fakeBroker(q)
	defer server.Close()

	cfg := &Config{}
	cfg.CheckManager.Check.SubmissionURL = server.URL

	cs, err := NewCirconusSink(cfg)
	if err != nil {
		t.Errorf("Expected no error, got '%v'", err)
	}

	go func() {
		cs.SetGauge([]string{"foo", "bar"}, 1)
		cs.Flush()
	}()

	expect := "{\"foo`bar\":{\"_type\":\"l\",\"_value\":1}}"
	actual := <-q

	if actual != expect {
		t.Errorf("Expected '%s', got '%s'", expect, actual)

	}
}

func TestSetPrecisionGauge(t *testing.T) {
	q := make(chan string)

	server := fakeBroker(q)
	defer server.Close()

	cfg := &Config{}
	cfg.CheckManager.Check.SubmissionURL = server.URL

	cs, err := NewCirconusSink(cfg)
	if err != nil {
		t.Errorf("Expected no error, got '%v'", err)
	}

	go func() {
		cs.SetPrecisionGauge([]string{"foo", "bar"}, 1)
		cs.Flush()
	}()

	expect := "{\"foo`bar\":{\"_type\":\"n\",\"_value\":1}}"
	actual := <-q

	if actual != expect {
		t.Errorf("Expected '%s', got '%s'", expect, actual)

	}
}

func TestIncrCounter(t *testing.T) {
	q := make(chan string)

	server := fakeBroker(q)
	defer server.Close()

	cfg := &Config{}
	cfg.CheckManager.Check.SubmissionURL = server.URL

	cs, err := NewCirconusSink(cfg)
	if err != nil {
		t.Errorf("Expected no error, got '%v'", err)
	}

	go func() {
		cs.IncrCounter([]string{"foo", "bar"}, 1)
		cs.Flush()
	}()

	expect := "{\"foo`bar\":{\"_type\":\"L\",\"_value\":1}}"
	actual := <-q

	if actual != expect {
		t.Errorf("Expected '%s', got '%s'", expect, actual)

	}
}

func TestAddSample(t *testing.T) {
	q := make(chan string)

	server := fakeBroker(q)
	defer server.Close()

	cfg := &Config{}
	cfg.CheckManager.Check.SubmissionURL = server.URL

	cs, err := NewCirconusSink(cfg)
	if err != nil {
		t.Errorf("Expected no error, got '%v'", err)
	}

	go func() {
		cs.AddSample([]string{"foo", "bar"}, 1)
		cs.Flush()
	}()

	expect := "{\"foo`bar\":{\"_type\":\"n\",\"_value\":[\"H[1.0e+00]=1\"]}}"
	actual := <-q

	if actual != expect {
		t.Errorf("Expected '%s', got '%s'", expect, actual)

	}
}

func TestMetricSinkInterface(t *testing.T) {
	var cs *CirconusSink
	_ = metrics.MetricSink(cs)
}
