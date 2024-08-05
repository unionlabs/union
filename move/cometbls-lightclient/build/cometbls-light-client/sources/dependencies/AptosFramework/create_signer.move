/// Provides a common place for exporting `create_signer` across the Aptos Framework.
///
/// To use create_signer, add the module below, such that:
/// `friend aptos_framework::friend_wants_create_signer`
/// where `friend_wants_create_signer` is the module that needs `create_signer`.
///
/// Note, that this is only available within the Aptos Framework.
///
/// This exists to make auditing straight forward and to limit the need to depend
/// on account to have access to this.
module aptos_framework::create_signer {
    friend aptos_framework::account;
    friend aptos_framework::aptos_account;
    friend aptos_framework::coin;
    friend aptos_framework::fungible_asset;
    friend aptos_framework::genesis;
    friend aptos_framework::multisig_account;
    friend aptos_framework::object;

    public(friend) native fun create_signer(addr: address): signer;
}
