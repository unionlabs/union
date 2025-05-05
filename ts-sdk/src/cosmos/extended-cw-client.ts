import { CosmWasmClient, type HttpEndpoint } from "@cosmjs/cosmwasm-stargate"
import { Tendermint34Client } from "@cosmjs/tendermint-rpc"

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
    height: number
  ) {
    const base = this.restUrl
    const encoded = btoa(JSON.stringify(queryMsg))
    const resp = await fetch(`${base}/cosmwasm/wasm/v1/contract/${contract}/smart/${encoded}`, {
      headers: {
        "Content-Type": "application/json",
        "x-cosmos-block-height": height.toString()
      }
    })
    if (!resp.ok) {
      throw new Error(`HTTP ${resp.status}: ${await resp.text()}`)
    }
    return resp.json()
  }
}
