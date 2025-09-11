<script lang="ts">
import { settingsStore } from "$lib/stores/settings.svelte"
import { uiStore } from "$lib/stores/ui.svelte"
import Button from "./ui/Button.svelte"
import Modal from "./ui/Modal.svelte"
import Switch from "./ui/Switch.svelte"

type Props = {
  isOpen: boolean
  onClose: () => void
}

const { isOpen, onClose }: Props = $props()

let tempPageLimit = $state(settingsStore.pageLimit)
let tempShowQuoteTokens = $state(settingsStore.showQuoteTokens)
let tempShowDeveloperChainDetails = $state(settingsStore.showDeveloperChainDetails)
let tempMainnetOnly = $state(settingsStore.mainnetOnly)
let tempShowZeroBalances = $state(uiStore.showZeroBalances)
let tempShowDeveloperPages = $state(uiStore.showDeveloperPages)
let tempGraphqlEndpoint = $state(uiStore.graphqlEndpoint)
let tempFilterWhitelist = $state(uiStore.filterWhitelist)

function handleSave() {
  settingsStore.pageLimit = tempPageLimit
  settingsStore.showQuoteTokens = tempShowQuoteTokens
  settingsStore.showDeveloperChainDetails = tempShowDeveloperChainDetails
  settingsStore.mainnetOnly = tempMainnetOnly
  uiStore.showZeroBalances = tempShowZeroBalances
  uiStore.showDeveloperPages = tempShowDeveloperPages
  uiStore.graphqlEndpoint = tempGraphqlEndpoint
  uiStore.filterWhitelist = tempFilterWhitelist
  onClose()
}
</script>

<Modal
  {isOpen}
  {onClose}
  class="w-md"
>
  <h2 class="text-xl font-bold mb-4">Settings</h2>

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

    <div class="flex justify-start gap-2 pt-4">
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
  </div>
</Modal>
