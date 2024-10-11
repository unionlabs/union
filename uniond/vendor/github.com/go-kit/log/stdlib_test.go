package log

import (
	"bytes"
	"fmt"
	"log"
	"testing"
	"time"
)

func TestStdlibWriter(t *testing.T) {
	buf := &bytes.Buffer{}
	log.SetOutput(buf)
	log.SetFlags(log.LstdFlags)
	logger := NewLogfmtLogger(StdlibWriter{})
	logger.Log("key", "val")
	timestamp := time.Now().Format("2006/01/02 15:04:05")
	if want, have := timestamp+" key=val\n", buf.String(); want != have {
		t.Errorf("want %q, have %q", want, have)
	}
}

func TestStdlibAdapterUsage(t *testing.T) {
	buf := &bytes.Buffer{}
	logger := NewLogfmtLogger(buf)
	writer := NewStdlibAdapter(logger)
	stdlog := log.New(writer, "", 0)

	now := time.Now()
	date := now.Format("2006/01/02")
	time := now.Format("15:04:05")

	for flag, want := range map[int]string{
		0:                                      "msg=hello\n",
		log.Ldate:                              "ts=" + date + " msg=hello\n",
		log.Ltime:                              "ts=" + time + " msg=hello\n",
		log.Ldate | log.Ltime:                  "ts=\"" + date + " " + time + "\" msg=hello\n",
		log.Lshortfile:                         "caller=stdlib_test.go:44 msg=hello\n",
		log.Lshortfile | log.Ldate:             "ts=" + date + " caller=stdlib_test.go:44 msg=hello\n",
		log.Lshortfile | log.Ldate | log.Ltime: "ts=\"" + date + " " + time + "\" caller=stdlib_test.go:44 msg=hello\n",
	} {
		buf.Reset()
		stdlog.SetFlags(flag)
		stdlog.Print("hello")
		if have := buf.String(); want != have {
			t.Errorf("flag=%d: want %#v, have %#v", flag, want, have)
		}
	}
}

func TestStdLibAdapterExtraction(t *testing.T) {
	buf := &bytes.Buffer{}
	logger := NewLogfmtLogger(buf)
	writer := NewStdlibAdapter(logger)
	for input, want := range map[string]string{
		"hello":                             "msg=hello\n",
		"2009/01/23: hello":                 "ts=2009/01/23 msg=hello\n",
		"2009/01/23 01:23:23: hello":        "ts=\"2009/01/23 01:23:23\" msg=hello\n",
		"01:23:23: hello":                   "ts=01:23:23 msg=hello\n",
		"2009/01/23 01:23:23.123123: hello": "ts=\"2009/01/23 01:23:23.123123\" msg=hello\n",
		"2009/01/23 01:23:23.123123 /a/b/c/d.go:23: hello": "ts=\"2009/01/23 01:23:23.123123\" caller=/a/b/c/d.go:23 msg=hello\n",
		"01:23:23.123123 /a/b/c/d.go:23: hello":            "ts=01:23:23.123123 caller=/a/b/c/d.go:23 msg=hello\n",
		"2009/01/23 01:23:23 /a/b/c/d.go:23: hello":        "ts=\"2009/01/23 01:23:23\" caller=/a/b/c/d.go:23 msg=hello\n",
		"2009/01/23 /a/b/c/d.go:23: hello":                 "ts=2009/01/23 caller=/a/b/c/d.go:23 msg=hello\n",
		"/a/b/c/d.go:23: hello":                            "caller=/a/b/c/d.go:23 msg=hello\n",
	} {
		buf.Reset()
		fmt.Fprint(writer, input)
		if have := buf.String(); want != have {
			t.Errorf("%q: want %#v, have %#v", input, want, have)
		}
	}
}

