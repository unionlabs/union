/// This engine module dispatches calls.
module zkgm::engine_zkgm {
    use aptos_framework::dispatchable_fungible_asset;
    use zkgm::dispatcher_zkgm;
    use aptos_std::copyable_any;
    use std::vector;

    /// The dispatch call knows both storage and indirectly the callback, thus the separate module.
    public fun dispatch(data: copyable_any::Any, type_info_addr: address) {
        let ret_value = vector::empty<u8>();
        let metadata = dispatcher_zkgm::insert(data, ret_value, type_info_addr);
        dispatchable_fungible_asset::derived_balance(metadata);
    }
}
