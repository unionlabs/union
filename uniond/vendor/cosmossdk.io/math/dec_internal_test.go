package math

import (
	"encoding/json"
	"math/big"
	"testing"

	"github.com/stretchr/testify/suite"
)

type decimalInternalTestSuite struct {
	suite.Suite
}

func TestDecimalInternalTestSuite(t *testing.T) {
	suite.Run(t, new(decimalInternalTestSuite))
}

func (s *decimalInternalTestSuite) TestPrecisionMultiplier() {
	tests := []struct {
		prec int64
		exp  *big.Int
	}{
		{
			5,
			big.NewInt(10000000000000),
		},
		{
			8,
			big.NewInt(10000000000),
		},
		{
			11,
			big.NewInt(10000000),
		},
		{
			15,
			big.NewInt(1000),
		},
		{
			18,
			big.NewInt(1),
		},
	}
	for _, tt := range tests {
		res := precisionMultiplier(tt.prec)
		s.Require().Equal(0, res.Cmp(tt.exp), "equality was incorrect, res %v, exp %v", res, tt.exp)
	}
}

func (s *decimalInternalTestSuite) TestZeroDeserializationJSON() {
	d := LegacyDec{new(big.Int)}
	err := json.Unmarshal([]byte(`"0"`), &d)
	s.Require().Nil(err)
	err = json.Unmarshal([]byte(`"{}"`), &d)
	s.Require().NotNil(err)
}

func (s *decimalInternalTestSuite) TestSerializationGocodecJSON() {
	d := LegacyMustNewDecFromStr("0.333")

	bz, err := json.Marshal(d)
	s.Require().NoError(err)

	d2 := LegacyDec{new(big.Int)}
	err = json.Unmarshal(bz, &d2)
	s.Require().NoError(err)
	s.Require().True(d.Equal(d2), "original: %v, unmarshalled: %v", d, d2)
}

func (s *decimalInternalTestSuite) TestDecMarshalJSON() {
	decimal := func(i int64) LegacyDec {
		d := LegacyNewDec(0)
		d.i = new(big.Int).SetInt64(i)
		return d
	}
	tests := []struct {
		name    string
		d       LegacyDec
		want    string
		wantErr bool // if wantErr = false, will also attempt unmarshaling
	}{
		{"zero", decimal(0), "\"0.000000000000000000\"", false},
		{"one", decimal(1), "\"0.000000000000000001\"", false},
		{"ten", decimal(10), "\"0.000000000000000010\"", false},
		{"12340", decimal(12340), "\"0.000000000000012340\"", false},
		{"zeroInt", LegacyNewDec(0), "\"0.000000000000000000\"", false},
		{"oneInt", LegacyNewDec(1), "\"1.000000000000000000\"", false},
		{"tenInt", LegacyNewDec(10), "\"10.000000000000000000\"", false},
		{"12340Int", LegacyNewDec(12340), "\"12340.000000000000000000\"", false},
	}
	for _, tt := range tests {
		tt := tt
		s.T().Run(tt.name, func(t *testing.T) {
			got, err := tt.d.MarshalJSON()
			if (err != nil) != tt.wantErr {
				t.Errorf("Dec.MarshalJSON() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !tt.wantErr {
				s.Require().Equal(tt.want, string(got), "incorrect marshaled value")
				unmarshalledDec := LegacyNewDec(0)
				err := unmarshalledDec.UnmarshalJSON(got)
				s.Require().NoError(err)
				s.Require().Equal(tt.d, unmarshalledDec, "incorrect unmarshalled value")
			}
		})
	}
}
