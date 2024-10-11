package logrus

import (
	"bytes"
	"context"
	"fmt"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

func TestEntryWithError(t *testing.T) {

	assert := assert.New(t)

	defer func() {
		ErrorKey = "error"
	}()

	err := fmt.Errorf("kaboom at layer %d", 4711)

	assert.Equal(err, WithError(err).Data["error"])

	logger := New()
	logger.Out = &bytes.Buffer{}
	entry := NewEntry(logger)

	assert.Equal(err, entry.WithError(err).Data["error"])

	ErrorKey = "err"

	assert.Equal(err, entry.WithError(err).Data["err"])

}

func TestEntryWithContext(t *testing.T) {
	assert := assert.New(t)
	ctx := context.WithValue(context.Background(), "foo", "bar")

	assert.Equal(ctx, WithContext(ctx).Context)

	logger := New()
	logger.Out = &bytes.Buffer{}
	entry := NewEntry(logger)

	assert.Equal(ctx, entry.WithContext(ctx).Context)
}

func TestEntryWithContextCopiesData(t *testing.T) {
	assert := assert.New(t)

	// Initialize a parent Entry object with a key/value set in its Data map
	logger := New()
	logger.Out = &bytes.Buffer{}
	parentEntry := NewEntry(logger).WithField("parentKey", "parentValue")

	// Create two children Entry objects from the parent in different contexts
	ctx1 := context.WithValue(context.Background(), "foo", "bar")
	childEntry1 := parentEntry.WithContext(ctx1)
	assert.Equal(ctx1, childEntry1.Context)

	ctx2 := context.WithValue(context.Background(), "bar", "baz")
	childEntry2 := parentEntry.WithContext(ctx2)
	assert.Equal(ctx2, childEntry2.Context)
	assert.NotEqual(ctx1, ctx2)

	// Ensure that data set in the parent Entry are preserved to both children
	assert.Equal("parentValue", childEntry1.Data["parentKey"])
	assert.Equal("parentValue", childEntry2.Data["parentKey"])

	// Modify data stored in the child entry
	childEntry1.Data["childKey"] = "childValue"

	// Verify that data is successfully stored in the child it was set on
	val, exists := childEntry1.Data["childKey"]
	assert.True(exists)
	assert.Equal("childValue", val)

	// Verify that the data change to child 1 has not affected its sibling
	val, exists = childEntry2.Data["childKey"]
	assert.False(exists)
	assert.Empty(val)

	// Verify that the data change to child 1 has not affected its parent
	val, exists = parentEntry.Data["childKey"]
	assert.False(exists)
	assert.Empty(val)
}

func TestEntryWithTimeCopiesData(t *testing.T) {
	assert := assert.New(t)

	// Initialize a parent Entry object with a key/value set in its Data map
	logger := New()
	logger.Out = &bytes.Buffer{}
	parentEntry := NewEntry(logger).WithField("parentKey", "parentValue")

	// Create two children Entry objects from the parent with two different times
	childEntry1 := parentEntry.WithTime(time.Now().AddDate(0, 0, 1))
	childEntry2 := parentEntry.WithTime(time.Now().AddDate(0, 0, 2))

	// Ensure that data set in the parent Entry are preserved to both children
	assert.Equal("parentValue", childEntry1.Data["parentKey"])
	assert.Equal("parentValue", childEntry2.Data["parentKey"])

	// Modify data stored in the child entry
	childEntry1.Data["childKey"] = "childValue"

	// Verify that data is successfully stored in the child it was set on
	val, exists := childEntry1.Data["childKey"]
	assert.True(exists)
	assert.Equal("childValue", val)

	// Verify that the data change to child 1 has not affected its sibling
	val, exists = childEntry2.Data["childKey"]
	assert.False(exists)
	assert.Empty(val)

	// Verify that the data change to child 1 has not affected its parent
	val, exists = parentEntry.Data["childKey"]
	assert.False(exists)
	assert.Empty(val)
}

func TestEntryPanicln(t *testing.T) {
	errBoom := fmt.Errorf("boom time")

	defer func() {
		p := recover()
		assert.NotNil(t, p)

		switch pVal := p.(type) {
		case *Entry:
			assert.Equal(t, "kaboom", pVal.Message)
			assert.Equal(t, errBoom, pVal.Data["err"])
		default:
			t.Fatalf("want type *Entry, got %T: %#v", pVal, pVal)
		}
	}()

	logger := New()
	logger.Out = &bytes.Buffer{}
	entry := NewEntry(logger)
	entry.WithField("err", errBoom).Panicln("kaboom")
}

func TestEntryPanicf(t *testing.T) {
	errBoom := fmt.Errorf("boom again")

	defer func() {
		p := recover()
		assert.NotNil(t, p)

		switch pVal := p.(type) {
		case *Entry:
			assert.Equal(t, "kaboom true", pVal.Message)
			assert.Equal(t, errBoom, pVal.Data["err"])
		default:
			t.Fatalf("want type *Entry, got %T: %#v", pVal, pVal)
		}
	}()

	logger := New()
	logger.Out = &bytes.Buffer{}
	entry := NewEntry(logger)
	entry.WithField("err", errBoom).Panicf("kaboom %v", true)
}

func TestEntryPanic(t *testing.T) {
	errBoom := fmt.Errorf("boom again")

	defer func() {
		p := recover()
		assert.NotNil(t, p)

		switch pVal := p.(type) {
		case *Entry:
			assert.Equal(t, "kaboom", pVal.Message)
			assert.Equal(t, errBoom, pVal.Data["err"])
		default:
			t.Fatalf("want type *Entry, got %T: %#v", pVal, pVal)
		}
	}()

	logger := New()
	logger.Out = &bytes.Buffer{}
	entry := NewEntry(logger)
	entry.WithField("err", errBoom).Panic("kaboom")
}

const (
	badMessage   = "this is going to panic"
	panicMessage = "this is broken"
)

type panickyHook struct{}

func (p *panickyHook) Levels() []Level {
	return []Level{InfoLevel}
}

func (p *panickyHook) Fire(entry *Entry) error {
	if entry.Message == badMessage {
		panic(panicMessage)
	}

	return nil
}

func TestEntryHooksPanic(t *testing.T) {
	logger := New()
	logger.Out = &bytes.Buffer{}
	logger.Level = InfoLevel
	logger.Hooks.Add(&panickyHook{})

	defer func() {
		p := recover()
		assert.NotNil(t, p)
		assert.Equal(t, panicMessage, p)

		entry := NewEntry(logger)
		entry.Info("another message")
	}()

	entry := NewEntry(logger)
	entry.Info(badMessage)
}

func TestEntryWithIncorrectField(t *testing.T) {
	assert := assert.New(t)

	fn := func() {}

	e := Entry{Logger: New()}
	eWithFunc := e.WithFields(Fields{"func": fn})
	eWithFuncPtr := e.WithFields(Fields{"funcPtr": &fn})

	assert.Equal(eWithFunc.err, `can not add field "func"`)
	assert.Equal(eWithFuncPtr.err, `can not add field "funcPtr"`)

	eWithFunc = eWithFunc.WithField("not_a_func", "it is a string")
	eWithFuncPtr = eWithFuncPtr.WithField("not_a_func", "it is a string")

	assert.Equal(eWithFunc.err, `can not add field "func"`)
	assert.Equal(eWithFuncPtr.err, `can not add field "funcPtr"`)

	eWithFunc = eWithFunc.WithTime(time.Now())
	eWithFuncPtr = eWithFuncPtr.WithTime(time.Now())

	assert.Equal(eWithFunc.err, `can not add field "func"`)
	assert.Equal(eWithFuncPtr.err, `can not add field "funcPtr"`)
}

func TestEntryLogfLevel(t *testing.T) {
	logger := New()
	buffer := &bytes.Buffer{}
	logger.Out = buffer
	logger.SetLevel(InfoLevel)
	entry := NewEntry(logger)

	entry.Logf(DebugLevel, "%s", "debug")
	assert.NotContains(t, buffer.String(), "debug")

	entry.Logf(WarnLevel, "%s", "warn")
	assert.Contains(t, buffer.String(), "warn")
}

func TestEntryReportCallerRace(t *testing.T) {
	logger := New()
	entry := NewEntry(logger)

	// logging before SetReportCaller has the highest chance of causing a race condition
	// to be detected, but doing it twice just to increase the likelyhood of detecting the race
	go func() {
		entry.Info("should not race")
	}()
	go func() {
		logger.SetReportCaller(true)
	}()
	go func() {
		entry.Info("should not race")
	}()
}

func TestEntryFormatterRace(t *testing.T) {
	logger := New()
	entry := NewEntry(logger)

	// logging before SetReportCaller has the highest chance of causing a race condition
	// to be detected, but doing it twice just to increase the likelyhood of detecting the race
	go func() {
		entry.Info("should not race")
	}()
	go func() {
		logger.SetFormatter(&TextFormatter{})
	}()
	go func() {
		entry.Info("should not race")
	}()
}
