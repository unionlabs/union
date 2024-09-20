<script lang="ts">
import { AddressForm, type ValidState } from "$lib/components/address"
import H4 from "$lib/components/typography/H4.svelte"
import H2 from "$lib/components/typography/H2.svelte"
import Button from "$lib/components/Button.svelte"
import Text from "$lib/components/typography/Text.svelte"
import type { ContributorState } from "$lib/stores/state.svelte.ts"
import { insertWalletData } from "$lib/supabase"
import { toast } from "svelte-sonner"

type Props = {
  contributor: ContributorState
}

let { contributor }: Props = $props()

let addressValidState: ValidState = $state("PENDING")

const skip = async () => {
  try {
    if (!contributor.userId) return
    const result = await insertWalletData({
      id: contributor.userId,
      wallet: "SKIPPED"
    })
    if (result) {
      toast.success("Wallet address saved successfully")
      contributor.userWallet = "SKIPPED"
    } else {
      toast.error("Failed to save wallet address")
    }
  } catch (error) {
    console.error("Error saving wallet address:", error)
    toast.error("An error occurred while saving the wallet address")
  }
}
</script>


<div class="text-center flex flex-col items-center gap-4">
  <H2 class="">Add an address</H2>
  <Text class="">You may receive rewards for successful contributions.</Text>
  <AddressForm class="" onValidation={result => (addressValidState = result)} {contributor} />
  <Text class="py-8">Or</Text>
  <H4>I don't want rewards</H4>
  <Text>You can contribute without adding an address</Text>
  <Button onclick={skip} class="bg-transparent text-white hover:text-white border-2 border-white hover:bg-neutral-800">Skip
    rewards
  </Button>
</div>