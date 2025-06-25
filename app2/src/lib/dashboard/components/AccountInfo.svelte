<script lang="ts">
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { Option } from "effect"
import DeleteAccountModal from "./DeleteAccountModal.svelte"

let showDeleteModal = $state(false)

function handleDelete() {
  showDeleteModal = true
}

function handleCloseModal() {
  showDeleteModal = false
}
</script>

{#if Option.isSome(dashboard.user)}
  <div class="text-center text-zinc-400 pb-12 text-xs flex flex-col">
    <h2 class="text-zinc-200 text-sm">Account Info:</h2>
    <p><span class="text-zinc-300">Email:</span> {dashboard.user.value.email}</p>
    <p><span class="text-zinc-300">ID:</span> {dashboard.user.value.id}</p>
    <button
      onclick={handleDelete}
      class="hover:text-rose-500 text-rose-400 text-xs font-medium mt-3"
    >
      Delete Account
    </button>
  </div>
{/if}

<DeleteAccountModal
  isOpen={showDeleteModal}
  onClose={handleCloseModal}
/>
