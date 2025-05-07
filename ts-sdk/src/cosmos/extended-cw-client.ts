import { CosmWasmClient, type HttpEndpoint } from "@cosmjs/cosmwasm-stargate"
import { Tendermint34Client } from "@cosmjs/tendermint-rpc"
import axios from "axios"

export class ExtendedCosmWasmClient extends CosmWasmClient {
  private restUrl!: string

  static async connectWithHeightSupport(endpoint: HttpEndpoint | string, restEndpoint: string) {
    const tmClient = await Tendermint34Client.connect(endpoint)
    const client = new ExtendedCosmWasmClient(tmClient)

    client.restUrl = restEndpoint.replace(/\/+$/, "")
    return client
  }

  async queryContractSmartAtHeight(
    contract: string,
    queryMsg: Record<string, unknown>,
    height: number,
  ) {
    const base = this.restUrl
    const encoded = btoa(JSON.stringify(queryMsg))
    const url = `${base}/cosmwasm/wasm/v1/contract/${contract}/smart/${encoded}`

    const resp = await axios.get(url, {
      headers: {
        "Content-Type": "application/json",
        "x-cosmos-block-height": height.toString(),
      },
    })
    if (resp.status < 200 || resp.status >= 300) {
      throw new Error(`HTTP ${resp.status}: ${JSON.stringify(resp.data)}`)
    }

    return resp.data
  }

  async getBalanceAtHeight(address: string, denom: string, height: number) {
    const base = this.restUrl
    const url = `${base}/cosmos/bank/v1beta1/balances/${address}`
    const resp = await fetch(url, {
      headers: {
        "Content-Type": "application/json",
        "x-cosmos-block-height": height.toString(),
      },
    })

    if (!resp.ok) {
      throw new Error(`HTTP ${resp.status}: ${await resp.text()}`)
    }

    const data = await resp.json()

    const balance = data.balances.find((b: { denom: string }) => b.denom === denom)
    if (balance) {
      return BigInt(balance.amount)
    } else {
      return null // No balance found for this denom
    }
  }
}
