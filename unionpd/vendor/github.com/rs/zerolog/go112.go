// +build go1.12

package zerolog

// Since go 1.12, some auto generated init functions are hidden from
// runtime.Caller.
const contextCallerSkipFrameCount = 2
