package types_test

import (
	"bytes"
	"testing"
	"time"

	"github.com/stretchr/testify/suite"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
)

type utilsTestSuite struct {
	suite.Suite
}

func TestUtilsTestSuite(t *testing.T) {
	suite.Run(t, new(utilsTestSuite))
}

func (s *utilsTestSuite) SetupSuite() {
	s.T().Parallel()
}

func (s *utilsTestSuite) TestTimeFormatAndParse() {
	cases := []struct {
		RFC3339NanoStr     string
		SDKSortableTimeStr string
		Equal              bool
	}{
		{"2009-11-10T23:00:00Z", "2009-11-10T23:00:00.000000000", true},
		{"2011-01-10T23:10:05.758230235Z", "2011-01-10T23:10:05.758230235", true},
	}
	for _, tc := range cases {

		timeFromRFC, err := time.Parse(time.RFC3339Nano, tc.RFC3339NanoStr)
		s.Require().Nil(err)
		timeFromSDKFormat, err := time.Parse(sdk.SortableTimeFormat, tc.SDKSortableTimeStr)
		s.Require().Nil(err)

		s.Require().True(timeFromRFC.Equal(timeFromSDKFormat))
		s.Require().Equal(timeFromRFC.Format(sdk.SortableTimeFormat), tc.SDKSortableTimeStr)
	}
}

func (s *utilsTestSuite) TestCopyBytes() {
	s.Require().Nil(sdk.CopyBytes(nil))
	s.Require().Equal(0, len(sdk.CopyBytes([]byte{})))
	bs := []byte("test")
	bsCopy := sdk.CopyBytes(bs)
	s.Require().True(bytes.Equal(bs, bsCopy))
}

func (s *utilsTestSuite) TestUint64ToBigEndian() {
	s.Require().Equal([]byte{0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0}, sdk.Uint64ToBigEndian(uint64(0)))
	s.Require().Equal([]byte{0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xa}, sdk.Uint64ToBigEndian(uint64(10)))
}

func (s *utilsTestSuite) TestFormatTimeBytes() {
	tm, err := time.Parse("Jan 2, 2006 at 3:04pm (MST)", "Mar 3, 2020 at 7:54pm (UTC)")
	s.Require().NoError(err)
	s.Require().Equal("2020-03-03T19:54:00.000000000", string(sdk.FormatTimeBytes(tm)))
}

func (s *utilsTestSuite) TestFormatTimeString() {
	tm, err := time.Parse("Jan 2, 2006 at 3:04pm (MST)", "Mar 3, 2020 at 7:54pm (UTC)")
	s.Require().NoError(err)
	s.Require().Equal("2020-03-03T19:54:00.000000000", sdk.FormatTimeString(tm))
}

func (s *utilsTestSuite) TestParseTimeBytes() {
	tm, err := sdk.ParseTimeBytes([]byte("2020-03-03T19:54:00.000000000"))
	s.Require().NoError(err)
	s.Require().True(tm.Equal(time.Date(2020, 3, 3, 19, 54, 0, 0, time.UTC)))

	_, err = sdk.ParseTimeBytes([]byte{})
	s.Require().Error(err)
}

func (s *utilsTestSuite) TestParseTime() {
	testCases := []struct {
		name           string
		input          any
		expectErr      bool
		expectedOutput string
	}{
		{
			name:      "valid time string",
			input:     "2020-03-03T19:54:00.000000000",
			expectErr: false,
		},
		{
			name:      "valid time []byte",
			input:     []byte("2020-03-03T19:54:00.000000000"),
			expectErr: false,
		},
		{
			name:      "valid time",
			input:     time.Date(2020, 3, 3, 19, 54, 0, 0, time.UTC),
			expectErr: false,
		},
		{
			name: "valid time different timezone",
			input: func() time.Time {
				ams, _ := time.LoadLocation("Asia/Seoul") // no daylight saving time
				return time.Date(2020, 3, 4, 4, 54, 0, 0, ams)
			}(),
			expectErr: false,
		},
		{
			name:      "invalid time",
			input:     struct{}{},
			expectErr: true,
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			tm, err := sdk.ParseTime(tc.input)
			if tc.expectErr {
				s.Require().Error(err)
			} else {
				s.Require().NoError(err)
				s.Require().True(tm.Equal(time.Date(2020, 3, 3, 19, 54, 0, 0, time.UTC)))
			}
		})
	}
}

func (s *utilsTestSuite) TestAppendParseBytes() {
	test1 := "test1"
	test2 := "testString2"
	testByte1 := []byte(test1)
	testByte2 := []byte(test2)

	combinedBytes := sdk.AppendLengthPrefixedBytes(address.MustLengthPrefix(testByte1), address.MustLengthPrefix(testByte2))
	testCombineBytes := append([]byte{}, address.MustLengthPrefix(testByte1)...)
	testCombineBytes = append(testCombineBytes, address.MustLengthPrefix(testByte2)...)
	s.Require().Equal(combinedBytes, testCombineBytes)

	test1Len, test1LenEndIndex := sdk.ParseLengthPrefixedBytes(combinedBytes, 0, 1)
	parseTest1, parseTest1EndIndex := sdk.ParseLengthPrefixedBytes(combinedBytes, test1LenEndIndex+1, int(test1Len[0]))
	s.Require().Equal(testByte1, parseTest1)

	test2Len, test2LenEndIndex := sdk.ParseLengthPrefixedBytes(combinedBytes, parseTest1EndIndex+1, 1)
	parseTest2, _ := sdk.ParseLengthPrefixedBytes(combinedBytes, test2LenEndIndex+1, int(test2Len[0]))
	s.Require().Equal(testByte2, parseTest2)
}