func TestStdLibAdapterPrefixedExtraction(t *testing.T) {
	buf := &bytes.Buffer{}
	logger := NewLogfmtLogger(buf)
	writer := NewStdlibAdapter(logger, Prefix("some prefix ", false))
	for input, want := range map[string]string{
		"some prefix hello":                                            "msg=hello\n",
		"some prefix 2009/01/23: hello":                                "ts=2009/01/23 msg=hello\n",
		"some prefix 2009/01/23 01:23:23: hello":                       "ts=\"2009/01/23 01:23:23\" msg=hello\n",
		"some prefix 01:23:23: hello":                                  "ts=01:23:23 msg=hello\n",
		"some prefix 2009/01/23 01:23:23.123123: hello":                "ts=\"2009/01/23 01:23:23.123123\" msg=hello\n",
		"some prefix 2009/01/23 01:23:23.123123 /a/b/c/d.go:23: hello": "ts=\"2009/01/23 01:23:23.123123\" caller=/a/b/c/d.go:23 msg=hello\n",
		"some prefix 01:23:23.123123 /a/b/c/d.go:23: hello":            "ts=01:23:23.123123 caller=/a/b/c/d.go:23 msg=hello\n",
		"some prefix 2009/01/23 01:23:23 /a/b/c/d.go:23: hello":        "ts=\"2009/01/23 01:23:23\" caller=/a/b/c/d.go:23 msg=hello\n",
		"some prefix 2009/01/23 /a/b/c/d.go:23: hello":                 "ts=2009/01/23 caller=/a/b/c/d.go:23 msg=hello\n",
		"some prefix /a/b/c/d.go:23: hello":                            "caller=/a/b/c/d.go:23 msg=hello\n",
		"/a/b/c/d.go:23: some prefix hello":                            "caller=/a/b/c/d.go:23 msg=hello\n",
	} {
		buf.Reset()
		fmt.Fprint(writer, input)
		if have := buf.String(); want != have {
			t.Errorf("%q: want %#v, have %#v", input, want, have)
		}
	}
}

func TestStdLibAdapterPrefixedExtractionWithJoinToMessage(t *testing.T) {
	buf := &bytes.Buffer{}
	logger := NewLogfmtLogger(buf)
	writer := NewStdlibAdapter(logger, Prefix("some prefix ", true))
	for input, want := range map[string]string{
		"some prefix hello":                                            "msg=\"some prefix hello\"\n",
		"some prefix 2009/01/23: hello":                                "ts=2009/01/23 msg=\"some prefix hello\"\n",
		"some prefix 2009/01/23 01:23:23: hello":                       "ts=\"2009/01/23 01:23:23\" msg=\"some prefix hello\"\n",
		"some prefix 01:23:23: hello":                                  "ts=01:23:23 msg=\"some prefix hello\"\n",
		"some prefix 2009/01/23 01:23:23.123123: hello":                "ts=\"2009/01/23 01:23:23.123123\" msg=\"some prefix hello\"\n",
		"some prefix 2009/01/23 01:23:23.123123 /a/b/c/d.go:23: hello": "ts=\"2009/01/23 01:23:23.123123\" caller=/a/b/c/d.go:23 msg=\"some prefix hello\"\n",
		"some prefix 01:23:23.123123 /a/b/c/d.go:23: hello":            "ts=01:23:23.123123 caller=/a/b/c/d.go:23 msg=\"some prefix hello\"\n",
		"some prefix 2009/01/23 01:23:23 /a/b/c/d.go:23: hello":        "ts=\"2009/01/23 01:23:23\" caller=/a/b/c/d.go:23 msg=\"some prefix hello\"\n",
		"some prefix 2009/01/23 /a/b/c/d.go:23: hello":                 "ts=2009/01/23 caller=/a/b/c/d.go:23 msg=\"some prefix hello\"\n",
		"some prefix /a/b/c/d.go:23: hello":                            "caller=/a/b/c/d.go:23 msg=\"some prefix hello\"\n",
		"/a/b/c/d.go:23: some prefix hello":                            "caller=/a/b/c/d.go:23 msg=\"some prefix hello\"\n",
	} {
		buf.Reset()
		fmt.Fprint(writer, input)
		if have := buf.String(); want != have {
			t.Errorf("%q: want %#v, have %#v", input, want, have)
		}
	}
}

