// Copyright 2010 Google Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package gomock_test

//go:generate mockgen -destination internal/mock_gomock/mock_matcher.go github.com/golang/mock/gomock Matcher

import (
	"context"
	"errors"
	"reflect"
	"testing"

	"github.com/golang/mock/gomock"
	"github.com/golang/mock/gomock/internal/mock_gomock"
)

type A []string

func TestMatchers(t *testing.T) {
	type e interface{}
	tests := []struct {
		name    string
		matcher gomock.Matcher
		yes, no []e
	}{
		{"test Any", gomock.Any(), []e{3, nil, "foo"}, nil},
		{"test All", gomock.Eq(4), []e{4}, []e{3, "blah", nil, int64(4)}},
		{"test Nil", gomock.Nil(),
			[]e{nil, (error)(nil), (chan bool)(nil), (*int)(nil)},
			[]e{"", 0, make(chan bool), errors.New("err"), new(int)}},
		{"test Not", gomock.Not(gomock.Eq(4)), []e{3, "blah", nil, int64(4)}, []e{4}},
		{"test All", gomock.All(gomock.Any(), gomock.Eq(4)), []e{4}, []e{3, "blah", nil, int64(4)}},
		{"test Len", gomock.Len(2),
			[]e{[]int{1, 2}, "ab", map[string]int{"a": 0, "b": 1}, [2]string{"a", "b"}},
			[]e{[]int{1}, "a", 42, 42.0, false, [1]string{"a"}},
		},
		{"test assignable types", gomock.Eq(A{"a", "b"}),
			[]e{[]string{"a", "b"}, A{"a", "b"}},
			[]e{[]string{"a"}, A{"b"}},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			for _, x := range tt.yes {
				if !tt.matcher.Matches(x) {
					t.Errorf(`"%v %s": got false, want true.`, x, tt.matcher)
				}
			}
			for _, x := range tt.no {
				if tt.matcher.Matches(x) {
					t.Errorf(`"%v %s": got true, want false.`, x, tt.matcher)
				}
			}
		})
	}
}

// A more thorough test of notMatcher
func TestNotMatcher(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	mockMatcher := mock_gomock.NewMockMatcher(ctrl)
	notMatcher := gomock.Not(mockMatcher)

	mockMatcher.EXPECT().Matches(4).Return(true)
	if match := notMatcher.Matches(4); match {
		t.Errorf("notMatcher should not match 4")
	}

	mockMatcher.EXPECT().Matches(5).Return(false)
	if match := notMatcher.Matches(5); !match {
		t.Errorf("notMatcher should match 5")
	}
}

type Dog struct {
	Breed, Name string
}

type ctxKey struct{}

// A thorough test of assignableToTypeOfMatcher
func TestAssignableToTypeOfMatcher(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	aStr := "def"
	anotherStr := "ghi"

	if match := gomock.AssignableToTypeOf("abc").Matches(4); match {
		t.Errorf(`AssignableToTypeOf("abc") should not match 4`)
	}
	if match := gomock.AssignableToTypeOf("abc").Matches(&aStr); match {
		t.Errorf(`AssignableToTypeOf("abc") should not match &aStr (*string)`)
	}
	if match := gomock.AssignableToTypeOf("abc").Matches("def"); !match {
		t.Errorf(`AssignableToTypeOf("abc") should match "def"`)
	}
	if match := gomock.AssignableToTypeOf(&aStr).Matches("abc"); match {
		t.Errorf(`AssignableToTypeOf(&aStr) should not match "abc"`)
	}
	if match := gomock.AssignableToTypeOf(&aStr).Matches(&anotherStr); !match {
		t.Errorf(`AssignableToTypeOf(&aStr) should match &anotherStr`)
	}
	if match := gomock.AssignableToTypeOf(0).Matches(4); !match {
		t.Errorf(`AssignableToTypeOf(0) should match 4`)
	}
	if match := gomock.AssignableToTypeOf(0).Matches("def"); match {
		t.Errorf(`AssignableToTypeOf(0) should not match "def"`)
	}
	if match := gomock.AssignableToTypeOf(Dog{}).Matches(&Dog{}); match {
		t.Errorf(`AssignableToTypeOf(Dog{}) should not match &Dog{}`)
	}
	if match := gomock.AssignableToTypeOf(Dog{}).Matches(Dog{Breed: "pug", Name: "Fido"}); !match {
		t.Errorf(`AssignableToTypeOf(Dog{}) should match Dog{Breed: "pug", Name: "Fido"}`)
	}
	if match := gomock.AssignableToTypeOf(&Dog{}).Matches(Dog{}); match {
		t.Errorf(`AssignableToTypeOf(&Dog{}) should not match Dog{}`)
	}
	if match := gomock.AssignableToTypeOf(&Dog{}).Matches(&Dog{Breed: "pug", Name: "Fido"}); !match {
		t.Errorf(`AssignableToTypeOf(&Dog{}) should match &Dog{Breed: "pug", Name: "Fido"}`)
	}

	ctxInterface := reflect.TypeOf((*context.Context)(nil)).Elem()
	if match := gomock.AssignableToTypeOf(ctxInterface).Matches(context.Background()); !match {
		t.Errorf(`AssignableToTypeOf(context.Context) should not match context.Background()`)
	}

	ctxWithValue := context.WithValue(context.Background(), ctxKey{}, "val")
	if match := gomock.AssignableToTypeOf(ctxInterface).Matches(ctxWithValue); !match {
		t.Errorf(`AssignableToTypeOf(context.Context) should not match ctxWithValue`)
	}
}

