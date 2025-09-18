module vault::metadata {
    use std::string::String;

    public struct Metadata has copy, drop, store {
        name: String,
        symbol: String,
        decimals: u8,
        icon_url: Option<String>,
        description: String,
        owner: address
    }

    public fun new(
        name: String,
        symbol: String,
        decimals: u8,
        icon_url: Option<String>,
        description: String,
        owner: address   
    ): Metadata {
        Metadata {
            name,
            symbol,
            decimals,
            icon_url,
            description,
            owner,
        }
    }

    public fun name(metadata: &Metadata): &String {
        &metadata.name        
    }

    public fun symbol(metadata: &Metadata): &String {
        &metadata.symbol        
    }

    public fun decimals(metadata: &Metadata): u8 {
        metadata.decimals
    }

    public fun icon_url(metadata: &Metadata): &Option<String> {
        &metadata.icon_url        
    }

    public fun description(metadata: &Metadata): &String {
        &metadata.description        
    }

    public fun owner(metadata: &Metadata): address {
        metadata.owner
    }
}
