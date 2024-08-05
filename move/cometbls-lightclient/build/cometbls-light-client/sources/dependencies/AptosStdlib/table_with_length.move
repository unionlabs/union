/// Extends Table and provides functions such as length and the ability to be destroyed

module aptos_std::table_with_length {
    use std::error;
    use aptos_std::table::{Self, Table};

    // native code raises this with error::invalid_arguments()
    const EALREADY_EXISTS: u64 = 100;
    // native code raises this with error::invalid_arguments()
    const ENOT_FOUND: u64 = 101;
    const ENOT_EMPTY: u64 = 102;

    /// Type of tables
    struct TableWithLength<phantom K: copy + drop, phantom V> has store {
        inner: Table<K, V>,
        length: u64,
    }

    /// Create a new Table.
    public fun new<K: copy + drop, V: store>(): TableWithLength<K, V> {
        TableWithLength {
            inner: table::new<K, V>(),
            length: 0,
        }
    }

    /// Destroy a table. The table must be empty to succeed.
    public fun destroy_empty<K: copy + drop, V>(table: TableWithLength<K, V>) {
        assert!(table.length == 0, error::invalid_state(ENOT_EMPTY));
        let TableWithLength { inner, length: _ } = table;
        table::destroy(inner)
    }

    /// Add a new entry to the table. Aborts if an entry for this
    /// key already exists. The entry itself is not stored in the
    /// table, and cannot be discovered from it.
    public fun add<K: copy + drop, V>(table: &mut TableWithLength<K, V>, key: K, val: V) {
        table::add(&mut table.inner, key, val);
        table.length = table.length + 1;
    }

    /// Acquire an immutable reference to the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public fun borrow<K: copy + drop, V>(table: &TableWithLength<K, V>, key: K): &V {
        table::borrow(&table.inner, key)
    }

    /// Acquire a mutable reference to the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public fun borrow_mut<K: copy + drop, V>(table: &mut TableWithLength<K, V>, key: K): &mut V {
        table::borrow_mut(&mut table.inner, key)
    }

    /// Returns the length of the table, i.e. the number of entries.
    public fun length<K: copy + drop, V>(table: &TableWithLength<K, V>): u64 {
        table.length
    }

    /// Returns true if this table is empty.
    public fun empty<K: copy + drop, V>(table: &TableWithLength<K, V>): bool {
        table.length == 0
    }

    /// Acquire a mutable reference to the value which `key` maps to.
    /// Insert the pair (`key`, `default`) first if there is no entry for `key`.
    public fun borrow_mut_with_default<K: copy + drop, V: drop>(table: &mut TableWithLength<K, V>, key: K, default: V): &mut V {
        if (table::contains(&table.inner, key)) {
            table::borrow_mut(&mut table.inner, key)
        } else {
            table::add(&mut table.inner, key, default);
            table.length = table.length + 1;
            table::borrow_mut(&mut table.inner, key)
        }
    }

    /// Insert the pair (`key`, `value`) if there is no entry for `key`.
    /// update the value of the entry for `key` to `value` otherwise
    public fun upsert<K: copy + drop, V: drop>(table: &mut TableWithLength<K, V>, key: K, value: V) {
        if (!table::contains(&table.inner, key)) {
            add(table, copy key, value)
        } else {
            let ref = table::borrow_mut(&mut table.inner, key);
            *ref = value;
        };
    }

    /// Remove from `table` and return the value which `key` maps to.
    /// Aborts if there is no entry for `key`.
    public fun remove<K: copy + drop, V>(table: &mut TableWithLength<K, V>, key: K): V {
        let val = table::remove(&mut table.inner, key);
        table.length = table.length - 1;
        val
    }

    /// Returns true iff `table` contains an entry for `key`.
    public fun contains<K: copy + drop, V>(table: &TableWithLength<K, V>, key: K): bool {
        table::contains(&table.inner, key)
    }

    #[test_only]
    /// Drop table even if not empty, only when testing.
    public fun drop_unchecked<K: copy + drop, V>(table: TableWithLength<K, V>) {
        // Unpack table with length, dropping length count but not
        // inner table.
        let TableWithLength{inner, length: _} = table;
        table::drop_unchecked(inner); // Drop inner table.
    }

    #[test]
    /// Verify test-only drop functionality.
    fun test_drop_unchecked() {
        let table = new<bool, bool>(); // Declare new table.
        add(&mut table, true, false); // Add table entry.
        drop_unchecked(table); // Drop table.
    }

    #[test]
    fun test_upsert() {
        let t = new<u8, u8>();
        // Table should not have key 0 yet
        assert!(!contains(&t, 0), 1);
        // This should insert key 0, with value 10, and length should be 1
        upsert(&mut t, 0, 10);
        // Ensure the value is correctly set to 10
        assert!(*borrow(&t, 0) == 10, 1);
        // Ensure the length is correctly set
        assert!(length(&t) == 1, 1);
        // Lets upsert the value to something else, and verify it's correct
        upsert(&mut t, 0, 23);
        assert!(*borrow(&t, 0) == 23, 1);
        // Since key 0 already existed, the length should not have changed
        assert!(length(&t) == 1, 1);
        // If we upsert a non-existing key, the length should increase
        upsert(&mut t, 1, 7);
        assert!(length(&t) == 2, 1);

        remove(&mut t, 0);
        remove(&mut t, 1);
        destroy_empty(t);
    }
}
