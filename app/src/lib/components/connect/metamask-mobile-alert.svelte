<script lang="ts">
import * as AlertDialog from "$lib/components/ui/alert-dialog/index.ts"
import SquareArrowOutUpRightIcon from "virtual:icons/lucide/square-arrow-out-up-right"

export let metamaskAlertDialogOpen: boolean
</script>

<AlertDialog.Root bind:open={metamaskAlertDialogOpen} closeOnOutsideClick={true}>
  <AlertDialog.Content class="antialiased text-pretty p-4">
    <AlertDialog.Header>
      <AlertDialog.Title class="text-md sm:text-lg">MetaMask Mobile app users</AlertDialog.Title>
      <AlertDialog.Description class="text-sm sm:text-md text-black">
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

        <p class="mt-3">Settings {"->"} Advanced {"->"} Toggle "Advanced Transactions" off</p>
      </AlertDialog.Description>
    </AlertDialog.Header>
    <AlertDialog.Footer class='antialiased'>
      <AlertDialog.Cancel class='w-full sm:w-1/2 tracking-wide'>I've disabled smart transactions</AlertDialog.Cancel>
      <AlertDialog.Action class="bg-card-foreground tracking-wide w-full sm:w-1/2">
        <a
          href="https://i.imgur.com/Rs24oAe.mp4"
          target="_blank"
          rel="noopener noreferrer"
          class="text-center"
        >
          Video Walkthrough <SquareArrowOutUpRightIcon class="inline-block ml-2 size-4" />
        </a>
      </AlertDialog.Action>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
