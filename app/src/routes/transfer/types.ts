export interface TransactionSearchParams {
  /**
   * CAIP-2 format chain ID: `$namespace:$blockchain_id`
   * Example: "eip155:11155111", "cosmos:union-testnet-9"
   * @see https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-2.md
   */
  from: string
  /**
   * CAIP-10 account ID (CAIP-2 format chain ID + address): `$namespace:$blockchain_id:$address`
   * Example: "eip155:11155111:0x1234", "cosmos:union-testnet-9:union..."
   * @see https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-10.md
   */
  to: string
}
