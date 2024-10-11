// Code created by gotmpl. DO NOT MODIFY.
// source: internal/shared/internaltest/env_test.go.tmpl

// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package internaltest

import (
	"os"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"github.com/stretchr/testify/suite"
)

type EnvStoreTestSuite struct {
	suite.Suite
}

func (s *EnvStoreTestSuite) Test_add() {
	envStore := newEnvStore()

	e := Env{
		Name:   "name",
		Value:  "value",
		Exists: true,
	}
	envStore.add(e)
	envStore.add(e)

	s.Assert().Len(envStore.store, 1)
}

func (s *EnvStoreTestSuite) TestRecord() {
	testCases := []struct {
		name             string
		env              Env
		expectedEnvStore *envStore
	}{
		{
			name: "record exists env",
			env: Env{
				Name:   "name",
				Value:  "value",
				Exists: true,
			},
			expectedEnvStore: &envStore{store: map[string]Env{
				"name": {
					Name:   "name",
					Value:  "value",
					Exists: true,
				},
			}},
		},
		{
			name: "record exists env, but its value is empty",
			env: Env{
				Name:   "name",
				Value:  "",
				Exists: true,
			},
			expectedEnvStore: &envStore{store: map[string]Env{
				"name": {
					Name:   "name",
					Value:  "",
					Exists: true,
				},
			}},
		},
		{
			name: "record not exists env",
			env: Env{
				Name:   "name",
				Exists: false,
			},
			expectedEnvStore: &envStore{store: map[string]Env{
				"name": {
					Name:   "name",
					Exists: false,
				},
			}},
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			if tc.env.Exists {
				s.Assert().NoError(os.Setenv(tc.env.Name, tc.env.Value))
			}

			envStore := newEnvStore()
			envStore.Record(tc.env.Name)

			s.Assert().Equal(tc.expectedEnvStore, envStore)

			if tc.env.Exists {
				s.Assert().NoError(os.Unsetenv(tc.env.Name))
			}
		})
	}
}

func (s *EnvStoreTestSuite) TestRestore() {
	testCases := []struct {
		name              string
		env               Env
		expectedEnvValue  string
		expectedEnvExists bool
	}{
		{
			name: "exists env",
			env: Env{
				Name:   "name",
				Value:  "value",
				Exists: true,
			},
			expectedEnvValue:  "value",
			expectedEnvExists: true,
		},
		{
			name: "no exists env",
			env: Env{
				Name:   "name",
				Exists: false,
			},
			expectedEnvExists: false,
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			envStore := newEnvStore()
			envStore.add(tc.env)

			// Backup
			backup := newEnvStore()
			backup.Record(tc.env.Name)

			s.Require().NoError(os.Unsetenv(tc.env.Name))

			s.Assert().NoError(envStore.Restore())
			v, exists := os.LookupEnv(tc.env.Name)
			s.Assert().Equal(tc.expectedEnvValue, v)
			s.Assert().Equal(tc.expectedEnvExists, exists)

			// Restore
			s.Require().NoError(backup.Restore())
		})
	}
}

func (s *EnvStoreTestSuite) Test_setEnv() {
	testCases := []struct {
		name              string
		key               string
		value             string
		expectedEnvStore  *envStore
		expectedEnvValue  string
		expectedEnvExists bool
	}{
		{
			name:  "normal",
			key:   "name",
			value: "value",
			expectedEnvStore: &envStore{store: map[string]Env{
				"name": {
					Name:   "name",
					Value:  "other value",
					Exists: true,
				},
			}},
			expectedEnvValue:  "value",
			expectedEnvExists: true,
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			envStore := newEnvStore()

			// Backup
			backup := newEnvStore()
			backup.Record(tc.key)

			s.Require().NoError(os.Setenv(tc.key, "other value"))

			s.Assert().NoError(envStore.setEnv(tc.key, tc.value))
			s.Assert().Equal(tc.expectedEnvStore, envStore)
			v, exists := os.LookupEnv(tc.key)
			s.Assert().Equal(tc.expectedEnvValue, v)
			s.Assert().Equal(tc.expectedEnvExists, exists)

			// Restore
			s.Require().NoError(backup.Restore())
		})
	}
}

func TestEnvStoreTestSuite(t *testing.T) {
	suite.Run(t, new(EnvStoreTestSuite))
}

func TestSetEnvVariables(t *testing.T) {
	envs := map[string]string{
		"name1": "value1",
		"name2": "value2",
	}

	// Backup
	backup := newEnvStore()
	for k := range envs {
		backup.Record(k)
	}
	defer func() {
		require.NoError(t, backup.Restore())
	}()

	store, err := SetEnvVariables(envs)
	assert.NoError(t, err)
	require.IsType(t, &envStore{}, store)
	concreteStore := store.(*envStore)
	assert.Len(t, concreteStore.store, 2)
	assert.Equal(t, backup, concreteStore)
}
