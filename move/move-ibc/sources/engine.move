/// This engine module dispatches calls.
module ibc::engine {
    use aptos_framework::dispatchable_fungible_asset;
    use ibc::dispatcher;
    use std::string;
    /// The dispatch call knows both storage and indirectly the callback, thus the separate module.
    public entry fun dispatch<T: store, P: store>(data: P) {
        let metadata = dispatcher::insert<T, P>(data);
        dispatchable_fungible_asset::derived_balance(metadata);
    }
}
