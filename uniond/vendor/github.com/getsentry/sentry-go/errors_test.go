package sentry

import (
	goErrors "github.com/go-errors/errors"
	pingcapErrors "github.com/pingcap/errors"
	pkgErrors "github.com/pkg/errors"
)

// NOTE: if you modify this file, you are also responsible for updating LoC position in Stacktrace tests

func Trace() *Stacktrace {
	return NewStacktrace()
}

func RedPkgErrorsRanger() error {
	return BluePkgErrorsRanger()
}

func BluePkgErrorsRanger() error {
	return pkgErrors.New("this is bad from pkgErrors")
}

func RedPingcapErrorsRanger() error {
	return BluePingcapErrorsRanger()
}

func BluePingcapErrorsRanger() error {
	return pingcapErrors.New("this is bad from pingcapErrors")
}

func RedGoErrorsRanger() error {
	return BlueGoErrorsRanger()
}

func BlueGoErrorsRanger() error {
	return goErrors.New("this is bad from goErrors")
}
