package icmd_test

import (
	"testing"

	"gotest.tools/v3/icmd"
)

var t = &testing.T{}

func ExampleRunCommand() {
	result := icmd.RunCommand("bash", "-c", "echo all good")
	result.Assert(t, icmd.Success)
}

func ExampleRunCmd() {
	result := icmd.RunCmd(icmd.Command("cat", "/does/not/exist"))
	result.Assert(t, icmd.Expected{
		ExitCode: 1,
		Err:      "cat: /does/not/exist: No such file or directory",
	})
}
