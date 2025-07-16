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
  class="md:hidden bg-zinc-950 min-h-[calc(100vh-3.5rem)] flex flex-col"
  transition:slide={{ duration: 200 }}
>
  <ProfileCard />
  <div class="flex-1 overflow-y-auto flex flex-col">
    <Navigation class="flex-1" />
  </div>
</div>
