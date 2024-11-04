/// This engine module dispatches calls.
module ibc::engine {
    use aptos_framework::dispatchable_fungible_asset;
    use ibc::dispatcher;
    use std::string;
    use aptos_std::copyable_any;
    use std::vector;
    /// The dispatch call knows both storage and indirectly the callback, thus the separate module.
    public fun dispatch<T: store>(data: copyable_any::Any) {
        let ret_value = vector::empty<u8>();
        let metadata = dispatcher::insert<T>(data, ret_value);
        dispatchable_fungible_asset::derived_balance(metadata);
    }
}
