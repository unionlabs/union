//go:build darwin || dragonfly || freebsd || openbsd || linux || netbsd || solaris || windows
// +build darwin dragonfly freebsd openbsd linux netbsd solaris windows

package viper

import "github.com/fsnotify/fsnotify"

type watcher = fsnotify.Watcher

func newWatcher() (*watcher, error) {
	return fsnotify.NewWatcher()
}
