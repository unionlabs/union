<script lang="ts">
import ProfileCard from "$lib/dashboard/components/SideCard.svelte"
import { slide } from "svelte/transition"
import Navigation from "../Navigation.svelte"

interface Props {
  onclose: () => void
}

const { onclose }: Props = $props()
</script>

<div
  class="md:hidden border-b border-zinc-900 bg-zinc-950"
  transition:slide={{ duration: 200 }}
>
  <ProfileCard />
  <Navigation />
</div>
