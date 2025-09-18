// License text copyright (c) 2020 MariaDB Corporation Ab, All Rights Reserved.
// "Business Source License" is a trademark of MariaDB Corporation Ab.

// Parameters

// Licensor:             Union.fi, Labs Inc.
// Licensed Work:        All files under https://github.com/unionlabs/union's sui subdirectory                      
//                       The Licensed Work is (c) 2024 Union.fi, Labs Inc.
// Change Date:          Four years from the date the Licensed Work is published.
// Change License:       Apache-2.0
// 

// For information about alternative licensing arrangements for the Licensed Work,
// please contact info@union.build.

// Notice

// Business Source License 1.1

// Terms

// The Licensor hereby grants you the right to copy, modify, create derivative
// works, redistribute, and make non-production use of the Licensed Work. The
// Licensor may make an Additional Use Grant, above, permitting limited production use.

// Effective on the Change Date, or the fourth anniversary of the first publicly
// available distribution of a specific version of the Licensed Work under this
// License, whichever comes first, the Licensor hereby grants you rights under
// the terms of the Change License, and the rights granted in the paragraph
// above terminate.

// If your use of the Licensed Work does not comply with the requirements
// currently in effect as described in this License, you must purchase a
// commercial license from the Licensor, its affiliated entities, or authorized
// resellers, or you must refrain from using the Licensed Work.

// All copies of the original and modified Licensed Work, and derivative works
// of the Licensed Work, are subject to this License. This License applies
// separately for each version of the Licensed Work and the Change Date may vary
// for each version of the Licensed Work released by Licensor.

// You must conspicuously display this License on each original or modified copy
// of the Licensed Work. If you receive the Licensed Work in original or
// modified form from a third party, the terms and conditions set forth in this
// License apply to your use of that work.

// Any use of the Licensed Work in violation of this License will automatically
// terminate your rights under this License for the current and all other
// versions of the Licensed Work.

// This License does not grant you any right in any trademark or logo of
// Licensor or its affiliates (provided that you may use a trademark or logo of
// Licensor as expressly required by this License).

// TO THE EXTENT PERMITTED BY APPLICABLE LAW, THE LICENSED WORK IS PROVIDED ON
// AN "AS IS" BASIS. LICENSOR HEREBY DISCLAIMS ALL WARRANTIES AND CONDITIONS,
// EXPRESS OR IMPLIED, INCLUDING (WITHOUT LIMITATION) WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, NON-INFRINGEMENT, AND
// TITLE.

module vault::vault {
    use sui::bcs;
    use sui::table::{Self, Table};
    use sui::object_bag::{Self, ObjectBag};
    use sui::coin::{Self, Coin, TreasuryCap, CoinMetadata};
    use sui::balance;
    use std::string::{Self, String};
    use std::type_name;

    const E_UNAUTHORIZED: u64 = 1;
    const E_INVALID_PACKET_HASH: u64 = 2;
    const E_INTENT_WHITELISTED_ONLY: u64 = 3;
    const E_BASE_AMOUNT_MUST_COVER_QUOTE_AMOUNT: u64 = 4;
    const E_NO_TREASURY_CAPABILITY: u64 = 5;
    const E_INVALID_QUOTE_TOKEN: u64 = 6;
    const E_ONLY_MAKER: u64 = 0xdeadc0de;

    public struct Vault has key {
        id: UID,
        token_type_to_treasury: ObjectBag,
        intent_whitelists: Table<IntentWhitelistKey, bool>,
        fungible_counterparties: Table<FungibleLane, FungibleCounterparty>,
    }

    public struct IntentWhitelistKey has copy, drop, store {
        token: vector<u8>,
        packet_hash: vector<u8>,
    }

    public struct FungibleCounterparty has copy, drop, store {
        beneficiary: vector<u8>
    }

    public struct FungibleLane has copy, drop, store {
        token: vector<u8>,
        path: u256,
        channel: u32,
        base_token: vector<u8>,
    }

    public struct TreasuryCapWithMetadata<phantom T> has key, store {
        id: UID,
        cap: TreasuryCap<T>,
        name: String,
        symbol: String,
        decimals: u8,
        icon_url: Option<String>,
        description: String,
        owner: address
    }

    
    fun init(ctx: &mut TxContext) {
        transfer::share_object(Vault {
            id: object::new(ctx),
            token_type_to_treasury: object_bag::new(ctx),
            intent_whitelists: table::new(ctx),
            fungible_counterparties: table::new(ctx),
        });
    }

    public fun register_capability<T>(
        vault: &mut Vault,
        mut capability: TreasuryCap<T>,
        metadata: &CoinMetadata<T>,
        owner: address,
        ctx: &mut TxContext,
    ) {
        if (owner != @0x0) {
            assert!(ctx.sender() == owner, E_UNAUTHORIZED);
        };
    
        let supply = coin::supply(&mut capability);
        if (balance::supply_value(supply) != 0 ) {
            abort 0
        };

        let typename_t = type_name::get<T>();
        vault.token_type_to_treasury.add(string::from_ascii(typename_t.into_string()), TreasuryCapWithMetadata {
            id: object::new(ctx),
            cap: capability,
            name: coin::get_name(metadata),
            symbol: string::from_ascii(coin::get_symbol(metadata)),
            decimals: coin::get_decimals(metadata),
            icon_url: coin::get_icon_url(metadata).map!(|url| string::utf8(url.inner_url().into_bytes())),
            description: coin::get_description(metadata),
            owner
        });
    }

