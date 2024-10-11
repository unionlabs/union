// Copyright 2016 Michal Witkowski. All Rights Reserved.
// See LICENSE for licensing terms.

package metautils_test

import (
	"context"
	"testing"

	"github.com/grpc-ecosystem/go-grpc-middleware/util/metautils"
	"github.com/stretchr/testify/assert"
	"google.golang.org/grpc/metadata"
)

var (
	testPairs = []string{"singlekey", "uno", "multikey", "one", "multikey", "two", "multikey", "three"}
	parentCtx = context.WithValue(context.TODO(), "parentKey", "parentValue")
)

func assertRetainsParentContext(t *testing.T, ctx context.Context) {
	x := ctx.Value("parentKey")
	assert.EqualValues(t, "parentValue", x, "context must contain parentCtx")
}

func TestNiceMD_Get(t *testing.T) {
	nmd := metautils.NiceMD(metadata.Pairs(testPairs...))
	assert.Equal(t, "uno", nmd.Get("singlekey"), "for present single-key value it should return it")
	assert.Equal(t, "one", nmd.Get("multikey"), "for present multi-key should return first value")
	assert.Empty(t, nmd.Get("nokey"), "for non existing key should return stuff")
}

func TestNiceMD_Del(t *testing.T) {
	nmd := metautils.NiceMD(metadata.Pairs(testPairs...))
	assert.Equal(t, "uno", nmd.Get("singlekey"), "for present single-key value it should return it")
	nmd.Del("singlekey").Del("doesnt exist")
	assert.Empty(t, nmd.Get("singlekey"), "after deletion singlekey shouldn't exist")
}

func TestNiceMD_Add(t *testing.T) {
	nmd := metautils.NiceMD(metadata.Pairs(testPairs...))
	nmd.Add("multikey", "four").Add("newkey", "something")
	assert.EqualValues(t, []string{"one", "two", "three", "four"}, nmd["multikey"], "append should add a new four at the end")
	assert.EqualValues(t, []string{"something"}, nmd["newkey"], "append should be able to create new keys")
}

func TestNiceMD_Set(t *testing.T) {
	nmd := metautils.NiceMD(metadata.Pairs(testPairs...))
	nmd.Set("multikey", "one").Set("newkey", "something").Set("newkey", "another")
	assert.EqualValues(t, []string{"one"}, nmd["multikey"], "set should override existing multi keys")
	assert.EqualValues(t, []string{"another"}, nmd["newkey"], "set should override new keys")
}

func TestNiceMD_SetGet(t *testing.T) {
	nmd := metautils.NiceMD(metadata.Pairs(testPairs...))
	nmd.Set("another-key", "onetwothree")
	assert.EqualValues(t, "onetwothree", nmd.Get("another-key"))
	nmd.Set("another-key-bin", "binarydata")
	assert.EqualValues(t, "binarydata", nmd.Get("another-key-bin"))
}

func TestNiceMD_Clone(t *testing.T) {
	nmd := metautils.NiceMD(metadata.Pairs(testPairs...))
	fullCopied := nmd.Clone()
	assert.Equal(t, len(fullCopied), len(nmd), "clone full should copy all keys")
	assert.Equal(t, "uno", fullCopied.Get("singlekey"), "full copied should have content")
	subCopied := nmd.Clone("multikey")
	assert.Len(t, subCopied, 1, "sub copied clone should only have one key")
	assert.Empty(t, subCopied.Get("singlekey"), "there shouldn't be a singlekey in the subcopied")

	// Test side effects and full copying:
	assert.EqualValues(t, subCopied["multikey"], nmd["multikey"], "before overwrites multikey should have the same values")
	subCopied["multikey"][1] = "modifiedtwo"
	assert.NotEqual(t, subCopied["multikey"], nmd["multikey"], "before overwrites multikey should have the same values")
}

func TestNiceMD_ToOutgoing(t *testing.T) {
	nmd := metautils.NiceMD(metadata.Pairs(testPairs...))
	nCtx := nmd.ToOutgoing(parentCtx)
	assertRetainsParentContext(t, nCtx)

	eCtx := metautils.ExtractOutgoing(nCtx).Clone().Set("newvalue", "something").ToOutgoing(nCtx)
	assertRetainsParentContext(t, eCtx)
	assert.NotEqual(t, metautils.ExtractOutgoing(nCtx), metautils.ExtractOutgoing(eCtx), "the niceMD pointed to by ectx and nctx are different.")
}

func TestNiceMD_ToIncoming(t *testing.T) {
	nmd := metautils.NiceMD(metadata.Pairs(testPairs...))
	nCtx := nmd.ToIncoming(parentCtx)
	assertRetainsParentContext(t, nCtx)

	eCtx := metautils.ExtractIncoming(nCtx).Clone().Set("newvalue", "something").ToIncoming(nCtx)
	assertRetainsParentContext(t, eCtx)
	assert.NotEqual(t, metautils.ExtractIncoming(nCtx), metautils.ExtractIncoming(eCtx), "the niceMD pointed to by ectx and nctx are different.")
}
