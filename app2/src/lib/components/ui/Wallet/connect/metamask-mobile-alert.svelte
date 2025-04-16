<script lang="ts">
export let metamaskAlertDialogOpen: boolean

function closeDialog() {
  metamaskAlertDialogOpen = false
}

function handleOutsideClick(event: MouseEvent) {
  const target = event.target as HTMLElement
  if (target.classList.contains("dialog-overlay")) {
    closeDialog()
  }
}
</script>

{#if metamaskAlertDialogOpen}
  <!-- Overlay -->
  <div
          class="dialog-overlay fixed inset-0 bg-black/50 z-50 flex items-center justify-center p-4"
          onclick={handleOutsideClick}
  >
    <!-- Dialog Content -->
    <div
            class="bg-white rounded-lg shadow-lg max-w-md w-full antialiased text-pretty p-4 animate-in fade-in zoom-in-95"
            role="alertdialog"
            aria-modal="true"
            aria-labelledby="alert-dialog-title"
            aria-describedby="alert-dialog-description"
    >
      <!-- Header -->
      <div class="mb-4">
        <h2 id="alert-dialog-title" class="text-md sm:text-lg font-semibold">
          MetaMask Mobile app users
        </h2>
        <div id="alert-dialog-description" class="text-sm sm:text-md text-black mt-2">
          MetaMask <a
                tabindex="-1"
                target="_blank"
                rel="noopener noreferrer"
                class="font-bold text-blue-600 hover:underline"
                href="https://support.metamask.io/transactions-and-gas/transactions/smart-transactions/#how-do-i-manage-my-smart-transactions-settings"
        >
          "smart transactions"
        </a>
          feature is not compatible with the Union App. Make sure to disable it in metamask settings before
          attempting a transaction.

          <p class="mt-3">Settings → Advanced → Toggle "Advanced Transactions" off</p>
        </div>
      </div>

      <!-- Footer -->
      <div class="flex flex-col sm:flex-row gap-3 mt-6 antialiased">
        <button
                class="w-full sm:w-1/2 py-2 px-4 border border-gray-300 rounded-md tracking-wide hover:bg-gray-100 transition-colors"
                onclick={closeDialog}
        >
          I've disabled smart transactions
        </button>

        <a
                href="https://i.imgur.com/Rs24oAe.mp4"
                target="_blank"
                rel="noopener noreferrer"
                class="w-full sm:w-1/2 bg-black text-white py-2 px-4 rounded-md tracking-wide flex items-center justify-center hover:bg-gray-800 transition-colors"
        >
          Video Walkthrough
        </a>
      </div>
    </div>
  </div>
{/if}

<style>
  .animate-in {
    animation: fadeIn 0.2s ease-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: scale(0.95);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }
</style>