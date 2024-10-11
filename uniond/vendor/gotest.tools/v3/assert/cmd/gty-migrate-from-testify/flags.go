package main

import (
	"encoding/csv"
	"strings"
)

type stringSliceValue []string

func (s stringSliceValue) String() string {
	return strings.Join(s, ", ")
}

func (s *stringSliceValue) Set(raw string) error {
	if raw == "" {
		return nil
	}
	v, err := csv.NewReader(strings.NewReader(raw)).Read()
	if err != nil {
		return err
	}
	*s = append(*s, v...)
	return nil
}
