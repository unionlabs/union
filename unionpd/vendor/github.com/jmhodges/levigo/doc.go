/*

Package levigo provides the ability to create and access LevelDB databases.

levigo.Open opens and creates databases.

	opts := levigo.NewOptions()
	opts.SetCache(levigo.NewLRUCache(3<<30))
	opts.SetCreateIfMissing(true)
	db, err := levigo.Open("/path/to/db", opts)

The DB struct returned by Open provides DB.Get, DB.Put and DB.Delete to modify
and query the database.

	ro := levigo.NewReadOptions()
	wo := levigo.NewWriteOptions()
	// if ro and wo are not used again, be sure to Close them.
	data, err := db.Get(ro, []byte("key"))
	...
	err = db.Put(wo, []byte("anotherkey"), data)
	...
	err = db.Delete(wo, []byte("key"))

For bulk reads, use an Iterator. If you want to avoid disturbing your live
traffic while doing the bulk read, be sure to call SetFillCache(false) on the
ReadOptions you use when creating the Iterator.

	ro := levigo.NewReadOptions()
	ro.SetFillCache(false)
	it := db.NewIterator(ro)
	defer it.Close()
	for it.Seek(mykey); it.Valid(); it.Next() {
		munge(it.Key(), it.Value())
	}
	if err := it.GetError(); err != nil {
		...
	}

Batched, atomic writes can be performed with a WriteBatch and
DB.Write.

	wb := levigo.NewWriteBatch()
	// defer wb.Close or use wb.Clear and reuse.
	wb.Delete([]byte("removed"))
	wb.Put([]byte("added"), []byte("data"))
	wb.Put([]byte("anotheradded"), []byte("more"))
	err := db.Write(wo, wb)

If your working dataset does not fit in memory, you'll want to add a bloom
filter to your database. NewBloomFilter and Options.SetFilterPolicy is what
you want. NewBloomFilter is amount of bits in the filter to use per key in
your database.

	filter := levigo.NewBloomFilter(10)
	opts.SetFilterPolicy(filter)
	db, err := levigo.Open("/path/to/db", opts)

If you're using a custom comparator in your code, be aware you may have to
make your own filter policy object.

This documentation is not a complete discussion of LevelDB. Please read the
LevelDB documentation <http://code.google.com/p/leveldb> for information on
its operation. You'll find lots of goodies there.
*/
package levigo
