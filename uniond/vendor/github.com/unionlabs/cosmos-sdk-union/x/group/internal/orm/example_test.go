package orm

import (
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/address"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
)

type TestKeeper struct {
	autoUInt64Table                     *AutoUInt64Table
	primaryKeyTable                     *PrimaryKeyTable
	autoUInt64TableModelByMetadataIndex Index
	primaryKeyTableModelByNameIndex     Index
	primaryKeyTableModelByNumberIndex   Index
	primaryKeyTableModelByMetadataIndex Index
}

var (
	AutoUInt64TablePrefix                     = [2]byte{0x0}
	PrimaryKeyTablePrefix                     = [2]byte{0x1}
	AutoUInt64TableSeqPrefix             byte = 0x2
	AutoUInt64TableModelByMetadataPrefix byte = 0x4
	PrimaryKeyTableModelByNamePrefix     byte = 0x5
	PrimaryKeyTableModelByNumberPrefix   byte = 0x6
	PrimaryKeyTableModelByMetadataPrefix byte = 0x7
)

func NewTestKeeper(cdc codec.Codec) TestKeeper {
	k := TestKeeper{}
	var err error
	ac := address.NewBech32Codec("cosmos")

	k.autoUInt64Table, err = NewAutoUInt64Table(AutoUInt64TablePrefix, AutoUInt64TableSeqPrefix, &testdata.TableModel{}, cdc, ac)
	if err != nil {
		panic(err.Error())
	}
	k.autoUInt64TableModelByMetadataIndex, err = NewIndex(k.autoUInt64Table, AutoUInt64TableModelByMetadataPrefix, func(val interface{}) ([]interface{}, error) {
		return []interface{}{val.(*testdata.TableModel).Metadata}, nil
	}, testdata.TableModel{}.Metadata)
	if err != nil {
		panic(err.Error())
	}

	k.primaryKeyTable, err = NewPrimaryKeyTable(PrimaryKeyTablePrefix, &testdata.TableModel{}, cdc, ac)
	if err != nil {
		panic(err.Error())
	}
	k.primaryKeyTableModelByNameIndex, err = NewIndex(k.primaryKeyTable, PrimaryKeyTableModelByNamePrefix, func(val interface{}) ([]interface{}, error) {
		return []interface{}{val.(*testdata.TableModel).Name}, nil
	}, testdata.TableModel{}.Name)
	if err != nil {
		panic(err.Error())
	}
	k.primaryKeyTableModelByNumberIndex, err = NewIndex(k.primaryKeyTable, PrimaryKeyTableModelByNumberPrefix, func(val interface{}) ([]interface{}, error) {
		return []interface{}{val.(*testdata.TableModel).Number}, nil
	}, testdata.TableModel{}.Number)
	if err != nil {
		panic(err.Error())
	}
	k.primaryKeyTableModelByMetadataIndex, err = NewIndex(k.primaryKeyTable, PrimaryKeyTableModelByMetadataPrefix, func(val interface{}) ([]interface{}, error) {
		return []interface{}{val.(*testdata.TableModel).Metadata}, nil
	}, testdata.TableModel{}.Metadata)
	if err != nil {
		panic(err.Error())
	}

	return k
}