func TestStdlibAdapterSubexps(t *testing.T) {
	for input, wantMap := range map[string]map[string]string{
		"hello world": {
			"date": "",
			"time": "",
			"file": "",
			"msg":  "hello world",
		},
		"hello\nworld": {
			"date": "",
			"time": "",
			"file": "",
			"msg":  "hello\nworld",
		},
		"2009/01/23: hello world": {
			"date": "2009/01/23",
			"time": "",
			"file": "",
			"msg":  "hello world",
		},
		"2009/01/23 01:23:23: hello world": {
			"date": "2009/01/23",
			"time": "01:23:23",
			"file": "",
			"msg":  "hello world",
		},
		"01:23:23: hello world": {
			"date": "",
			"time": "01:23:23",
			"file": "",
			"msg":  "hello world",
		},
		"2009/01/23 01:23:23.123123: hello world": {
			"date": "2009/01/23",
			"time": "01:23:23.123123",
			"file": "",
			"msg":  "hello world",
		},
		"2009/01/23 01:23:23.123123 /a/b/c/d.go:23: hello world": {
			"date": "2009/01/23",
			"time": "01:23:23.123123",
			"file": "/a/b/c/d.go:23",
			"msg":  "hello world",
		},
		"01:23:23.123123 /a/b/c/d.go:23: hello world": {
			"date": "",
			"time": "01:23:23.123123",
			"file": "/a/b/c/d.go:23",
			"msg":  "hello world",
		},
		"2009/01/23 01:23:23 /a/b/c/d.go:23: hello world": {
			"date": "2009/01/23",
			"time": "01:23:23",
			"file": "/a/b/c/d.go:23",
			"msg":  "hello world",
		},
		"2009/01/23 /a/b/c/d.go:23: hello world": {
			"date": "2009/01/23",
			"time": "",
			"file": "/a/b/c/d.go:23",
			"msg":  "hello world",
		},
		"/a/b/c/d.go:23: hello world": {
			"date": "",
			"time": "",
			"file": "/a/b/c/d.go:23",
			"msg":  "hello world",
		},
		"2009/01/23 01:23:23.123123 C:/a/b/c/d.go:23: hello world": {
			"date": "2009/01/23",
			"time": "01:23:23.123123",
			"file": "C:/a/b/c/d.go:23",
			"msg":  "hello world",
		},
		"01:23:23.123123 C:/a/b/c/d.go:23: hello world": {
			"date": "",
			"time": "01:23:23.123123",
			"file": "C:/a/b/c/d.go:23",
			"msg":  "hello world",
		},
		"2009/01/23 01:23:23 C:/a/b/c/d.go:23: hello world": {
			"date": "2009/01/23",
			"time": "01:23:23",
			"file": "C:/a/b/c/d.go:23",
			"msg":  "hello world",
		},
		"2009/01/23 C:/a/b/c/d.go:23: hello world": {
			"date": "2009/01/23",
			"time": "",
			"file": "C:/a/b/c/d.go:23",
			"msg":  "hello world",
		},
		"C:/a/b/c/d.go:23: hello world": {
			"date": "",
			"time": "",
			"file": "C:/a/b/c/d.go:23",
			"msg":  "hello world",
		},
		"2009/01/23 01:23:23.123123 C:/a/b/c/d.go:23: :.;<>_#{[]}\"\\": {
			"date": "2009/01/23",
			"time": "01:23:23.123123",
			"file": "C:/a/b/c/d.go:23",
			"msg":  ":.;<>_#{[]}\"\\",
		},
		"01:23:23.123123 C:/a/b/c/d.go:23: :.;<>_#{[]}\"\\": {
			"date": "",
			"time": "01:23:23.123123",
			"file": "C:/a/b/c/d.go:23",
			"msg":  ":.;<>_#{[]}\"\\",
		},
		"2009/01/23 01:23:23 C:/a/b/c/d.go:23: :.;<>_#{[]}\"\\": {
			"date": "2009/01/23",
			"time": "01:23:23",
			"file": "C:/a/b/c/d.go:23",
			"msg":  ":.;<>_#{[]}\"\\",
		},
		"2009/01/23 C:/a/b/c/d.go:23: :.;<>_#{[]}\"\\": {
			"date": "2009/01/23",
			"time": "",
			"file": "C:/a/b/c/d.go:23",
			"msg":  ":.;<>_#{[]}\"\\",
		},
		"C:/a/b/c/d.go:23: :.;<>_#{[]}\"\\": {
			"date": "",
			"time": "",
			"file": "C:/a/b/c/d.go:23",
			"msg":  ":.;<>_#{[]}\"\\",
		},
	} {
		haveMap := subexps([]byte(input))
		for key, want := range wantMap {
			if have := haveMap[key]; want != have {
				t.Errorf("%q: %q: want %q, have %q", input, key, want, have)
			}
		}
	}
}
