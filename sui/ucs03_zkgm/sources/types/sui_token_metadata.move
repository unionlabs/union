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

module zkgm::sui_token_metadata {
    use std::string::{Self, String};

    use sui::bcs;

    public struct SuiTokenMetadata has copy, drop {
        name: String,
        symbol: String,
        decimals: u8,
        owner: address,
        icon_url: Option<String>,
        description: String,
    }

    public(package) fun new(
        name: String,
        symbol: String,
        decimals: u8,
        owner: address,
        icon_url: Option<String>,
        description: String,
    ): SuiTokenMetadata {
        SuiTokenMetadata {
            name,
            symbol,
            decimals,
            owner,
            icon_url,
            description,
        }
    }

    public(package) fun decode(
        buf: vector<u8>,
    ): SuiTokenMetadata {
        let mut b = bcs::new(buf);
        new(
            string::utf8(b.peel_vec_u8()),
            string::utf8(b.peel_vec_u8()),
            b.peel_u8(),
            b.peel_address(),
            b.peel_option!(|b| string::utf8(b.peel_vec_u8())),
            string::utf8(b.peel_vec_u8()),
        )
    }

    public(package) fun name(m: &SuiTokenMetadata): &String {
        &m.name
    }

    public(package) fun symbol(m: &SuiTokenMetadata): &String {
        &m.symbol
    }

    public(package) fun decimals(m: &SuiTokenMetadata): u8 {
        m.decimals
    }

    public(package) fun owner(m: &SuiTokenMetadata): address {
        m.owner
    }

    public(package) fun icon_url(m: &SuiTokenMetadata): &Option<String> {
        &m.icon_url
    }

    public(package) fun description(m: &SuiTokenMetadata): &String {
        &m.description
    }
    #[test]
    fun test_decode_with_icon_some() {
        let name = string::utf8(b"Token");
        let symbol = string::utf8(b"TKN");
        let decimals: u8 = 9;
        let owner: address = @0xA11CE;
        let icon = string::utf8(b"https://icon");
        let description = string::utf8(b"desc");

        let mut buf: vector<u8> = vector::empty();
        buf.append(bcs::to_bytes(&name.into_bytes()));
        buf.append(bcs::to_bytes(&symbol.into_bytes()));
        buf.append(bcs::to_bytes(&decimals));
        buf.append(bcs::to_bytes(&owner));
        buf.append(bcs::to_bytes(&option::some(icon.into_bytes())));
        buf.append(bcs::to_bytes(&description.into_bytes()));

        let m = decode(buf);
        assert!(*name(&m) == string::utf8(b"Token"), 1);
        assert!(*symbol(&m) == string::utf8(b"TKN"), 2);
        assert!(decimals(&m) == 9, 3);
        assert!(owner(&m) == @0xA11CE, 4);
        let iu = icon_url(&m);
        assert!(option::is_some(iu), 5);
        let iu_ref = option::borrow(iu);
        assert!(*iu_ref == string::utf8(b"https://icon"), 6);
        assert!(*description(&m) == string::utf8(b"desc"), 7);
    }

    #[test]
    fun test_decode_with_icon_none() {
        let name = string::utf8(b"N");
        let symbol = string::utf8(b"S");
        let decimals: u8 = 6;
        let owner: address = @0xB0B;
        let description = string::utf8(b"D");

        let mut buf: vector<u8> = vector::empty();
        buf.append(bcs::to_bytes(&name.into_bytes()));
        buf.append(bcs::to_bytes(&symbol.into_bytes()));
        buf.append(bcs::to_bytes(&decimals));
        buf.append(bcs::to_bytes(&owner));
        buf.append(bcs::to_bytes(&option::none<vector<u8>>()));
        buf.append(bcs::to_bytes(&description.into_bytes()));

        let m = decode(buf);
        assert!(*name(&m) == string::utf8(b"N"), 1);
        assert!(*symbol(&m) == string::utf8(b"S"), 2);
        assert!(decimals(&m) == 6, 3);
        assert!(owner(&m) == @0xB0B, 4);
        assert!(option::is_none(icon_url(&m)), 5);
        assert!(*description(&m) == string::utf8(b"D"), 6);
    }

    #[test]
    fun test_new_and_getters() {
        let m = new(
            string::utf8(b"Alpha"),
            string::utf8(b"ALP"),
            8,
            @0xC0FFEE,
            option::some(string::utf8(b"https://a")),
            string::utf8(b"zzz"),
        );
        assert!(*name(&m) == string::utf8(b"Alpha"), 1);
        assert!(*symbol(&m) == string::utf8(b"ALP"), 2);
        assert!(decimals(&m) == 8, 3);
        assert!(owner(&m) == @0xC0FFEE, 4);
        let iu = icon_url(&m);
        assert!(option::is_some(iu), 5);
        let iu_ref = option::borrow(iu);
        assert!(*iu_ref == string::utf8(b"https://a"), 6);
        assert!(*description(&m) == string::utf8(b"zzz"), 7);
    }

}
