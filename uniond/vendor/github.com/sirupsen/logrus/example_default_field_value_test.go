package logrus_test

import (
	"os"

	"github.com/sirupsen/logrus"
)

type DefaultFieldHook struct {
	GetValue func() string
}

func (h *DefaultFieldHook) Levels() []logrus.Level {
	return logrus.AllLevels
}

func (h *DefaultFieldHook) Fire(e *logrus.Entry) error {
	e.Data["aDefaultField"] = h.GetValue()
	return nil
}

func ExampleDefaultFieldHook() {
	l := logrus.New()
	l.Out = os.Stdout
	l.Formatter = &logrus.TextFormatter{DisableTimestamp: true, DisableColors: true}

	l.AddHook(&DefaultFieldHook{GetValue: func() string { return "with its default value" }})
	l.Info("first log")
	// Output:
	// level=info msg="first log" aDefaultField="with its default value"
}
