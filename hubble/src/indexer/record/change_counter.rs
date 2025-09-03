use std::{
    collections::HashMap,
    fmt,
    ops::{Add, AddAssign},
};

/// Represents the type of database operation performed on a record.
#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum ChangeType {
    Insert,
    Delete,
    #[allow(dead_code)]
    Update,
}

/// Tracks database changes by record type and operation type.
///
/// # Design Invariants
///
/// This struct maintains several important invariants that allow for performance optimizations:
///
/// 1. **No Zero Values**: All stored counts are guaranteed to be > 0. We only add positive
///    values and only use addition operations (never subtraction).
///
/// 2. **No Empty Collections**: Once a `RecordKind` has changes, its inner `HashMap<ChangeType, u64>`
///    will never be empty. We only add entries, never remove them.
///
/// 3. **Additive Only**: The struct only supports adding changes via `+=` operations. There are
///    no operations that subtract or remove changes.
///
/// These invariants allow us to:
/// - Use `contains_key()` instead of checking if values are > 0
/// - Use `is_empty()` only on the outer HashMap  
/// - Skip defensive checks for zero values or empty collections
///
/// # Structure
///
/// ```text
/// HashMap<RecordKind, HashMap<ChangeType, u64>>
/// │                   │                   │
/// │                   │                   └─ Count (always > 0)
/// │                   └─ Operation type (Insert/Delete/Update)  
/// └─ Record type (ChannelOpenInit, PacketSend, etc.)
/// ```
///
/// # Example
///
/// ```rust
/// let mut changes = Changes::default();
/// changes.change::<MyRecord>(ChangeType::Insert, 5);
/// changes.change::<MyRecord>(ChangeType::Delete, 2);
///
/// // Check for any changes to specific record types
/// const PACKET_KINDS: &[RecordKind] = &[RecordKind::PacketSend, RecordKind::PacketRecv];
/// if changes.has_changes_for(PACKET_KINDS) {
///     println!("Packet records were modified");
/// }
/// ```
#[derive(Default)]
pub struct Changes {
    changes: HashMap<RecordKind, HashMap<ChangeType, u64>>,
}

/// Constant for single-change operations to avoid magic numbers.
const SINGLE_CHANGE: u64 = 1;

impl Changes {
    /// Returns `true` if no changes have been recorded.
    ///
    /// Due to our invariants, we only need to check if the outer HashMap is empty.
    /// Once any changes are recorded, inner HashMaps will never be empty.
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    #[allow(dead_code)]
    pub fn has_changes(&self) -> bool {
        !self.is_empty()
    }

    /// Checks if there are any changes for any of the specified record kinds.
    ///
    /// # Arguments
    /// * `record_kinds` - Slice of record kinds to check. Works well with static arrays.
    ///
    /// # Example
    /// ```rust
    /// const PACKET_KINDS: &[RecordKind] = &[RecordKind::PacketSend, RecordKind::PacketRecv];
    /// if changes.has_changes_for(PACKET_KINDS) {
    ///     println!("Packet records were modified");
    /// }
    /// ```
    pub fn has_changes_for(&self, record_kinds: &[RecordKind]) -> bool {
        record_kinds
            .iter()
            .any(|kind| self.changes.contains_key(kind))
    }

    /// Checks if there are any changes for record kinds that match the provided predicate function.
    ///
    /// This method allows for more flexible filtering logic while maintaining compile-time
    /// exhaustiveness guarantees when the predicate function uses exhaustive pattern matching.
    ///
    /// # Arguments
    /// * `predicate` - Function that takes a RecordKind and returns true if changes for that kind should be considered
    ///
    /// # Example
    /// ```rust
    /// const fn is_channel_event(kind: RecordKind) -> bool {
    ///     use RecordKind::*;
    ///     match kind {
    ///         ChannelOpenInit | ChannelOpenTry => true,
    ///         _ => false,
    ///     }
    /// }
    ///
    /// if changes.has_changes_matching(is_channel_event) {
    ///     println!("Channel events were modified");
    /// }
    /// ```
    pub fn has_changes_matching<F>(&self, predicate: F) -> bool
    where
        F: Fn(RecordKind) -> bool,
    {
        self.changes.keys().any(|&kind| predicate(kind))
    }

