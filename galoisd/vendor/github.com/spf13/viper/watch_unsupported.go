//go:build appengine || (!darwin && !dragonfly && !freebsd && !openbsd && !linux && !netbsd && !solaris && !windows)
// +build appengine !darwin,!dragonfly,!freebsd,!openbsd,!linux,!netbsd,!solaris,!windows

package viper

import (
	"fmt"
	"runtime"

	"github.com/fsnotify/fsnotify"
)

func newWatcher() (*watcher, error) {
	return &watcher{}, fmt.Errorf("fsnotify not supported on %s", runtime.GOOS)
}

type watcher struct {
	Events chan fsnotify.Event
	Errors chan error
}

func (*watcher) Close() error {
	return nil
}

func (*watcher) Add(name string) error {
	return nil
}

func (*watcher) Remove(name string) error {
	return nil
}
