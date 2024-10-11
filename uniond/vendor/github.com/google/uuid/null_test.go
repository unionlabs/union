// Copyright 2021 Google Inc.  All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package uuid

import (
	"bytes"
	"encoding/json"
	"testing"
)

func TestNullUUIDScan(t *testing.T) {
	var u UUID
	var nu NullUUID

	uNilErr := u.Scan(nil)
	nuNilErr := nu.Scan(nil)
	if uNilErr != nil &&
		nuNilErr != nil &&
		uNilErr.Error() != nuNilErr.Error() {
		t.Errorf("expected errors to be equal, got %s, %s", uNilErr, nuNilErr)
	}

	uInvalidStringErr := u.Scan("test")
	nuInvalidStringErr := nu.Scan("test")
	if uInvalidStringErr != nil &&
		nuInvalidStringErr != nil &&
		uInvalidStringErr.Error() != nuInvalidStringErr.Error() {
		t.Errorf("expected errors to be equal, got %s, %s", uInvalidStringErr, nuInvalidStringErr)
	}

	valid := "12345678-abcd-1234-abcd-0123456789ab"
	uValidErr := u.Scan(valid)
	nuValidErr := nu.Scan(valid)
	if uValidErr != nuValidErr {
		t.Errorf("expected errors to be equal, got %s, %s", uValidErr, nuValidErr)
	}
}

func TestNullUUIDValue(t *testing.T) {
	var u UUID
	var nu NullUUID

	nuValue, nuErr := nu.Value()
	if nuErr != nil {
		t.Errorf("expected nil err, got err %s", nuErr)
	}
	if nuValue != nil {
		t.Errorf("expected nil value, got non-nil %s", nuValue)
	}

	u = MustParse("12345678-abcd-1234-abcd-0123456789ab")
	nu = NullUUID{
		UUID:  MustParse("12345678-abcd-1234-abcd-0123456789ab"),
		Valid: true,
	}

	uValue, uErr := u.Value()
	nuValue, nuErr = nu.Value()
	if uErr != nil {
		t.Errorf("expected nil err, got err %s", uErr)
	}
	if nuErr != nil {
		t.Errorf("expected nil err, got err %s", nuErr)
	}
	if uValue != nuValue {
		t.Errorf("expected uuid %s and nulluuid %s to be equal ", uValue, nuValue)
	}
}

func TestNullUUIDMarshalText(t *testing.T) {
	tests := []struct {
		nullUUID NullUUID
	}{
		{
			nullUUID: NullUUID{},
		},
		{
			nullUUID: NullUUID{
				UUID:  MustParse("12345678-abcd-1234-abcd-0123456789ab"),
				Valid: true,
			},
		},
	}
	for _, test := range tests {
		var uText []byte
		var uErr error
		nuText, nuErr := test.nullUUID.MarshalText()
		if test.nullUUID.Valid {
			uText, uErr = test.nullUUID.UUID.MarshalText()
		} else {
			uText = []byte("null")
		}
		if nuErr != uErr {
			t.Errorf("expected error %e, got %e", nuErr, uErr)
		}
		if !bytes.Equal(nuText, uText) {
			t.Errorf("expected text data %s, got %s", string(nuText), string(uText))
		}
	}
}

func TestNullUUIDUnmarshalText(t *testing.T) {
	tests := []struct {
		nullUUID NullUUID
	}{
		{
			nullUUID: NullUUID{},
		},
		{
			nullUUID: NullUUID{
				UUID:  MustParse("12345678-abcd-1234-abcd-0123456789ab"),
				Valid: true,
			},
		},
	}
	for _, test := range tests {
		var uText []byte
		var uErr error
		nuText, nuErr := test.nullUUID.MarshalText()
		if test.nullUUID.Valid {
			uText, uErr = test.nullUUID.UUID.MarshalText()
		} else {
			uText = []byte("null")
		}
		if nuErr != uErr {
			t.Errorf("expected error %e, got %e", nuErr, uErr)
		}
		if !bytes.Equal(nuText, uText) {
			t.Errorf("expected text data %s, got %s", string(nuText), string(uText))
		}
	}
}

func TestNullUUIDMarshalBinary(t *testing.T) {
	tests := []struct {
		nullUUID NullUUID
	}{
		{
			nullUUID: NullUUID{},
		},
		{
			nullUUID: NullUUID{
				UUID:  MustParse("12345678-abcd-1234-abcd-0123456789ab"),
				Valid: true,
			},
		},
	}
	for _, test := range tests {
		var uBinary []byte
		var uErr error
		nuBinary, nuErr := test.nullUUID.MarshalBinary()
		if test.nullUUID.Valid {
			uBinary, uErr = test.nullUUID.UUID.MarshalBinary()
		} else {
			uBinary = []byte(nil)
		}
		if nuErr != uErr {
			t.Errorf("expected error %e, got %e", nuErr, uErr)
		}
		if !bytes.Equal(nuBinary, uBinary) {
			t.Errorf("expected binary data %s, got %s", string(nuBinary), string(uBinary))
		}
	}
}

func TestNullUUIDMarshalJSON(t *testing.T) {
	jsonNull, _ := json.Marshal(nil)
	jsonUUID, _ := json.Marshal(MustParse("12345678-abcd-1234-abcd-0123456789ab"))
	tests := []struct {
		nullUUID    NullUUID
		expected    []byte
		expectedErr error
	}{
		{
			nullUUID:    NullUUID{},
			expected:    jsonNull,
			expectedErr: nil,
		},
		{
			nullUUID: NullUUID{
				UUID:  MustParse(string(jsonUUID)),
				Valid: true,
			},
			expected:    []byte(`"12345678-abcd-1234-abcd-0123456789ab"`),
			expectedErr: nil,
		},
	}
	for _, test := range tests {
		data, err := json.Marshal(&test.nullUUID)
		if err != test.expectedErr {
			t.Errorf("expected error %e, got %e", test.expectedErr, err)
		}
		if !bytes.Equal(data, test.expected) {
			t.Errorf("expected json data %s, got %s", string(test.expected), string(data))
		}
	}
}

func TestNullUUIDUnmarshalJSON(t *testing.T) {
	jsonNull, _ := json.Marshal(nil)
	jsonUUID, _ := json.Marshal(MustParse("12345678-abcd-1234-abcd-0123456789ab"))

	var nu NullUUID
	err := json.Unmarshal(jsonNull, &nu)
	if err != nil || nu.Valid {
		t.Errorf("expected nil when unmarshaling null, got %s", err)
	}
	err = json.Unmarshal(jsonUUID, &nu)
	if err != nil || !nu.Valid {
		t.Errorf("expected nil when unmarshaling null, got %s", err)
	}
}