    /// Checks if there are changes of a specific type for any of the specified record kinds.
    ///
    /// # Arguments
    /// * `record_kinds` - Slice of record kinds to check
    /// * `change_type` - Specific type of change to look for
    ///
    /// # Example
    /// ```rust
    /// const CHANNEL_KINDS: &[RecordKind] = &[RecordKind::ChannelOpenInit, RecordKind::ChannelOpenTry];
    /// if changes.has_changes_for_type(CHANNEL_KINDS, ChangeType::Insert) {
    ///     println!("New channel records were created");
    /// }
    /// ```
    #[allow(dead_code)]
    pub fn has_changes_for_type(
        &self,
        record_kinds: &[RecordKind],
        change_type: ChangeType,
    ) -> bool {
        record_kinds.iter().any(|kind| {
            self.changes
                .get(kind)
                .map(|change_map| change_map.contains_key(&change_type))
                .unwrap_or(false)
        })
    }

    /// Creates a new `Changes` instance with a single change entry.
    ///
    /// This is the most flexible constructor, allowing any combination of record type,
    /// change type, and count.
    ///
    /// # Arguments
    /// * `change_type` - The type of change (Insert/Delete/Update)
    /// * `count` - Number of changes (should be > 0 to maintain invariants)
    ///
    /// # Example
    /// ```rust
    /// let changes = Changes::with::<MyRecord>(ChangeType::Insert, 10);
    /// ```
    pub fn with<T: HasKind>(change_type: ChangeType, count: u64) -> Self {
        let mut result = Self::default();
        result.change::<T>(change_type, count);
        result
    }

    #[allow(dead_code)]
    pub fn with_single_delete<T: HasKind>() -> Self {
        Self::with_deletes::<T>(SINGLE_CHANGE)
    }

    pub fn with_single_insert<T: HasKind>() -> Self {
        Self::with_inserts::<T>(SINGLE_CHANGE)
    }

    #[allow(dead_code)]
    pub fn with_single_update<T: HasKind>() -> Self {
        Self::with_updates::<T>(SINGLE_CHANGE)
    }

    pub fn with_deletes<T: HasKind>(count: u64) -> Self {
        Self::with::<T>(ChangeType::Delete, count)
    }

    #[allow(dead_code)]
    pub fn with_inserts<T: HasKind>(count: u64) -> Self {
        Self::with::<T>(ChangeType::Insert, count)
    }

    #[allow(dead_code)]
    pub fn with_updates<T: HasKind>(count: u64) -> Self {
        Self::with::<T>(ChangeType::Update, count)
    }

    #[allow(dead_code)]
    pub fn delete<T: HasKind>(&mut self, count: u64) {
        self.change::<T>(ChangeType::Delete, count);
    }

    #[allow(dead_code)]
    pub fn insert<T: HasKind>(&mut self, count: u64) {
        self.change::<T>(ChangeType::Insert, count);
    }

    #[allow(dead_code)]
    pub fn update<T: HasKind>(&mut self, count: u64) {
        self.change::<T>(ChangeType::Update, count);
    }

    /// Records a change for the specified record type and change type.
    ///
    /// This is the core method that all other change-recording methods delegate to.
    ///
    /// # Arguments
    /// * `change_type` - The type of change being recorded
    /// * `count` - Number of changes to add (should be > 0)
    ///
    /// # Note
    /// This method maintains our invariants by only adding positive values.
    /// Zero values are ignored to maintain the "no zero values" invariant.
    pub fn change<T: HasKind>(&mut self, change_type: ChangeType, count: u64) {
        // Maintain invariant: only store positive values
        if count > 0 {
            let kind = T::kind();
            *self
                .changes
                .entry(kind)
                .or_default()
                .entry(change_type)
                .or_default() += count;
        }
    }
}

impl fmt::Display for Changes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            write!(f, "no changes")
        } else {
            let mut parts = Vec::new();

            // Sort by RecordKind for consistent output
            let mut sorted_changes: Vec<_> = self.changes.iter().collect();
            sorted_changes.sort_by_key(|(kind, _)| format!("{:?}", kind));

            for (kind, change_map) in sorted_changes {
                // Sort by ChangeType for consistent output
                let mut sorted_change_map: Vec<_> = change_map.iter().collect();
                sorted_change_map.sort_by_key(|(change_type, _)| format!("{:?}", change_type));

                let mut change_parts = Vec::new();
                for (change_type, count) in sorted_change_map {
                    // Our design invariants guarantee all stored counts are > 0
                    let change_code = match change_type {
                        ChangeType::Insert => "I",
                        ChangeType::Delete => "D",
                        ChangeType::Update => "U",
                    };
                    change_parts.push(format!("{}={}", change_code, count));
                }
                parts.push(format!("{:?}:{}", kind, change_parts.join(",")));
            }
            write!(f, "{}", parts.join(", "))
        }
    }
}

impl Add for Changes {
    type Output = Changes;

    fn add(self, rhs: Changes) -> Self::Output {
        let mut result = self;
        result += rhs;
        result
    }
}

