// Code created by gotmpl. DO NOT MODIFY.
// source: internal/shared/internaltest/env.go.tmpl

// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package internaltest // import "go.opentelemetry.io/otel/internal/internaltest"

import (
	"os"
)

type Env struct {
	Name   string
	Value  string
	Exists bool
}

// EnvStore stores and recovers environment variables.
type EnvStore interface {
	// Records the environment variable into the store.
	Record(key string)

	// Restore recovers the environment variables in the store.
	Restore() error
}

var _ EnvStore = (*envStore)(nil)

type envStore struct {
	store map[string]Env
}

func (s *envStore) add(env Env) {
	s.store[env.Name] = env
}

func (s *envStore) Restore() error {
	var err error
	for _, v := range s.store {
		if v.Exists {
			err = os.Setenv(v.Name, v.Value)
		} else {
			err = os.Unsetenv(v.Name)
		}
		if err != nil {
			return err
		}
	}
	return nil
}

func (s *envStore) setEnv(key, value string) error {
	s.Record(key)

	err := os.Setenv(key, value)
	if err != nil {
		return err
	}
	return nil
}

func (s *envStore) Record(key string) {
	originValue, exists := os.LookupEnv(key)
	s.add(Env{
		Name:   key,
		Value:  originValue,
		Exists: exists,
	})
}

func NewEnvStore() EnvStore {
	return newEnvStore()
}

func newEnvStore() *envStore {
	return &envStore{store: make(map[string]Env)}
}

func SetEnvVariables(env map[string]string) (EnvStore, error) {
	envStore := newEnvStore()

	for k, v := range env {
		err := envStore.setEnv(k, v)
		if err != nil {
			return nil, err
		}
	}
	return envStore, nil
}
