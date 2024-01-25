// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray

// import ()

// MarshalBinary implements the standard encoding.BinaryMarshaler interface to
// marshal a BitArray into a binary form.
//
// This appends 0 to 7 bits padding followed by 3 bits marker indicating the
// actual number of padding LSBs at the end of the original bit array.
//
//     | nBits&7 | last two bytes    mark | pad |
//     +---------+-----------+------------+-----+
//     |       5 | 1111 1111 | 1111 1:000 |   3 |
//     |       4 | 1111 1111 | 1111 -:001 |   4 |
//     |       3 | 1111 1111 | 111- -:010 |   5 |
//     |       2 | 1111 1111 | 11-- -:011 |   6 |
//     |       1 | 1111 1111 | 1--- -:100 |   7 |
//     |       0 | 1111 1111 | ---- -:101 |   8 |
//     |       7 | 1111 111- | ---- -:110 |   9 |
//     |       6 | 1111 11-- | ---- -:111 |  10 |
func (ba *BitArray) MarshalBinary() ([]byte, error) {
	if ba.IsZero() {
		return []byte{}, nil
	}
	nBytes := (ba.nBits + 7) >> 3
	nfrac := ba.nBits & 7
	marker := byte((13 - nfrac) & 7)
	if nfrac == 0 || nfrac>>1 == 3 {
		buf := make([]byte, nBytes+1)
		if ba.b != nil {
			copy(buf, ba.b)
		}
		buf[nBytes] = marker
		return buf, nil
	}
	buf := make([]byte, nBytes)
	if ba.b != nil {
		copy(buf, ba.b)
	}
	buf[nBytes-1] |= marker

	return buf, nil
}

// MarshalText implements the standard encoding.TextMarshaler interface to
// marshal a BitArray into a textual form. This always returns a nil error.
func (ba *BitArray) MarshalText() ([]byte, error) {
	if ba.IsZero() {
		return []byte{}, nil
	}
	sb := make([]byte, ba.nBits)
	if ba.b == nil {
		fill30(sb)
	} else {
		for i := 0; i < ba.nBits; i++ {
			sb[i] = '0' + ba.b[i>>3]>>(7-i&7)&1
		}
	}

	return sb, nil
}

// MarshalJSON implements the standard json.Marshaler interface to marshal a
// BitArray into a JSON form. This always returns a nil error.
func (ba *BitArray) MarshalJSON() ([]byte, error) {
	if ba.IsZero() {
		return []byte(`""`), nil
	}
	sb := make([]byte, ba.nBits+2)
	if ba.b == nil {
		fill30(sb)
	} else {
		for i := 0; i < ba.nBits; i++ {
			sb[i+1] = '0' + ba.b[i>>3]>>(7-i&7)&1
		}
	}
	sb[0] = '"'
	sb[ba.nBits+1] = '"'

	return sb, nil
}

// MarshalYAML implements the yaml.Marshaler.
func (ba *BitArray) MarshalYAML() (interface{}, error) {
	if ba == nil {
		return nil, nil
	}
	return ba.String(), nil
}
