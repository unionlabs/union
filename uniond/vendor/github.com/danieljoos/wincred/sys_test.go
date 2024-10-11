//go:build windows
// +build windows

package wincred

import (
	"errors"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

type mockProc struct {
	mock.Mock
	orig   proc
	target *proc
}

func (t *mockProc) Setup(target *proc) {
	t.target = target
	t.orig = *t.target
	*(t.target) = t
}

func (t *mockProc) TearDown() {
	*(t.target) = t.orig
}

func (t *mockProc) Call(a ...uintptr) (r1, r2 uintptr, lastErr error) {
	args := t.Called(a)
	return uintptr(args.Int(0)), uintptr(args.Int(1)), args.Error(2)
}

func TestSysCredWrite_MockFailure(t *testing.T) {
	// Mock `CreadWrite`: returns failure state and the error
	mockCredWrite := new(mockProc)
	mockCredWrite.On("Call", mock.AnythingOfType("[]uintptr")).Return(0, 0, errors.New("test error"))
	mockCredWrite.Setup(&procCredWrite)
	defer mockCredWrite.TearDown()

	// Test it:
	var err error
	assert.NotPanics(t, func() { err = sysCredWrite(new(Credential), sysCRED_TYPE_GENERIC) })
	assert.NotNil(t, err)
	assert.Equal(t, "test error", err.Error())
	mockCredWrite.AssertNumberOfCalls(t, "Call", 1)
}

func TestSysCredWrite_Mock(t *testing.T) {
	// Mock `CreadWrite`: returns success state
	mockCredWrite := new(mockProc)
	mockCredWrite.On("Call", mock.AnythingOfType("[]uintptr")).Return(1, 0, nil)
	mockCredWrite.Setup(&procCredWrite)
	defer mockCredWrite.TearDown()

	// Test it:
	var err error
	assert.NotPanics(t, func() { err = sysCredWrite(new(Credential), sysCRED_TYPE_GENERIC) })
	assert.Nil(t, err)
	mockCredWrite.AssertNumberOfCalls(t, "Call", 1)
}

func TestSysCredDelete_MockFailure(t *testing.T) {
	// Mock `CreadDelete`: returns failure state and an error
	mockCredDelete := new(mockProc)
	mockCredDelete.On("Call", mock.AnythingOfType("[]uintptr")).Return(0, 0, errors.New("test error"))
	mockCredDelete.Setup(&procCredDelete)
	defer mockCredDelete.TearDown()

	// Test it:
	var err error
	assert.NotPanics(t, func() { err = sysCredDelete(new(Credential), sysCRED_TYPE_GENERIC) })
	assert.NotNil(t, err)
	assert.Equal(t, "test error", err.Error())
	mockCredDelete.AssertNumberOfCalls(t, "Call", 1)
}

func TestSysCredDelete_Mock(t *testing.T) {
	// Mock `CreadDelete`: returns success state
	mockCredDelete := new(mockProc)
	mockCredDelete.On("Call", mock.AnythingOfType("[]uintptr")).Return(1, 0, nil)
	mockCredDelete.Setup(&procCredDelete)
	defer mockCredDelete.TearDown()

	// Test it:
	var err error
	assert.NotPanics(t, func() { err = sysCredDelete(new(Credential), sysCRED_TYPE_GENERIC) })
	assert.Nil(t, err)
	mockCredDelete.AssertNumberOfCalls(t, "Call", 1)
}
