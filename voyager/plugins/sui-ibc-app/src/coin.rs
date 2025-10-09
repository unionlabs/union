use move_core_types_sui::ident_str;
use serde::Serialize;
use sui_sdk::types::{
    TypeTag,
    base_types::{ObjectID, ObjectRef},
    programmable_transaction_builder::ProgrammableTransactionBuilder,
    transaction::{Argument, CallArg, Command, ObjectArg},
};

pub async fn update_name(
    ptb: &mut ProgrammableTransactionBuilder,
    treasury_ref: ObjectRef,
    metadata_ref: ObjectRef,
    coin_t: TypeTag,
    name: String,
) {
    call_coin_setter(
        ptb,
        "update_name",
        treasury_ref,
        metadata_ref,
        coin_t.clone(),
        name,
    )
    .await;
}
pub async fn update_symbol(
    ptb: &mut ProgrammableTransactionBuilder,
    treasury_ref: ObjectRef,
    metadata_ref: ObjectRef,
    coin_t: TypeTag,
    symbol: String,
) {
    call_coin_setter(
        ptb,
        "update_symbol",
        treasury_ref,
        metadata_ref,
        coin_t.clone(),
        symbol,
    )
    .await;
}
pub async fn update_icon_url(
    ptb: &mut ProgrammableTransactionBuilder,
    treasury_ref: ObjectRef,
    metadata_ref: ObjectRef,
    coin_t: TypeTag,
    icon_url: String,
) {
    call_coin_setter(
        ptb,
        "update_icon_url",
        treasury_ref,
        metadata_ref,
        coin_t.clone(),
        icon_url,
    )
    .await;
}
pub async fn update_description(
    ptb: &mut ProgrammableTransactionBuilder,
    treasury_ref: ObjectRef,
    metadata_ref: ObjectRef,
    coin_t: TypeTag,
    description: String,
) {
    call_coin_setter(
        ptb,
        "update_description",
        treasury_ref,
        metadata_ref,
        coin_t.clone(),
        description,
    )
    .await;
}
async fn call_coin_setter<T: Serialize>(
    ptb: &mut ProgrammableTransactionBuilder,
    function: &'static str,
    treasury_ref: ObjectRef,
    metadata_ref: ObjectRef,
    coin_t: TypeTag,
    data: T,
) {
    let arguments: Vec<Argument> = [
        CallArg::Object(ObjectArg::ImmOrOwnedObject(treasury_ref)),
        CallArg::Object(ObjectArg::SharedObject {
            id: metadata_ref.0,
            initial_shared_version: metadata_ref.1,
            mutable: true,
        }),
        CallArg::Pure(bcs::to_bytes(&data).unwrap()),
    ]
    .into_iter()
    .map(|arg| ptb.input(arg).unwrap())
    .collect();

    let _ = ptb.command(Command::move_call(
        ObjectID::from_single_byte(2),
        ident_str!("coin").into(),
        ident_str!(function).into(),
        vec![coin_t],
        arguments.to_vec(),
    ));
}