    public fun set_fungible_counterparty<T>(
        vault: &mut Vault,
        path: u256,
        channel: u32,
        base_token: vector<u8>,
        beneficiary: vector<u8>,
        ctx: &mut TxContext,
    ) {
        let token = type_name::get<T>().into_string();
        let cap: &TreasuryCapWithMetadata<T> = vault.token_type_to_treasury.borrow(token);

        assert!(cap.owner == ctx.sender(), E_UNAUTHORIZED);

        vault.fungible_counterparties.add(
            FungibleLane {
                token: token.into_bytes(),
                path,
                channel,
                base_token
            },
            FungibleCounterparty {
                beneficiary
            }
        );
    }

    public fun whitelist_intent<T>(
        vault: &mut Vault,
        packet_hashes: vector<vector<u8>>,
        whitelist: bool,
        ctx: &mut TxContext,
    ) {
        let token = type_name::get<T>().into_string();
        let cap: &TreasuryCapWithMetadata<T> = vault.token_type_to_treasury.borrow(token);

        assert!(cap.owner == ctx.sender(), E_UNAUTHORIZED);

        packet_hashes.do!(|hash| {
            assert!(hash.length() == 32, E_INVALID_PACKET_HASH);

            let whitelist_key = IntentWhitelistKey {
                token: token.into_bytes(),
                packet_hash: hash
            };

            if (vault.intent_whitelists.contains(whitelist_key)) {
                if (!whitelist) {
                    let _ = vault.intent_whitelists.remove(whitelist_key);
                };
            } else {
                vault.intent_whitelists.add(whitelist_key, true);
            };
        });
    }

    public fun solve<T>(
        vault: &mut Vault,
        packet: ibc::packet::Packet,
        base_token: vector<u8>,
        quote_token: vector<u8>,
        base_amount: u256,
        quote_amount: u256,
        receiver: vector<u8>,
        path: u256,
        relayer: address,
        _relayer_msg: vector<u8>,
        intent: bool,
        ctx: &mut TxContext,
    ): (vector<u8>, u64) {
        assert!(ctx.sender() == @zkgm, E_UNAUTHORIZED);

        if (type_name::get<T>().into_string().into_bytes() != quote_token) {
            return (vector::empty(), E_INVALID_QUOTE_TOKEN)
        };

        if (intent) {
            let packet_hash = ibc::commitment::commit_packet(&packet);
            if (!vault.intent_whitelists.contains(IntentWhitelistKey {
                token: quote_token,
                packet_hash
            })) {
                return (vector::empty(), E_INTENT_WHITELISTED_ONLY)
            };
        };

        let fungibility = FungibleLane {
            token: quote_token,
            path,
            channel: packet.destination_channel_id(),
            base_token,
        };

        if (vault.fungible_counterparties.contains(fungibility)) {
            return (vector::empty(), E_ONLY_MAKER)
        };

        let counterparty_beneficiary = vault.fungible_counterparties.borrow(fungibility).beneficiary;       

        if (quote_amount > base_amount) {
            return (vector::empty(), E_BASE_AMOUNT_MUST_COVER_QUOTE_AMOUNT)
        };

        let (base_amount, quote_amount) = (base_amount, quote_amount);

        let capability = vault.get_treasury_cap<T>();
        let fee = base_amount - quote_amount;
        if (fee > 0) {
            coin::mint_and_transfer<T>(capability, fee as u64, relayer, ctx);
        };

        if (quote_amount > 0) {
            let receiver = bcs::new(receiver).peel_address();
            coin::mint_and_transfer<T>(capability, quote_amount as u64, receiver, ctx);
        };

        (counterparty_beneficiary, 0)
    }

    public fun burn<T>(vault: &mut Vault, c: Coin<T>, ctx: &mut TxContext): u64 {
        let typename_t = type_name::get<T>();
        let cap = vault.token_type_to_treasury.borrow_mut<String, TreasuryCapWithMetadata<T>>(string::from_ascii(typename_t.into_string()));

        if (ctx.sender() != @zkgm) {
            assert!(ctx.sender() == cap.owner, E_UNAUTHORIZED);
        };
        
        coin::burn<T>(&mut cap.cap, c)
    }
    
    fun get_treasury_cap<T>(
        vault: &mut Vault
    ): &mut TreasuryCap<T> {
        let typename_t = type_name::get<T>();
        let key = string::from_ascii(typename_t.into_string());
        if (!vault.token_type_to_treasury.contains(key)) {
            abort E_NO_TREASURY_CAPABILITY             
        };
        &mut vault.token_type_to_treasury.borrow_mut<String, TreasuryCapWithMetadata<T>>(key).cap
    }
}
