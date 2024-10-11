package testing

import (
	"errors"
	"reflect"
	"strconv"
	"strings"
	"testing"

	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	pb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestSSZTagSize(t *testing.T) {
	sigSize := fieldparams.BLSSignatureLength
	pubKeySize := 48
	rootSize := 32

	sizes, err := sszTagSizes(pb.Attestation{}, "Signature")
	require.NoError(t, err)
	assert.Equal(t, sigSize, sizes[0], "Unexpected signature size")

	sizes, err = sszTagSizes(pb.SignedBeaconBlock{}, "Signature")
	require.NoError(t, err)
	assert.Equal(t, sigSize, sizes[0], "Unexpected signature size")

	sizes, err = sszTagSizes(pb.Checkpoint{}, "Root")
	require.NoError(t, err)
	assert.Equal(t, rootSize, sizes[0], "Unexpected signature size")

	sizes, err = sszTagSizes(pb.Validator{}, "PublicKey")
	require.NoError(t, err)
	assert.Equal(t, pubKeySize, sizes[0], "Unexpected signature size")
}

func sszTagSizes(i interface{}, fName string) ([]int, error) {
	v := reflect.ValueOf(i)
	field, exists := v.Type().FieldByName(fName)
	if !exists {
		return nil, errors.New("wanted field does not exist")
	}
	tag, exists := field.Tag.Lookup("ssz-size")
	if !exists {
		return nil, errors.New("wanted field does not contain ssz-size tag")
	}
	start := strings.IndexRune(tag, '=')
	items := strings.Split(tag[start+1:], ",")
	sizes := make([]int, len(items))
	var err error
	for i := 0; i < len(items); i++ {
		if items[i] == "?" {
			sizes[i] = 0
			continue
		}
		sizes[i], err = strconv.Atoi(items[i])
		if err != nil {
			return nil, err
		}
	}
	return sizes, nil
}
