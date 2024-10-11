package sentry

import (
	"github.com/getsentry/sentry-go/internal/testutils"
)

var assertEqual = testutils.AssertEqual
var assertNotEqual = testutils.AssertNotEqual
var assertBaggageStringsEqual = testutils.AssertBaggageStringsEqual
