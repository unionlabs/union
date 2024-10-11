#!/bin/sh

# This script uses gocov to generate a test coverage report.
# The gocov tool my be obtained with the following command:
#   go get github.com/axw/gocov/gocov
#
# It will be installed to $GOPATH/bin, so ensure that location is in your $PATH.

# Check for gocov.
type gocov >/dev/null 2>&1
if [ $? -ne 0 ]; then
	echo >&2 "This script requires the gocov tool."
	echo >&2 "You may obtain it with the following command:"
	echo >&2 "go get github.com/axw/gocov/gocov"
	exit 1
fi
gocov test | gocov report
