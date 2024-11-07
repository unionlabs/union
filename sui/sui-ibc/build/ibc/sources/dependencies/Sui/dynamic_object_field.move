// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// Similar to `sui::dynamic_field`, this module allows for the access of dynamic fields. But
/// unlike, `sui::dynamic_field` the values bound to these dynamic fields _must_ be objects
/// themselves. This allows for the objects to still exist within in storage, which may be important
/// for external tools. The difference is otherwise not observable from within Move.
module sui::dynamic_object_field;

use sui::dynamic_field::{
    Self as field,
    add_child_object,
    borrow_child_object,
    borrow_child_object_mut,
    remove_child_object
};

// Internal object used for storing the field and the name associated with the value
// The separate type is necessary to prevent key collision with direct usage of dynamic_field
public struct Wrapper<Name> has copy, drop, store {
    name: Name,
}

/// Adds a dynamic object field to the object `object: &mut UID` at field specified by `name: Name`.
/// Aborts with `EFieldAlreadyExists` if the object already has that field with that name.
public fun add<Name: copy + drop + store, Value: key + store>(
    // we use &mut UID in several spots for access control
    object: &mut UID,
    name: Name,
    value: Value,
) {
    add_impl!(object, name, value)
}

/// Immutably borrows the `object`s dynamic object field with the name specified by `name: Name`.
/// Aborts with `EFieldDoesNotExist` if the object does not have a field with that name.
/// Aborts with `EFieldTypeMismatch` if the field exists, but the value object does not have the
/// specified type.
public fun borrow<Name: copy + drop + store, Value: key + store>(object: &UID, name: Name): &Value {
    borrow_impl!(object, name)
}

/// Mutably borrows the `object`s dynamic object field with the name specified by `name: Name`.
/// Aborts with `EFieldDoesNotExist` if the object does not have a field with that name.
/// Aborts with `EFieldTypeMismatch` if the field exists, but the value object does not have the
/// specified type.
public fun borrow_mut<Name: copy + drop + store, Value: key + store>(
    object: &mut UID,
    name: Name,
): &mut Value {
    borrow_mut_impl!(object, name)
}

/// Removes the `object`s dynamic object field with the name specified by `name: Name` and returns
/// the bound object.
/// Aborts with `EFieldDoesNotExist` if the object does not have a field with that name.
/// Aborts with `EFieldTypeMismatch` if the field exists, but the value object does not have the
/// specified type.
public fun remove<Name: copy + drop + store, Value: key + store>(
    object: &mut UID,
    name: Name,
): Value {
    remove_impl!(object, name)
}

/// Returns true if and only if the `object` has a dynamic object field with the name specified by
/// `name: Name`.
public fun exists_<Name: copy + drop + store>(object: &UID, name: Name): bool {
    let key = Wrapper { name };
    field::exists_with_type<Wrapper<Name>, ID>(object, key)
}

/// Returns true if and only if the `object` has a dynamic field with the name specified by
/// `name: Name` with an assigned value of type `Value`.
public fun exists_with_type<Name: copy + drop + store, Value: key + store>(
    object: &UID,
    name: Name,
): bool {
    exists_with_type_impl!<_, Value>(object, name)
}

/// Returns the ID of the object associated with the dynamic object field
/// Returns none otherwise
public fun id<Name: copy + drop + store>(object: &UID, name: Name): Option<ID> {
    let key = Wrapper { name };
    if (!field::exists_with_type<Wrapper<Name>, ID>(object, key)) return option::none();
    let (_field, value_addr) = field::field_info<Wrapper<Name>>(object, key);
    option::some(value_addr.to_id())
}

public(package) fun internal_add<Name: copy + drop + store, Value: key>(
    // we use &mut UID in several spots for access control
    object: &mut UID,
    name: Name,
    value: Value,
) {
    add_impl!(object, name, value)
}

public(package) fun internal_borrow<Name: copy + drop + store, Value: key>(
    object: &UID,
    name: Name,
): &Value {
    borrow_impl!(object, name)
}

public(package) fun internal_borrow_mut<Name: copy + drop + store, Value: key>(
    object: &mut UID,
    name: Name,
): &mut Value {
    borrow_mut_impl!(object, name)
}

public(package) fun internal_remove<Name: copy + drop + store, Value: key>(
    object: &mut UID,
    name: Name,
): Value {
    remove_impl!(object, name)
}

public(package) fun internal_exists_with_type<Name: copy + drop + store, Value: key>(
    object: &UID,
    name: Name,
): bool {
    exists_with_type_impl!<_, Value>(object, name)
}

macro fun add_impl<$Name: copy + drop + store, $Value: key>(
    // we use &mut UID in several spots for access control
    $object: &mut UID,
    $name: $Name,
    $value: $Value,
) {
    let object = $object;
    let name = $name;
    let value = $value;
    let key = Wrapper { name };
    let id = object::id(&value);
    field::add(object, key, id);
    let (field, _) = field::field_info<Wrapper<$Name>>(object, key);
    add_child_object(field.to_address(), value);
}

macro fun borrow_impl<$Name: copy + drop + store, $Value: key>(
    $object: &UID,
    $name: $Name,
): &$Value {
    let object = $object;
    let name = $name;
    let key = Wrapper { name };
    let (field, value_id) = field::field_info<Wrapper<$Name>>(object, key);
    borrow_child_object<$Value>(field, value_id)
}

macro fun borrow_mut_impl<$Name: copy + drop + store, $Value: key>(
    $object: &mut UID,
    $name: $Name,
): &mut $Value {
    let object = $object;
    let name = $name;
    let key = Wrapper { name };
    let (field, value_id) = field::field_info_mut<Wrapper<$Name>>(object, key);
    borrow_child_object_mut<$Value>(field, value_id)
}

macro fun remove_impl<$Name: copy + drop + store, $Value: key>(
    $object: &mut UID,
    $name: $Name,
): $Value {
    let object = $object;
    let name = $name;
    let key = Wrapper { name };
    let (field, value_id) = field::field_info<Wrapper<$Name>>(object, key);
    let value = remove_child_object<$Value>(field.to_address(), value_id);
    field::remove<Wrapper<$Name>, ID>(object, key);
    value
}

macro fun exists_with_type_impl<$Name: copy + drop + store, $Value: key>(
    $object: &UID,
    $name: $Name,
): bool {
    let object = $object;
    let name = $name;
    let key = Wrapper { name };
    if (!field::exists_with_type<Wrapper<$Name>, ID>(object, key)) return false;
    let (field, value_id) = field::field_info<Wrapper<$Name>>(object, key);
    field::has_child_object_with_ty<$Value>(field.to_address(), value_id)
}
