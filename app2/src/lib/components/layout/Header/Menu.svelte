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
  class="md:hidden bg-zinc-950 h-[calc(100dvh-3.5rem)] overflow-hidden flex flex-col"
  transition:slide={{ duration: 200 }}
>
  <ProfileCard />
  <div class="flex-1 overflow-y-auto overflow-x-hidden">
    <Navigation class="min-h-full flex flex-col" />
  </div>
</div>
