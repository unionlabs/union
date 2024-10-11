package components

import (
	"github.com/sirupsen/logrus"
)

var log = logrus.WithField("prefix", "components")

func init() {
	logrus.SetReportCaller(true)
}
