// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// Defines a Display struct which defines the way an Object
/// should be displayed. The intention is to keep data as independent
/// from its display as possible, protecting the development process
/// and keeping it separate from the ecosystem agreements.
///
/// Each of the fields of the Display object should allow for pattern
/// substitution and filling-in the pieces using the data from the object T.
///
/// More entry functions might be added in the future depending on the use cases.
module sui::display;

use std::string::String;
use sui::event;
use sui::package::Publisher;
use sui::vec_map::{Self, VecMap};

/// For when T does not belong to the package `Publisher`.
const ENotOwner: u64 = 0;

/// For when vectors passed into one of the multiple insert functions
/// don't match in their lengths.
const EVecLengthMismatch: u64 = 1;

/// The Display<T> object. Defines the way a T instance should be
/// displayed. Display object can only be created and modified with
/// a PublisherCap, making sure that the rules are set by the owner
/// of the type.
///
/// Each of the display properties should support patterns outside
/// of the system, making it simpler to customize Display based
/// on the property values of an Object.
/// ```
/// // Example of a display object
/// Display<0x...::capy::Capy> {
///  fields:
///    <name, "Capy { genes }">
///    <link, "https://capy.art/capy/{ id }">
///    <image, "https://api.capy.art/capy/{ id }/svg">
///    <description, "Lovely Capy, one of many">
/// }
/// ```
///
/// Uses only String type due to external-facing nature of the object,
/// the property names have a priority over their types.
public struct Display<phantom T: key> has key, store {
    id: UID,
    /// Contains fields for display. Currently supported
    /// fields are: name, link, image and description.
    fields: VecMap<String, String>,
    /// Version that can only be updated manually by the Publisher.
    version: u16,
}

/// Event: emitted when a new Display object has been created for type T.
/// Type signature of the event corresponds to the type while id serves for
/// the discovery.
///
/// Since Sui RPC supports querying events by type, finding a Display for the T
/// would be as simple as looking for the first event with `Display<T>`.
public struct DisplayCreated<phantom T: key> has copy, drop {
    id: ID,
}

/// Version of Display got updated -
public struct VersionUpdated<phantom T: key> has copy, drop {
    id: ID,
    version: u16,
    fields: VecMap<String, String>,
}

// === Initializer Methods ===

/// Create an empty Display object. It can either be shared empty or filled
/// with data right away via cheaper `set_owned` method.
public fun new<T: key>(pub: &Publisher, ctx: &mut TxContext): Display<T> {
    assert!(is_authorized<T>(pub), ENotOwner);
    create_internal(ctx)
}

/// Create a new Display<T> object with a set of fields.
public fun new_with_fields<T: key>(
    pub: &Publisher,
    fields: vector<String>,
    values: vector<String>,
    ctx: &mut TxContext,
): Display<T> {
    let len = fields.length();
    assert!(len == values.length(), EVecLengthMismatch);

    let mut i = 0;
    let mut display = new<T>(pub, ctx);
    while (i < len) {
        display.add_internal(fields[i], values[i]);
        i = i + 1;
    };

    display
}

// === Entry functions: Create ===

#[allow(lint(self_transfer))]
/// Create a new empty Display<T> object and keep it.
public entry fun create_and_keep<T: key>(pub: &Publisher, ctx: &mut TxContext) {
    transfer::public_transfer(new<T>(pub, ctx), ctx.sender())
}

/// Manually bump the version and emit an event with the updated version's contents.
public entry fun update_version<T: key>(display: &mut Display<T>) {
    display.version = display.version + 1;
    event::emit(VersionUpdated<T> {
        version: display.version,
        fields: *&display.fields,
        id: display.id.to_inner(),
    })
}

// === Entry functions: Add/Modify fields ===

/// Sets a custom `name` field with the `value`.
public entry fun add<T: key>(self: &mut Display<T>, name: String, value: String) {
    self.add_internal(name, value)
}

/// Sets multiple `fields` with `values`.
public entry fun add_multiple<T: key>(
    self: &mut Display<T>,
    fields: vector<String>,
    values: vector<String>,
) {
    let len = fields.length();
    assert!(len == values.length(), EVecLengthMismatch);

    let mut i = 0;
    while (i < len) {
        self.add_internal(fields[i], values[i]);
        i = i + 1;
    };
}

/// Change the value of the field.
/// TODO (long run): version changes;
public entry fun edit<T: key>(self: &mut Display<T>, name: String, value: String) {
    let (_, _) = self.fields.remove(&name);
    self.add_internal(name, value)
}

/// Remove the key from the Display.
public entry fun remove<T: key>(self: &mut Display<T>, name: String) {
    self.fields.remove(&name);
}

// === Access fields ===

/// Authorization check; can be performed externally to implement protection rules for Display.
public fun is_authorized<T: key>(pub: &Publisher): bool {
    pub.from_package<T>()
}

/// Read the `version` field.
public fun version<T: key>(d: &Display<T>): u16 {
    d.version
}

/// Read the `fields` field.
public fun fields<T: key>(d: &Display<T>): &VecMap<String, String> {
    &d.fields
}

// === Private functions ===

/// Internal function to create a new `Display<T>`.
fun create_internal<T: key>(ctx: &mut TxContext): Display<T> {
    let uid = object::new(ctx);

    event::emit(DisplayCreated<T> {
        id: uid.to_inner(),
    });

    Display {
        id: uid,
        fields: vec_map::empty(),
        version: 0,
    }
}

/// Private method for inserting fields without security checks.
fun add_internal<T: key>(display: &mut Display<T>, name: String, value: String) {
    display.fields.insert(name, value)
}
