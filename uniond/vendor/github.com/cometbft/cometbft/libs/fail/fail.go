package fail

import (
	"fmt"
	"os"
	"strconv"
)

func envSet() int {
	callIndexToFailS := os.Getenv("FAIL_TEST_INDEX")

	if callIndexToFailS == "" {
		return -1
	}

	var err error
	callIndexToFail, err := strconv.Atoi(callIndexToFailS)
	if err != nil {
		return -1
	}

	return callIndexToFail
}

// Fail when FAIL_TEST_INDEX == callIndex
var callIndex int // indexes Fail calls

func Fail() {
	callIndexToFail := envSet()
	if callIndexToFail < 0 {
		return
	}

	if callIndex == callIndexToFail {
		Exit()
	}

	callIndex++
}

func Exit() {
	fmt.Printf("*** fail-test %d ***\n", callIndex)
	os.Exit(1)
	//	proc, _ := os.FindProcess(os.Getpid())
	//	proc.Signal(os.Interrupt)
	//	panic(fmt.Sprintf("*** fail-test %d ***", callIndex))
}