impl AddAssign for Changes {
    fn add_assign(&mut self, rhs: Changes) {
        for (kind, change_map) in rhs.changes {
            let self_change_map = self.changes.entry(kind).or_default();
            for (change_type, count) in change_map {
                *self_change_map.entry(change_type).or_default() += count;
            }
        }
    }
}

pub struct LegacyRecord {}

impl HasKind for LegacyRecord {
    fn kind() -> RecordKind {
        RecordKind::Legacy
    }
}

/// Represents the different types of database records that can be changed.
///
/// This enum is used as a key to categorize changes by the type of record being modified.
/// Each variant corresponds to a specific database table or record type in the indexer.
#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum RecordKind {
    Legacy,
    ChannelOpenInit,
    ChannelOpenTry,
    ChannelOpenAck,
    ChannelOpenConfirm,
    ConnectionOpenInit,
    ConnectionOpenTry,
    ConnectionOpenAck,
    ConnectionOpenConfirm,
    CreateClient,
    CreateLensClient,
    UpdateClient,
    PacketSend,
    PacketRecv,
    WriteAck,
    PacketAck,
    PacketTimeout,
    TokenBucketUpdate,
    WalletMutationEntry,
    PacketSendBond,
    PacketSendDecoded,
    PacketSendTransfers,
    PacketSendInstructionsSearch,
    PacketSendUnbond,
    CreateWrappedToken,
    CreateWrappedTokenRelation,
}

/// Trait for types that can be associated with a specific `RecordKind`.
///
/// This trait allows the `Changes` struct to work with type-safe record types
/// while mapping them to their corresponding `RecordKind` for storage.
///
/// # Example Implementation
/// ```rust
/// struct MyRecord;
/// impl HasKind for MyRecord {
///     fn kind() -> RecordKind {
///         RecordKind::PacketSend
///     }
/// }
/// ```
pub trait HasKind {
    /// Returns the `RecordKind` associated with this type.
    fn kind() -> RecordKind;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestRecord;
    impl HasKind for TestRecord {
        fn kind() -> RecordKind {
            RecordKind::ChannelOpenInit
        }
    }

    #[test]
    fn test_is_empty() {
        // Default changes should be empty
        let changes = Changes::default();
        assert!(changes.is_empty());

        // Changes with insert should not be empty
        let changes = Changes::with_single_insert::<TestRecord>();
        assert!(!changes.is_empty());

        // Changes with delete should not be empty
        let changes = Changes::with_single_delete::<TestRecord>();
        assert!(!changes.is_empty());

        // Changes with update should not be empty
        let changes = Changes::with_single_update::<TestRecord>();
        assert!(!changes.is_empty());

        // Combined changes should not be empty
        let changes1 = Changes::with_single_insert::<TestRecord>();
        let changes2 = Changes::with_single_delete::<TestRecord>();
        let combined = changes1 + changes2;
        assert!(!combined.is_empty());
    }

    #[test]
    fn test_new_structure() {
        // Test that the new nested HashMap structure works correctly
        let mut changes = Changes::default();
        changes.change::<TestRecord>(ChangeType::Insert, 5);
        changes.change::<TestRecord>(ChangeType::Delete, 3);

        assert!(!changes.is_empty());
        assert!(changes.has_changes());

        // Test += operator with new structure
        let mut changes1 = Changes::with_single_insert::<TestRecord>();
        let changes2 = Changes::with_single_delete::<TestRecord>();
        changes1 += changes2;

        assert!(!changes1.is_empty());
    }

    #[test]
    fn test_generic_with_function() {
        // Test the new generic with function
        let changes_insert = Changes::with::<TestRecord>(ChangeType::Insert, 5);
        let changes_delete = Changes::with::<TestRecord>(ChangeType::Delete, 3);
        let changes_update = Changes::with::<TestRecord>(ChangeType::Update, 2);

        assert!(!changes_insert.is_empty());
        assert!(!changes_delete.is_empty());
        assert!(!changes_update.is_empty());

        // Test that it's equivalent to the specific methods
        let specific_insert = Changes::with_inserts::<TestRecord>(5);
        let specific_delete = Changes::with_deletes::<TestRecord>(3);
        let specific_update = Changes::with_updates::<TestRecord>(2);

        // Both approaches should create equivalent structures
        assert_eq!(changes_insert.has_changes(), specific_insert.has_changes());
        assert_eq!(changes_delete.has_changes(), specific_delete.has_changes());
        assert_eq!(changes_update.has_changes(), specific_update.has_changes());
    }

