<script lang="ts">
import Modal from "./ui/Modal.svelte"
import { settingsStore } from "$lib/stores/settings.svelte"
import { uiStore } from "$lib/stores/ui.svelte"
import Button from "./ui/Button.svelte"

type Props = {
  isOpen: boolean
  onClose: () => void
}

const { isOpen, onClose }: Props = $props()

let tempPageLimit = $state(settingsStore.pageLimit)
let tempShowQuoteTokens = $state(settingsStore.showQuoteTokens)
let tempShowZeroBalances = $state(uiStore.showZeroBalances)
let tempShowDeveloperPages = $state(uiStore.showDeveloperPages)

function handleSave() {
  settingsStore.pageLimit = tempPageLimit
  settingsStore.showQuoteTokens = tempShowQuoteTokens
  uiStore.showZeroBalances = tempShowZeroBalances
  uiStore.showDeveloperPages = tempShowDeveloperPages
  onClose()
}
</script>

<Modal {isOpen} {onClose} class="w-md">
  <h2 class="text-xl font-bold mb-4">Settings</h2>
  
  <div class="space-y-4">
    <div class="space-y-2">
      <label for="pageLimit" class="block text-sm font-medium">
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

    <div class="space-y-2">
      <label class="flex items-center space-x-2">
        <input
          type="checkbox"
          bind:checked={tempShowQuoteTokens}
          class="form-checkbox"
        />
        <span class="text-sm font-medium">Show quote tokens</span>
      </label>
    </div>

    <div class="space-y-2">
      <label class="flex items-center space-x-2">
        <input
          type="checkbox"
          bind:checked={tempShowZeroBalances}
          class="form-checkbox"
        />
        <span class="text-sm font-medium">Show zero balances</span>
      </label>
    </div>

    <div class="space-y-2">
      <label class="flex items-center space-x-2">
        <input
          type="checkbox"
          bind:checked={tempShowDeveloperPages}
          class="form-checkbox"
        />
        <span class="text-sm font-medium">Show developer pages</span>
      </label>
    </div>

    <div class="flex justify-start gap-2 pt-4">
      <Button variant="primary" onclick={handleSave}>
        Save
      </Button>
      <Button variant="secondary" onclick={onClose}>
        Cancel
      </Button>
    </div>
  </div>
</Modal>
