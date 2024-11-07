// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#[allow(unused_const)]
/// In addition to the fields declared in its type definition, a Sui object can have dynamic fields
/// that can be added after the object has been constructed. Unlike ordinary field names
/// (which are always statically declared identifiers) a dynamic field name can be any value with
/// the `copy`, `drop`, and `store` abilities, e.g. an integer, a boolean, or a string.
/// This gives Sui programmers the flexibility to extend objects on-the-fly, and it also serves as a
/// building block for core collection types
module sui::dynamic_field;

/// The object already has a dynamic field with this name (with the value and type specified)
const EFieldAlreadyExists: u64 = 0;
/// Cannot load dynamic field.
/// The object does not have a dynamic field with this name (with the value and type specified)
const EFieldDoesNotExist: u64 = 1;
/// The object has a field with that name, but the value type does not match
const EFieldTypeMismatch: u64 = 2;
/// Failed to serialize the field's name
const EBCSSerializationFailure: u64 = 3;
/// The object added as a dynamic field was previously a shared object
const ESharedObjectOperationNotSupported: u64 = 4;

/// Internal object used for storing the field and value
public struct Field<Name: copy + drop + store, Value: store> has key {
    /// Determined by the hash of the object ID, the field name value and it's type,
    /// i.e. hash(parent.id || name || Name)
    id: UID,
    /// The value for the name of this field
    name: Name,
    /// The value bound to this field
    value: Value,
}

/// Adds a dynamic field to the object `object: &mut UID` at field specified by `name: Name`.
/// Aborts with `EFieldAlreadyExists` if the object already has that field with that name.
public fun add<Name: copy + drop + store, Value: store>(
    // we use &mut UID in several spots for access control
    object: &mut UID,
    name: Name,
    value: Value,
) {
    let object_addr = object.to_address();
    let hash = hash_type_and_key(object_addr, name);
    assert!(!has_child_object(object_addr, hash), EFieldAlreadyExists);
    let field = Field {
        id: object::new_uid_from_hash(hash),
        name,
        value,
    };
    add_child_object(object_addr, field)
}

/// Immutably borrows the `object`s dynamic field with the name specified by `name: Name`.
/// Aborts with `EFieldDoesNotExist` if the object does not have a field with that name.
/// Aborts with `EFieldTypeMismatch` if the field exists, but the value does not have the specified
/// type.
public fun borrow<Name: copy + drop + store, Value: store>(object: &UID, name: Name): &Value {
    let object_addr = object.to_address();
    let hash = hash_type_and_key(object_addr, name);
    let field = borrow_child_object<Field<Name, Value>>(object, hash);
    &field.value
}

/// Mutably borrows the `object`s dynamic field with the name specified by `name: Name`.
/// Aborts with `EFieldDoesNotExist` if the object does not have a field with that name.
/// Aborts with `EFieldTypeMismatch` if the field exists, but the value does not have the specified
/// type.
public fun borrow_mut<Name: copy + drop + store, Value: store>(
    object: &mut UID,
    name: Name,
): &mut Value {
    let object_addr = object.to_address();
    let hash = hash_type_and_key(object_addr, name);
    let field = borrow_child_object_mut<Field<Name, Value>>(object, hash);
    &mut field.value
}

/// Removes the `object`s dynamic field with the name specified by `name: Name` and returns the
/// bound value.
/// Aborts with `EFieldDoesNotExist` if the object does not have a field with that name.
/// Aborts with `EFieldTypeMismatch` if the field exists, but the value does not have the specified
/// type.
public fun remove<Name: copy + drop + store, Value: store>(object: &mut UID, name: Name): Value {
    let object_addr = object.to_address();
    let hash = hash_type_and_key(object_addr, name);
    let Field { id, name: _, value } = remove_child_object<Field<Name, Value>>(object_addr, hash);
    id.delete();
    value
}

/// Returns true if and only if the `object` has a dynamic field with the name specified by
/// `name: Name` but without specifying the `Value` type
public fun exists_<Name: copy + drop + store>(object: &UID, name: Name): bool {
    let object_addr = object.to_address();
    let hash = hash_type_and_key(object_addr, name);
    has_child_object(object_addr, hash)
}

/// Removes the dynamic field if it exists. Returns the `some(Value)` if it exists or none otherwise.
public fun remove_if_exists<Name: copy + drop + store, Value: store>(
    object: &mut UID,
    name: Name,
): Option<Value> {
    if (exists_<Name>(object, name)) {
        option::some(remove(object, name))
    } else {
        option::none()
    }
}

/// Returns true if and only if the `object` has a dynamic field with the name specified by
/// `name: Name` with an assigned value of type `Value`.
public fun exists_with_type<Name: copy + drop + store, Value: store>(
    object: &UID,
    name: Name,
): bool {
    let object_addr = object.to_address();
    let hash = hash_type_and_key(object_addr, name);
    has_child_object_with_ty<Field<Name, Value>>(object_addr, hash)
}

public(package) fun field_info<Name: copy + drop + store>(
    object: &UID,
    name: Name,
): (&UID, address) {
    let object_addr = object.to_address();
    let hash = hash_type_and_key(object_addr, name);
    let Field { id, name: _, value } = borrow_child_object<Field<Name, ID>>(object, hash);
    (id, value.to_address())
}

public(package) fun field_info_mut<Name: copy + drop + store>(
    object: &mut UID,
    name: Name,
): (&mut UID, address) {
    let object_addr = object.to_address();
    let hash = hash_type_and_key(object_addr, name);
    let Field { id, name: _, value } = borrow_child_object_mut<Field<Name, ID>>(object, hash);
    (id, value.to_address())
}

/// May abort with `EBCSSerializationFailure`.
public(package) native fun hash_type_and_key<K: copy + drop + store>(
    parent: address,
    k: K,
): address;

public(package) native fun add_child_object<Child: key>(parent: address, child: Child);

/// throws `EFieldDoesNotExist` if a child does not exist with that ID
/// or throws `EFieldTypeMismatch` if the type does not match,
/// and may also abort with `EBCSSerializationFailure`
/// we need two versions to return a reference or a mutable reference
public(package) native fun borrow_child_object<Child: key>(object: &UID, id: address): &Child;

public(package) native fun borrow_child_object_mut<Child: key>(
    object: &mut UID,
    id: address,
): &mut Child;

/// throws `EFieldDoesNotExist` if a child does not exist with that ID
/// or throws `EFieldTypeMismatch` if the type does not match,
/// and may also abort with `EBCSSerializationFailure`.
public(package) native fun remove_child_object<Child: key>(parent: address, id: address): Child;

public(package) native fun has_child_object(parent: address, id: address): bool;

public(package) native fun has_child_object_with_ty<Child: key>(parent: address, id: address): bool;