func TestInAnyOrder(t *testing.T) {
	tests := []struct {
		name      string
		wanted    interface{}
		given     interface{}
		wantMatch bool
	}{
		{
			name:      "match for equal slices",
			wanted:    []int{1, 2, 3},
			given:     []int{1, 2, 3},
			wantMatch: true,
		},
		{
			name:      "match for slices with same elements of different order",
			wanted:    []int{1, 2, 3},
			given:     []int{1, 3, 2},
			wantMatch: true,
		},
		{
			name:      "not match for slices with different elements",
			wanted:    []int{1, 2, 3},
			given:     []int{1, 2, 4},
			wantMatch: false,
		},
		{
			name:      "not match for slices with missing elements",
			wanted:    []int{1, 2, 3},
			given:     []int{1, 2},
			wantMatch: false,
		},
		{
			name:      "not match for slices with extra elements",
			wanted:    []int{1, 2, 3},
			given:     []int{1, 2, 3, 4},
			wantMatch: false,
		},
		{
			name:      "match for empty slices",
			wanted:    []int{},
			given:     []int{},
			wantMatch: true,
		},
		{
			name:      "not match for equal slices of different types",
			wanted:    []float64{1, 2, 3},
			given:     []int{1, 2, 3},
			wantMatch: false,
		},
		{
			name:      "match for equal arrays",
			wanted:    [3]int{1, 2, 3},
			given:     [3]int{1, 2, 3},
			wantMatch: true,
		},
		{
			name:      "match for equal arrays of different order",
			wanted:    [3]int{1, 2, 3},
			given:     [3]int{1, 3, 2},
			wantMatch: true,
		},
		{
			name:      "not match for arrays of different elements",
			wanted:    [3]int{1, 2, 3},
			given:     [3]int{1, 2, 4},
			wantMatch: false,
		},
		{
			name:      "not match for arrays with extra elements",
			wanted:    [3]int{1, 2, 3},
			given:     [4]int{1, 2, 3, 4},
			wantMatch: false,
		},
		{
			name:      "not match for arrays with missing elements",
			wanted:    [3]int{1, 2, 3},
			given:     [2]int{1, 2},
			wantMatch: false,
		},
		{
			name:      "not match for equal strings", // matcher shouldn't treat strings as collections
			wanted:    "123",
			given:     "123",
			wantMatch: false,
		},
		{
			name:      "not match if x type is not iterable",
			wanted:    123,
			given:     []int{123},
			wantMatch: false,
		},
		{
			name:      "not match if in type is not iterable",
			wanted:    []int{123},
			given:     123,
			wantMatch: false,
		},
		{
			name:      "not match if both are not iterable",
			wanted:    123,
			given:     123,
			wantMatch: false,
		},
		{
			name:      "match for equal slices with unhashable elements",
			wanted:    [][]int{{1}, {1, 2}, {1, 2, 3}},
			given:     [][]int{{1}, {1, 2}, {1, 2, 3}},
			wantMatch: true,
		},
		{
			name:      "match for equal slices with unhashable elements of different order",
			wanted:    [][]int{{1}, {1, 2, 3}, {1, 2}},
			given:     [][]int{{1}, {1, 2}, {1, 2, 3}},
			wantMatch: true,
		},
		{
			name:      "not match for different slices with unhashable elements",
			wanted:    [][]int{{1}, {1, 2, 3}, {1, 2}},
			given:     [][]int{{1}, {1, 2, 4}, {1, 3}},
			wantMatch: false,
		},
		{
			name:      "not match for unhashable missing elements",
			wanted:    [][]int{{1}, {1, 2}, {1, 2, 3}},
			given:     [][]int{{1}, {1, 2}},
			wantMatch: false,
		},
		{
			name:      "not match for unhashable extra elements",
			wanted:    [][]int{{1}, {1, 2}},
			given:     [][]int{{1}, {1, 2}, {1, 2, 3}},
			wantMatch: false,
		},
		{
			name:      "match for equal slices of assignable types",
			wanted:    [][]string{{"a", "b"}},
			given:     []A{{"a", "b"}},
			wantMatch: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := gomock.InAnyOrder(tt.wanted).Matches(tt.given); got != tt.wantMatch {
				t.Errorf("got = %v, wantMatch %v", got, tt.wantMatch)
			}
		})
	}
}
