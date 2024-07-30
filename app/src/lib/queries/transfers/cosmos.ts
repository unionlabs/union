/**
 * these transfer queries are hitting chain endpoints directly
 */

// TODO: add support for sender and sender && recipient queries
export async function cosmosChainAddressTransfers({
  url,
  address,
  include = ["recipient"]
}: {
  url: string
  address: string
  include?: ["recipient"]
}) {
  const searchParams = include.map(queryKey =>
    new URLSearchParams({ query: `transfer.${queryKey}='${address}'` }).toString()
  )

  const responses = await Promise.all(
    searchParams.map(
      async searchParam => await fetch(`${url}/cosmos/tx/v1beta1/txs?${searchParam}`)
    )
  )

  if (responses.some(response => !response.ok)) {
    throw new Error(`failed to fetch from ${url}/cosmos/tx/v1beta1/txs?${searchParams.join("&")}`)
  }

  const jsons = (await Promise.all(
    responses.map(async response => await response.json())
  )) as Array<CosmosChainAddressTransfers>

  // manually set to 1 for. TODO: add support for multiple include queries
  if (include.length === 1) return jsons.at(0)
}

export interface CosmosChainAddressTransfers {
  txs: Array<CosmosChainAddressTransaction>
  tx_responses: Array<CosmosChainAddressTransactionResponse>
  pagination: any
  total: string
}

export interface CosmosChainAddressTransaction {
  body: {
    messages: Array<{
      "@type": string
      from_address: string
      to_address: string
      amount: Array<{
        denom: string
        amount: string
      }>
    }>
    memo: string
    timeout_height: string
    extension_options: Array<any>
    non_critical_extension_options: Array<any>
  }
  auth_info: {
    signer_infos: Array<{
      public_key: {
        "@type": string
        key: string
      }
      mode_info: {
        single: {
          mode: string
        }
      }
      sequence: string
    }>
    fee: {
      amount: Array<{
        denom: string
        amount: string
      }>
      gas_limit: string
      payer: string
      granter: string
    }
    tip: any
  }
  signatures: Array<string>
}

export interface CosmosChainAddressTransactionResponse {
  height: string
  txhash: string
  codespace: string
  code: number
  data: string
  raw_log: string
  logs: Array<any>
  info: string
  gas_wanted: string
  gas_used: string
  tx: {
    "@type": string
    body: {
      messages: Array<{
        "@type": string
        from_address: string
        to_address: string
        amount: Array<{
          denom: string
          amount: string
        }>
      }>
      memo: string
      timeout_height: string
      extension_options: Array<any>
      non_critical_extension_options: Array<any>
    }
    auth_info: {
      signer_infos: Array<{
        public_key: {
          "@type": string
          key: string
        }
        mode_info: {
          single: {
            mode: string
          }
        }
        sequence: string
      }>
      fee: {
        amount: Array<{
          denom: string
          amount: string
        }>
        gas_limit: string
        payer: string
        granter: string
      }
      tip: any
    }
    signatures: Array<string>
  }
  timestamp: string
  events: Array<{
    type: string
    attributes: Array<{
      key: string
      value: string
      index: boolean
    }>
  }>
}
