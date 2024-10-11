package amino

import (
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

var cdc = NewCodec()

func init() {
	cdc.Seal()
}

type testTime struct {
	Time time.Time
}

func TestDecodeSkippedFieldsInTime(t *testing.T) {
	tm, err := time.Parse("2006-01-02 15:04:05 +0000 UTC", "1970-01-01 00:00:00 +0000 UTC")
	assert.NoError(t, err)

	b, err := cdc.MarshalBinaryLengthPrefixed(testTime{Time: tm})
	assert.NoError(t, err)
	var ti testTime
	err = cdc.UnmarshalBinaryLengthPrefixed(b, &ti)
	assert.NoError(t, err)
	assert.Equal(t, testTime{Time: tm}, ti)

	tm2, err := time.Parse("2006-01-02 15:04:05 +0000 UTC", "1970-01-01 00:00:01.978131102 +0000 UTC")
	assert.NoError(t, err)

	b, err = cdc.MarshalBinaryLengthPrefixed(testTime{Time: tm2})
	assert.NoError(t, err)
	err = cdc.UnmarshalBinaryLengthPrefixed(b, &ti)
	assert.NoError(t, err)
	assert.Equal(t, testTime{Time: tm2}, ti)

	t1, _ := time.Parse("2006-01-02 15:04:05 +0000 UTC", "1970-01-01 00:00:11.577968799 +0000 UTC")
	t2, _ := time.Parse("2006-01-02 15:04:05 +0000 UTC", "2078-07-10 15:44:58.406865636 +0000 UTC")
	t3, _ := time.Parse("2006-01-02 15:04:05 +0000 UTC", "1970-01-01 00:00:00 +0000 UTC")
	t4, _ := time.Parse("2006-01-02 15:04:05 +0000 UTC", "1970-01-01 00:00:14.48251984 +0000 UTC")

	type tArr struct {
		TimeAr [4]time.Time
	}
	st := tArr{
		TimeAr: [4]time.Time{t1, t2, t3, t4},
	}
	b, err = cdc.MarshalBinaryLengthPrefixed(st)
	assert.NoError(t, err)

	var tStruct tArr
	err = cdc.UnmarshalBinaryLengthPrefixed(b, &tStruct)
	assert.NoError(t, err)
	assert.Equal(t, st, tStruct)
}

func TestMinMaxTimeEncode(t *testing.T) {
	tMin, err := time.Parse("2006-01-02 15:04:05 +0000 UTC", "0001-01-01 00:00:00 +0000 UTC")
	assert.NoError(t, err)
	tm := testTime{tMin}
	_, err = cdc.MarshalBinaryBare(tm)
	assert.NoError(t, err)


	tErr := time.Unix(minSeconds-1, 0)
	_, err = cdc.MarshalBinaryBare(tErr)
	assert.Error(t, err)
	assert.IsType(t, InvalidTimeErr(""), err)
	t.Log(err)

	tErrMaxSec := time.Unix(maxSeconds, 0)
	_, err = cdc.MarshalBinaryBare(tErrMaxSec)
	assert.Error(t, err)
	assert.IsType(t, InvalidTimeErr(""), err)
	t.Log(err)

	tMaxNs := time.Unix(0, maxNanos)
	_, err = cdc.MarshalBinaryBare(tMaxNs)
	assert.NoError(t, err)

	// we can't construct a time.Time with nanos > maxNanos
	// underlying seconds will be incremented -> still expect an error:
	tErr2 := time.Unix(maxSeconds, maxNanos+1)
	_, err = cdc.MarshalBinaryBare(tErr2)
	assert.Error(t, err)
}