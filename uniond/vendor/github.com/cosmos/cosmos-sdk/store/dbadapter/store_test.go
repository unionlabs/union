package dbadapter_test

import (
	"bytes"
	"errors"
	"testing"

	"github.com/cosmos/cosmos-sdk/store/cachekv"

	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/store/dbadapter"
	"github.com/cosmos/cosmos-sdk/store/types"
	"github.com/cosmos/cosmos-sdk/testutil/mock"
)

var errFoo = errors.New("dummy")

func TestAccessors(t *testing.T) {
	mockCtrl := gomock.NewController(t)
	defer mockCtrl.Finish()

	mockDB := mock.NewMockDB(mockCtrl)
	store := dbadapter.Store{mockDB}
	key := []byte("test")
	value := []byte("testvalue")

	require.Panics(t, func() { store.Set(nil, []byte("value")) }, "setting a nil key should panic")
	require.Panics(t, func() { store.Set([]byte(""), []byte("value")) }, "setting an empty key should panic")

	require.Equal(t, types.StoreTypeDB, store.GetStoreType())
	store.GetStoreType()

	retFoo := []byte("xxx")
	mockDB.EXPECT().Get(gomock.Eq(key)).Times(1).Return(retFoo, nil)
	require.True(t, bytes.Equal(retFoo, store.Get(key)))

	mockDB.EXPECT().Get(gomock.Eq(key)).Times(1).Return(nil, errFoo)
	require.Panics(t, func() { store.Get(key) })

	mockDB.EXPECT().Has(gomock.Eq(key)).Times(1).Return(true, nil)
	require.True(t, store.Has(key))

	mockDB.EXPECT().Has(gomock.Eq(key)).Times(1).Return(false, nil)
	require.False(t, store.Has(key))

	mockDB.EXPECT().Has(gomock.Eq(key)).Times(1).Return(false, errFoo)
	require.Panics(t, func() { store.Has(key) })

	mockDB.EXPECT().Set(gomock.Eq(key), gomock.Eq(value)).Times(1).Return(nil)
	require.NotPanics(t, func() { store.Set(key, value) })

	mockDB.EXPECT().Set(gomock.Eq(key), gomock.Eq(value)).Times(1).Return(errFoo)
	require.Panics(t, func() { store.Set(key, value) })

	mockDB.EXPECT().Delete(gomock.Eq(key)).Times(1).Return(nil)
	require.NotPanics(t, func() { store.Delete(key) })

	mockDB.EXPECT().Delete(gomock.Eq(key)).Times(1).Return(errFoo)
	require.Panics(t, func() { store.Delete(key) })

	start, end := []byte("start"), []byte("end")
	mockDB.EXPECT().Iterator(gomock.Eq(start), gomock.Eq(end)).Times(1).Return(nil, nil)
	require.NotPanics(t, func() { store.Iterator(start, end) })

	mockDB.EXPECT().Iterator(gomock.Eq(start), gomock.Eq(end)).Times(1).Return(nil, errFoo)
	require.Panics(t, func() { store.Iterator(start, end) })

	mockDB.EXPECT().ReverseIterator(gomock.Eq(start), gomock.Eq(end)).Times(1).Return(nil, nil)
	require.NotPanics(t, func() { store.ReverseIterator(start, end) })

	mockDB.EXPECT().ReverseIterator(gomock.Eq(start), gomock.Eq(end)).Times(1).Return(nil, errFoo)
	require.Panics(t, func() { store.ReverseIterator(start, end) })
}

func TestCacheWraps(t *testing.T) {
	mockCtrl := gomock.NewController(t)
	mockDB := mock.NewMockDB(mockCtrl)
	store := dbadapter.Store{mockDB}

	cacheWrapper := store.CacheWrap()
	require.IsType(t, &cachekv.Store{}, cacheWrapper)

	cacheWrappedWithTrace := store.CacheWrapWithTrace(nil, nil)
	require.IsType(t, &cachekv.Store{}, cacheWrappedWithTrace)
}
