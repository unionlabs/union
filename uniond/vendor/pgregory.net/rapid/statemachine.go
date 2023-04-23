// Copyright 2019 Gregory Petrosyan <gregory.petrosyan@gmail.com>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package rapid

import (
	"math"
	"reflect"
	"sort"
	"testing"
)

const (
	actionLabel      = "action"
	validActionTries = 100 // hack, but probably good enough for now

	initMethodName    = "Init"
	checkMethodName   = "Check"
	cleanupMethodName = "Cleanup"

	noValidActionsMsg = "can't find a valid action"
)

type StateMachine interface {
	// Check is ran after every action and should contain invariant checks.
	//
	// Other public methods are treated as follows:
	// - Init(t *rapid.T), if present, is ran at the beginning of each test case
	//   to initialize the state machine instance;
	// - Cleanup(), if present, is called at the end of each test case;
	// - All other public methods should have a form ActionName(t *rapid.T)
	//   and are used as possible actions. At least one action has to be specified.
	//
	Check(*T)
}

// Run is a convenience function for defining "state machine" tests,
// to be run by [Check] or [MakeCheck].
//
// State machine test is a pattern for testing stateful systems that looks
// like this:
//
//	m := new(StateMachineType)
//	m.Init(t)          // optional
//	defer m.Cleanup()  // optional
//	m.Check(t)
//	for {
//	    m.RandomAction(t)
//	    m.Check(t)
//	}
//
// Run synthesizes such test from the M type, which must be a pointer,
// using reflection.
func Run[M StateMachine]() func(*T) {
	var m M
	typ := reflect.TypeOf(m)

	steps := flags.steps
	if testing.Short() {
		steps /= 5
	}

	return func(t *T) {
		t.Helper()

		repeat := newRepeat(0, steps, math.MaxInt, typ.String())

		sm := newStateMachine(typ)
		if sm.init != nil {
			sm.init(t)
			t.failOnError()
		}
		if sm.cleanup != nil {
			defer sm.cleanup()
		}

		sm.check(t)
		t.failOnError()
		for repeat.more(t.s) {
			ok := sm.executeAction(t)
			if ok {
				sm.check(t)
				t.failOnError()
			} else {
				repeat.reject()
			}
		}
	}
}

type stateMachine struct {
	init       func(*T)
	cleanup    func()
	check      func(*T)
	actionKeys *Generator[string]
	actions    map[string]func(*T)
}

func newStateMachine(typ reflect.Type) *stateMachine {
	assertf(typ.Kind() == reflect.Ptr, "state machine type should be a pointer, not %v", typ.Kind())

	var (
		v          = reflect.New(typ.Elem())
		n          = typ.NumMethod()
		init       func(*T)
		cleanup    func()
		actionKeys []string
		actions    = map[string]func(*T){}
	)

	for i := 0; i < n; i++ {
		name := typ.Method(i).Name
		m, ok := v.Method(i).Interface().(func(*T))
		if ok {
			if name == initMethodName {
				init = m
			} else if name != checkMethodName {
				actionKeys = append(actionKeys, name)
				actions[name] = m
			}
		} else if name == cleanupMethodName {
			m, ok := v.Method(i).Interface().(func())
			assertf(ok, "method %v should have type func(), not %v", cleanupMethodName, v.Method(i).Type())
			cleanup = m
		}
	}

	assertf(len(actions) > 0, "state machine of type %v has no actions specified", typ)
	sort.Strings(actionKeys)

	return &stateMachine{
		init:       init,
		cleanup:    cleanup,
		check:      v.Interface().(StateMachine).Check,
		actionKeys: SampledFrom(actionKeys),
		actions:    actions,
	}
}

func (sm *stateMachine) executeAction(t *T) bool {
	t.Helper()

	for n := 0; n < validActionTries; n++ {
		i := t.s.beginGroup(actionLabel, false)
		action := sm.actions[sm.actionKeys.Draw(t, "action")]
		invalid, skipped := runAction(t, action)
		t.s.endGroup(i, false)

		if skipped {
			continue
		} else {
			return !invalid
		}
	}

	panic(stopTest(noValidActionsMsg))
}

func runAction(t *T, action func(*T)) (invalid bool, skipped bool) {
	defer func(draws int) {
		if r := recover(); r != nil {
			if _, ok := r.(invalidData); ok {
				invalid = true
				skipped = t.draws == draws
			} else {
				panic(r)
			}
		}
	}(t.draws)

	action(t)
	t.failOnError()

	return false, false
}
