<script lang="ts">
import {
  LST_CONFIG_LABELS,
  LST_CONFIGS,
  type LSTConfig,
  lstConfig,
} from "$lib/stake/config.svelte.ts"
import { settingsStore } from "$lib/stores/settings.svelte"
import { uiStore } from "$lib/stores/ui.svelte"
import Button from "./ui/Button.svelte"
import Modal from "./ui/Modal.svelte"
import Switch from "./ui/Switch.svelte"
import Tabs from "./ui/Tabs.svelte"

type Props = {
  isOpen: boolean
  onClose: () => void
}

const { isOpen, onClose }: Props = $props()

let activeTab = $state("general")

// General settings temp state
let tempPageLimit = $state(settingsStore.pageLimit)
let tempShowQuoteTokens = $state(settingsStore.showQuoteTokens)
let tempShowDeveloperChainDetails = $state(settingsStore.showDeveloperChainDetails)
let tempMainnetOnly = $state(settingsStore.mainnetOnly)
let tempShowZeroBalances = $state(uiStore.showZeroBalances)
let tempShowDeveloperPages = $state(uiStore.showDeveloperPages)
let tempGraphqlEndpoint = $state(uiStore.graphqlEndpoint)
let tempFilterWhitelist = $state(uiStore.filterWhitelist)

// LST config temp state
let selectedConfig = $state<LSTConfig>("mainnet")
let tempEthereumChainId = $state(lstConfig.ethereumChainId)
let tempUnionChainId = $state(lstConfig.unionChainId)
let tempSourceChannelId = $state(lstConfig.sourceChannelId)
let tempDestinationChannelId = $state(lstConfig.destinationChannelId)
let tempEvmRpcEndpoint = $state(lstConfig.evmRpcEndpoint)
let tempUnionRpcEndpoint = $state(lstConfig.unionRpcEndpoint)
let tempUcs03EvmAddress = $state(lstConfig.ucs03EvmAddress)
let tempUcs03MinterOnUnion = $state(lstConfig.ucs03MinterOnUnion)
let tempUcs03Zkgm = $state(lstConfig.ucs03Zkgm)

function handleSave() {
  // Save general settings
  settingsStore.pageLimit = tempPageLimit
  settingsStore.showQuoteTokens = tempShowQuoteTokens
  settingsStore.showDeveloperChainDetails = tempShowDeveloperChainDetails
  settingsStore.mainnetOnly = tempMainnetOnly
  uiStore.showZeroBalances = tempShowZeroBalances
  uiStore.showDeveloperPages = tempShowDeveloperPages
  uiStore.graphqlEndpoint = tempGraphqlEndpoint
  uiStore.filterWhitelist = tempFilterWhitelist

  // Save LST config
  lstConfig.ethereumChainId = tempEthereumChainId
  lstConfig.unionChainId = tempUnionChainId
  lstConfig.sourceChannelId = tempSourceChannelId
  lstConfig.destinationChannelId = tempDestinationChannelId
  lstConfig.evmRpcEndpoint = tempEvmRpcEndpoint
  lstConfig.unionRpcEndpoint = tempUnionRpcEndpoint
  lstConfig.ucs03EvmAddress = tempUcs03EvmAddress
  lstConfig.ucs03MinterOnUnion = tempUcs03MinterOnUnion
  lstConfig.ucs03Zkgm = tempUcs03Zkgm

  onClose()
}

function handleConfigChange(config: LSTConfig) {
  selectedConfig = config

  if (config === "custom") {
    // Empty all fields for custom configuration
    tempEthereumChainId = ""
    tempUnionChainId = ""
    tempSourceChannelId = 0
    tempDestinationChannelId = 0
    tempEvmRpcEndpoint = ""
    tempUnionRpcEndpoint = ""
    tempUcs03EvmAddress = "" as `0x${string}`
    tempUcs03MinterOnUnion = "" as `${string}1${string}`
    tempUcs03Zkgm = "" as `${string}1${string}`
  } else {
    // Load predefined configuration
    lstConfig.loadPredefined(config)
    tempEthereumChainId = lstConfig.ethereumChainId
    tempUnionChainId = lstConfig.unionChainId
    tempSourceChannelId = lstConfig.sourceChannelId
    tempDestinationChannelId = lstConfig.destinationChannelId
    tempEvmRpcEndpoint = lstConfig.evmRpcEndpoint
    tempUnionRpcEndpoint = lstConfig.unionRpcEndpoint
    tempUcs03EvmAddress = lstConfig.ucs03EvmAddress
    tempUcs03MinterOnUnion = lstConfig.ucs03MinterOnUnion
    tempUcs03Zkgm = lstConfig.ucs03Zkgm
  }
}
</script>

<Modal
  {isOpen}
  {onClose}
  class="w-md"
