<script lang="ts">
import { onMount } from "svelte"
import { sleep } from "#/lib/utilities.ts"
import { gql, setContextClient } from "@urql/svelte"
import { client, getQueryStore } from "#/lib/graphql.ts"

const FETCH_INTERVAL = 2_000

let terminalElement: HTMLDivElement

const errors: Array<string> = []

type Network = "union" | "sepolia"
type Action = "fetching" | "observed event" | "sending message"
type LogLine = {
  network: Network | undefined
  action: Action | undefined
  logLine: string
}

const replayOffset = 300
let logLines: Array<LogLine> = []

const FETCH_EVENT = gql`
    query FetchEvent($id: Int!) {
      demo_txes_by_pk(id: $id) {
        data
        created_at
      }
    }
  `

const FETCH_LATEST_ID = gql`
    query GetLatestId {
      demo_txes(limit: 1, order_by: { id: desc }) {
        id
      }
    }
  `

function formatLogLine(queryResult: any): LogLine | undefined {
  if (!queryResult.data.demo_txes_by_pk) return
  const result = queryResult.data.demo_txes_by_pk.data
  let [network, action, data] = ["", "", result] as unknown as [Network, Action, any]

  if ("EthereumMinimal" in data) {
    network = "union"
    data = data["EthereumMinimal"]
  }

  if ("CometblsMinimal" in data) {
    network = "sepolia"
    data = data["CometblsMinimal"]
  }

  if ("Fetch" in data) {
    action = "fetching"
    data = data["Fetch"]["data"]
  }

  if ("Event" in data) {
    action = "observed event"
    data = data["Event"]["data"]
  }

  if ("Msg" in data) {
    action = "sending message"
    data = data["Msg"]["data"]
  }

  return { network, action, logLine: JSON.stringify(data) }
}

// https://formidable.com/open-source/urql/docs/basics/svelte/#providing-the-client
setContextClient(client)
const latestIdQuery = getQueryStore(FETCH_LATEST_ID)

async function fetchDemoTransactions(latestIdWorker: number) {
  const startHeight = latestIdWorker - replayOffset
  let index = startHeight
  for (;;) {
    index++
    await sleep(FETCH_INTERVAL)
    const response = await client.query(FETCH_EVENT, { id: index }).toPromise()
    const newLine = formatLogLine(response)
    if (newLine) logLines = [newLine, ...logLines]
    if (index > latestIdWorker - 10) index = startHeight
  }
}

onMount(() => {
  const unsubscribe = latestIdQuery.subscribe(({ data, error }) => {
    if (error) {
      errors.push(error.message)
      return
    }
    if (!data?.demo_txes) return
    fetchDemoTransactions(1_502)
  })
  return () => unsubscribe()
})
</script>

<section
  class="font-mono h-[332px] md:h-[432px] max-h-[432px] my-8 mx-auto w-6xl min-w-4xl bg-[#1e1e1e]"
>
  <div
    class="flex max-w-4xl p-4 md:shadow-2xl md:right-[16px] bg-black text-xs sm:text-sm font-jetbrains overflow-auto max-h-[432px] mx-auto flex-col-reverse"
    bind:this={terminalElement}
  >
    {#each logLines as { network, action, logLine }}
      <div class="p-0 break-words">
        <span class={network == "union" ? "text-accent" : "text-yellow-300"}>
          [{network}]
        </span>
        <span>{action}</span>
        <span class="text-gray-400 overflow-auto max-w-3xl">{logLine} </span>
      </div>
    {/each}
  </div>
</section>
