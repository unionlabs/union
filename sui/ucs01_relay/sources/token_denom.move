// /*
// /// Module: ucs01_relay
// module ucs01_relay::ucs01_relay;
// */
module ucs01::fungible_token{
    //import library
    use sui::table::{Self, Table};
    use std::option;
    use sui::coin::{Self, Coin, TreasuryCap, CoinMetadata};
    use sui::transfer;
    use sui::tx_context::{Self, TxContext};
    use std::string;
    use std::ascii;
    

    // one time witness
    public struct FUNGIBLE_TOKEN has drop {}


    //init function, first args is one-time-witness(OTW)
    fun init(witness: FUNGIBLE_TOKEN, /*decimals: u8,*/ ctx: &mut TxContext){
        let (treasury_cap, metadata) = coin::create_currency<FUNGIBLE_TOKEN>(
            witness,
            2, // TODO: Decimals should be hardcoded. Its a problem..
            b"",
            b"",
            b"",
            option::none(),
            ctx);

        transfer::public_share_object(metadata);
        transfer::public_transfer(treasury_cap, tx_context::sender(ctx))
        // let treasury_cap_object_id = transfer::receiving_object_id(treasury_cap);
    }


    // mint the fingible tokens.
    public entry fun mint(treasury_cap: &mut TreasuryCap<FUNGIBLE_TOKEN>, amount: u64, recipient: address, ctx: &mut TxContext){
        //call sui framework coin module to mint this fungible token
        //coin <FUNGIBLE_TOKEN> represent tokens we publish.
        coin::mint_and_transfer(treasury_cap, amount, recipient, ctx)
    }
    
    //burn the fungible tokens.
    public entry fun burn (treasury_cap: &mut TreasuryCap<FUNGIBLE_TOKEN>, coin: Coin<FUNGIBLE_TOKEN>){
        //call sui framework coin module to burn this fungible token
        //coin <FUNGIBLE_TOKEN> represent tokens we publish.
        coin::burn(treasury_cap, coin);
    }

    // Transfer a specific amount of FUNGIBLE_TOKEN from one account to another
    public entry fun transfer_with_split(
        from: &mut Coin<FUNGIBLE_TOKEN>,
        to: address,
        amount: u64,
        ctx: &mut TxContext,
    ) {
        let transferred_coin = coin::split(from, amount, ctx);
        transfer::public_transfer(transferred_coin, to);
    }    // Transfer a specific amount of FUNGIBLE_TOKEN from one account to another

    //join two coin object to one.
    public entry fun join(self: &mut Coin<FUNGIBLE_TOKEN>, coin: Coin<FUNGIBLE_TOKEN>){
        coin::join(self, coin);
    }
    // // split one coin object to two 
    // public entry fun split (self: &mut Coin<FUNGIBLE_TOKEN>, amount: u64, recipient: address, ctx: &mut TxContext){
    //     let new_coin_object = coin::split(self, amount, ctx);

    //     // coin::split is not an rntry function, it has return object type Coin<T>, so in this function need to transfer return object to owner.
    //     transfer::public_transfer(new_coin_object, recipient);
    // }

    public entry fun update_with_metadata(
        treasury_cap: &mut TreasuryCap<FUNGIBLE_TOKEN>,
        metadata: &mut CoinMetadata<FUNGIBLE_TOKEN>,
        mut name: option::Option<string::String>,
        mut symbol: option::Option<ascii::String>,
        mut description: option::Option<string::String>,
        mut icon_uri: option::Option<ascii::String>,
    ) {
        if(name.is_some()){
            coin::update_name(treasury_cap, metadata, option::extract(&mut name));
        };
        if(symbol.is_some()){
            coin::update_symbol(treasury_cap, metadata, option::extract(&mut symbol));
        };
        if(description.is_some()){
            coin::update_description(treasury_cap, metadata, option::extract(&mut description));
        };
        if(icon_uri.is_some()){
            coin::update_icon_url(treasury_cap, metadata, option::extract(&mut icon_uri));
        };
    }

    // Transfer the entire balance of FUNGIBLE_TOKEN from one account to another
    public entry fun transfer(
        from: Coin<FUNGIBLE_TOKEN>,
        to: address,
        ctx: &mut TxContext,
    ) {
        // Transfer the 'from' coin to the 'to' address
        transfer::public_transfer(from, to);
    }

}
