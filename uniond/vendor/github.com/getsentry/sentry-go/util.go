package sentry

import (
	"crypto/rand"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"strings"
	"time"

	exec "golang.org/x/sys/execabs"
)

func uuid() string {
	id := make([]byte, 16)
	// Prefer rand.Read over rand.Reader, see https://go-review.googlesource.com/c/go/+/272326/.
	_, _ = rand.Read(id)
	id[6] &= 0x0F // clear version
	id[6] |= 0x40 // set version to 4 (random uuid)
	id[8] &= 0x3F // clear variant
	id[8] |= 0x80 // set to IETF variant
	return hex.EncodeToString(id)
}

func fileExists(fileName string) bool {
	_, err := os.Stat(fileName)
	return err == nil
}

// monotonicTimeSince replaces uses of time.Now() to take into account the
// monotonic clock reading stored in start, such that duration = end - start is
// unaffected by changes in the system wall clock.
func monotonicTimeSince(start time.Time) (end time.Time) {
	return start.Add(time.Since(start))
}

// nolint: deadcode, unused
func prettyPrint(data interface{}) {
	dbg, _ := json.MarshalIndent(data, "", "  ")
	fmt.Println(string(dbg))
}

// defaultRelease attempts to guess a default release for the currently running
// program.
func defaultRelease() (release string) {
	// Return first non-empty environment variable known to hold release info, if any.
	envs := []string{
		"SENTRY_RELEASE",
		"HEROKU_SLUG_COMMIT",
		"SOURCE_VERSION",
		"CODEBUILD_RESOLVED_SOURCE_VERSION",
		"CIRCLE_SHA1",
		"GAE_DEPLOYMENT_ID",
		"GITHUB_SHA",             // GitHub Actions - https://help.github.com/en/actions
		"COMMIT_REF",             // Netlify - https://docs.netlify.com/
		"VERCEL_GIT_COMMIT_SHA",  // Vercel - https://vercel.com/
		"ZEIT_GITHUB_COMMIT_SHA", // Zeit (now known as Vercel)
		"ZEIT_GITLAB_COMMIT_SHA",
		"ZEIT_BITBUCKET_COMMIT_SHA",
	}
	for _, e := range envs {
		if release = os.Getenv(e); release != "" {
			Logger.Printf("Using release from environment variable %s: %s", e, release)
			return release
		}
	}

	// Derive a version string from Git. Example outputs:
	// 	v1.0.1-0-g9de4
	// 	v2.0-8-g77df-dirty
	// 	4f72d7
	cmd := exec.Command("git", "describe", "--long", "--always", "--dirty")
	b, err := cmd.Output()
	if err != nil {
		// Either Git is not available or the current directory is not a
		// Git repository.
		var s strings.Builder
		fmt.Fprintf(&s, "Release detection failed: %v", err)
		if err, ok := err.(*exec.ExitError); ok && len(err.Stderr) > 0 {
			fmt.Fprintf(&s, ": %s", err.Stderr)
		}
		Logger.Print(s.String())
		Logger.Print("Some Sentry features will not be available. See https://docs.sentry.io/product/releases/.")
		Logger.Print("To stop seeing this message, pass a Release to sentry.Init or set the SENTRY_RELEASE environment variable.")
		return ""
	}
	release = strings.TrimSpace(string(b))
	Logger.Printf("Using release from Git: %s", release)
	return release
}