>
  <div class="mb-4 w-fit p-1 bg-zinc-900 rounded-md">
    <Tabs
      items={[
        { id: "general", label: "General" },
        { id: "lst", label: "LST" },
      ]}
      activeId={activeTab}
      onTabChange={(id) => activeTab = id}
    />
  </div>

  {#if activeTab === "general"}
    <div class="space-y-4">
      <div class="space-y-2">
        <label
          for="pageLimit"
          class="block text-sm font-medium uppercase text-zinc-500"
        >
          Items per page
        </label>
        <input
          id="pageLimit"
          type="number"
          min="1"
          max="100"
          bind:value={tempPageLimit}
          class="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-md"
        />
      </div>

      <div>
        <div class="flex w-full justify-between">
          <label
            for="graphqlEndpoint"
            class="block text-sm font-medium uppercase text-zinc-500"
          >
            GraphQL Endpoint
          </label>
          <Button
            variant="inline"
            class="text-sm font-medium"
            onclick={() => uiStore.clearGqlCache()}
          >
            Clear Cache
          </Button>
        </div>
        <input
          id="graphqlEndpoint"
          type="text"
          bind:value={tempGraphqlEndpoint}
          class="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-md"
        />
      </div>

      <div class="space-y-2">
        <Switch
          checked={tempShowQuoteTokens}
          label="Show quote tokens"
          change={(value) => tempShowQuoteTokens = value}
        />
      </div>

      <div class="space-y-2">
        <Switch
          checked={tempShowDeveloperChainDetails}
          label="Show developer chain details"
          change={(value) => tempShowDeveloperChainDetails = value}
        />
      </div>

      <div class="space-y-2">
        <Switch
          checked={tempMainnetOnly}
          label="Mainnet only (explorer)"
          change={(value) => tempMainnetOnly = value}
        />
      </div>

      <div class="space-y-2">
        <Switch
          checked={tempShowZeroBalances}
          label="Show zero balances"
          change={(value) => tempShowZeroBalances = value}
        />
      </div>

      <div class="space-y-2">
        <Switch
          checked={tempShowDeveloperPages}
          label="Show developer pages"
          change={(value) => tempShowDeveloperPages = value}
        />
      </div>

      <div class="space-y-2">
        <Switch
          checked={tempFilterWhitelist}
          label="Filter whitelist"
          change={(value) => tempFilterWhitelist = value}
        />
      </div>
    </div>
  {:else if activeTab === "lst"}
    <div class="space-y-4">
      <div class="space-y-2">
        <label
          for="lstConfig"
          class="block text-sm font-medium uppercase text-zinc-500"
        >
          Config
        </label>
        <select
          id="lstConfig"
          bind:value={selectedConfig}
          onchange={(e) => handleConfigChange(e.currentTarget.value as LSTConfig)}
          class="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-md"
        >
          {#each LST_CONFIGS as config}
            <option value={config}>{LST_CONFIG_LABELS[config]}</option>
          {/each}
        </select>
      </div>

      <div class="grid grid-cols-2 gap-2">
        <div class="space-y-2">
          <label
            for="ethereumChainId"
            class="block text-sm font-medium uppercase text-zinc-500"
          >
            EVM Chain ID
          </label>
          <input
            id="ethereumChainId"
            type="text"
            bind:value={tempEthereumChainId}
            class="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-md font-mono text-sm"
          />
        </div>

        <div class="space-y-2">
          <label
            for="unionChainId"
            class="block text-sm font-medium uppercase text-zinc-500"
          >
            Union Chain ID
          </label>
          <input
            id="unionChainId"
            type="text"
            bind:value={tempUnionChainId}
            class="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-md font-mono text-sm"
          />
        </div>
      </div>

      <div class="grid grid-cols-2 gap-2">
        <div class="space-y-2">
          <label
            for="sourceChannelId"
            class="block text-sm font-medium uppercase text-zinc-500"
          >
            Source Channel ID
          </label>
          <input
            id="sourceChannelId"
            type="number"
            bind:value={tempSourceChannelId}
            class="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-md"
          />
        </div>

        <div class="space-y-2">
          <label
            for="destinationChannelId"
            class="block text-sm font-medium uppercase text-zinc-500"
          >
            Dest Channel ID
          </label>
          <input
            id="destinationChannelId"
            type="number"
            bind:value={tempDestinationChannelId}
            class="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-md"
          />
        </div>
      </div>

      <div class="space-y-2">
        <label
          for="evmRpcEndpoint"
          class="block text-sm font-medium uppercase text-zinc-500"
        >
          EVM RPC Endpoint
        </label>
        <input
          id="evmRpcEndpoint"
          type="text"
          bind:value={tempEvmRpcEndpoint}
          class="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-md font-mono text-sm"
        />
      </div>

      <div class="space-y-2">
        <label
          for="unionRpcEndpoint"
          class="block text-sm font-medium uppercase text-zinc-500"
        >
          Union RPC Endpoint
        </label>
        <input
          id="unionRpcEndpoint"
          type="text"
          bind:value={tempUnionRpcEndpoint}
          class="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-md font-mono text-sm"
        />
      </div>

      <div class="space-y-2">
        <label
          for="ucs03EvmAddress"
          class="block text-sm font-medium uppercase text-zinc-500"
        >
          UCS03 EVM Address
        </label>
        <input
          id="ucs03EvmAddress"
          type="text"
          bind:value={tempUcs03EvmAddress}
          class="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-md font-mono text-sm"
        />
      </div>

      <div class="space-y-2">
        <label
          for="ucs03MinterOnUnion"
          class="block text-sm font-medium uppercase text-zinc-500"
        >
          UCS03 Minter on Union
        </label>
        <input
          id="ucs03MinterOnUnion"
          type="text"
          bind:value={tempUcs03MinterOnUnion}
          class="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-md font-mono text-sm"
        />
      </div>

      <div class="space-y-2">
        <label
          for="ucs03Zkgm"
          class="block text-sm font-medium uppercase text-zinc-500"
        >
          UCS03 ZKGM Address
        </label>
        <input
          id="ucs03Zkgm"
          type="text"
          bind:value={tempUcs03Zkgm}
          class="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-md font-mono text-sm"
        />
      </div>
    </div>
  {/if}

  <div class="flex justify-start gap-2 pt-4 mt-4 border-t border-zinc-700">
    <Button
      variant="primary"
      onclick={handleSave}
    >
      Save
    </Button>
    <Button
      variant="secondary"
      onclick={onClose}
    >
      Cancel
    </Button>
  </div>
</Modal>
