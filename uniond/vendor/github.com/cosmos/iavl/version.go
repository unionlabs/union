package iavl

import (
	"fmt"
	"runtime"
)

// Version of iavl. Fill in fields with build flags
var (
	Version = ""
	Commit  = ""
	Branch  = ""
)

// VersionInfo contains useful versioning information in struct
type VersionInfo struct {
	IAVL      string `json:"iavl"`
	GitCommit string `json:"commit"`
	Branch    string `json:"branch"`
	GoVersion string `json:"go"`
}

func (v VersionInfo) String() string {
	return fmt.Sprintf(`iavl: %s
git commit: %s
git branch: %s
%s`, v.IAVL, v.GitCommit, v.Branch, v.GoVersion)
}

// Returns VersionInfo with global vars filled in
func GetVersionInfo() VersionInfo {
	return VersionInfo{
		Version,
		Commit,
		Branch,
		fmt.Sprintf("go version %s %s/%s\n", runtime.Version(), runtime.GOOS, runtime.GOARCH),
	}
}
