package testdata

import (
	"errors"
	"fmt"
)

var log = &logger{}

func LogThis(err error) {
	// Most common use cases with all types of formatted log methods.
	log.Debugf("Something really bad happened: %v", err)   // want "use log.WithError rather than templated log statements with errors"
	log.Infof("Something really bad happened: %v", err)    // want "use log.WithError rather than templated log statements with errors"
	log.Printf("Something really bad happened: %v", err)   // want "use log.WithError rather than templated log statements with errors"
	log.Warnf("Something really bad happened: %v", err)    // want "use log.WithError rather than templated log statements with errors"
	log.Warningf("Something really bad happened: %v", err) // want "use log.WithError rather than templated log statements with errors"
	log.Errorf("Something really bad happened: %v", err)   // want "use log.WithError rather than templated log statements with errors"
	log.Fatalf("Something really bad happened: %v", err)   // want "use log.WithError rather than templated log statements with errors"
	log.Panicf("Something really bad happened: %v", err)   // want "use log.WithError rather than templated log statements with errors"

	// Other common use cases.
	log.Panicf("Something really bad happened %d times: %v", 12, err) // want "use log.WithError rather than templated log statements with errors"

	if _, err := do(); err != nil {
		log.Errorf("Something really bad happened: %v", err) // want "use log.WithError rather than templated log statements with errors"
	}

	if ok, err := false, do2(); !ok || err != nil {
		log.Errorf("Something really bad happened: %v", err) // want "use log.WithError rather than templated log statements with errors"
	}

	log.WithError(err).Error("Something bad happened, but this log statement is OK :)")

	_ = fmt.Errorf("this is ok: %v", err)
}

func do() (bool, error) {
	return false, errors.New("bad")
}

func do2() error {
	return errors.New("bad2")
}

type logger struct{}

func (*logger) Debugf(format string, args ...interface{}) {
}

func (*logger) Infof(format string, args ...interface{}) {
}

func (*logger) Printf(format string, args ...interface{}) {
}

func (*logger) Warnf(format string, args ...interface{}) {
}

func (*logger) Warningf(format string, args ...interface{}) {
}

func (*logger) Errorf(format string, args ...interface{}) {
}

func (*logger) Error(msg string) {
}

func (*logger) Fatalf(format string, args ...interface{}) {
}

func (*logger) Panicf(format string, args ...interface{}) {
}

func (l *logger) WithError(err error) *logger {
	return l
}