    #[test]
    fn test_has_changes_for() {
        struct AnotherTestRecord;
        impl HasKind for AnotherTestRecord {
            fn kind() -> RecordKind {
                RecordKind::PacketSend
            }
        }

        // Create changes for different record types
        let mut changes = Changes::default();
        changes.change::<TestRecord>(ChangeType::Insert, 3);
        changes.change::<AnotherTestRecord>(ChangeType::Delete, 2);

        // Test has_changes_for with static arrays
        const INTERESTED_KINDS: &[RecordKind] =
            &[RecordKind::ChannelOpenInit, RecordKind::PacketSend];
        const UNRELATED_KINDS: &[RecordKind] =
            &[RecordKind::UpdateClient, RecordKind::TokenBucketUpdate];

        // Should find changes for PacketSend (from AnotherTestRecord)
        assert!(changes.has_changes_for(INTERESTED_KINDS));

        // Should not find changes for unrelated kinds
        assert!(!changes.has_changes_for(UNRELATED_KINDS));

        // Test has_changes_for_type
        assert!(changes.has_changes_for_type(INTERESTED_KINDS, ChangeType::Delete));
        assert!(!changes.has_changes_for_type(INTERESTED_KINDS, ChangeType::Update));
        assert!(!changes.has_changes_for_type(UNRELATED_KINDS, ChangeType::Delete));

        // Test with ChannelOpenInit (from TestRecord)
        const CHANNEL_KINDS: &[RecordKind] = &[RecordKind::ChannelOpenInit];
        assert!(changes.has_changes_for(CHANNEL_KINDS));
        assert!(changes.has_changes_for_type(CHANNEL_KINDS, ChangeType::Insert));
        assert!(!changes.has_changes_for_type(CHANNEL_KINDS, ChangeType::Delete));
    }

    #[test]
    fn test_no_zero_values_invariant() {
        struct TestRecord;
        impl HasKind for TestRecord {
            fn kind() -> RecordKind {
                RecordKind::ChannelOpenInit
            }
        }

        // Test that we never store zero values
        let mut changes = Changes::default();

        // This should not create any entries since count is 0
        changes.change::<TestRecord>(ChangeType::Insert, 0);

        // The changes should still be empty
        assert!(
            changes.is_empty(),
            "Adding 0 count should not create any entries"
        );

        // Test adding a positive value, then checking no zeros are stored
        changes.change::<TestRecord>(ChangeType::Insert, 5);

        // Verify no zero values exist in the internal structure
        for change_map in changes.changes.values() {
            for count in change_map.values() {
                assert!(
                    *count > 0,
                    "Found zero value in changes structure: {}",
                    count
                );
            }
        }

        // Test that += with 0 doesn't create entries
        let zero_changes = Changes::with::<TestRecord>(ChangeType::Delete, 0);
        let original_display = format!("{}", changes);
        changes += zero_changes;
        let new_display = format!("{}", changes);

        // Adding zero changes should not modify the structure
        assert_eq!(
            original_display, new_display,
            "Adding zero changes modified the structure"
        );
    }

    #[test]
    fn test_display_implementation() {
        struct TestRecord;
        impl HasKind for TestRecord {
            fn kind() -> RecordKind {
                RecordKind::ChannelOpenInit
            }
        }

        struct AnotherTestRecord;
        impl HasKind for AnotherTestRecord {
            fn kind() -> RecordKind {
                RecordKind::PacketSend
            }
        }

        // Test empty changes
        let changes = Changes::default();
        assert_eq!(format!("{}", changes), "no changes");

        // Test single change
        let changes = Changes::with_single_insert::<TestRecord>();
        assert_eq!(format!("{}", changes), "ChannelOpenInit:I=1");

        // Test multiple changes for same record type
        let mut changes = Changes::default();
        changes.change::<TestRecord>(ChangeType::Insert, 3);
        changes.change::<TestRecord>(ChangeType::Delete, 2);
        // Should be sorted by ChangeType (Delete comes before Insert alphabetically)
        assert_eq!(format!("{}", changes), "ChannelOpenInit:D=2,I=3");

        // Test multiple record types
        let mut changes = Changes::default();
        changes.change::<AnotherTestRecord>(ChangeType::Insert, 5);
        changes.change::<TestRecord>(ChangeType::Delete, 1);
        // Should be sorted by RecordKind (ChannelOpenInit comes before PacketSend alphabetically)
        assert_eq!(
            format!("{}", changes),
            "ChannelOpenInit:D=1, PacketSend:I=5"
        );

        // Test that only non-zero values would be shown (though our invariants prevent zero values)
        let changes = Changes::with::<TestRecord>(ChangeType::Update, 42);
        assert_eq!(format!("{}", changes), "ChannelOpenInit:U=42");
    }
}
