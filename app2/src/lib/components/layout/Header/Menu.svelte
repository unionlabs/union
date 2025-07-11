<script lang="ts">
    import { beforeNavigate } from "$app/navigation"
    import SpinnerIcon from "$lib/components/icons/SpinnerIcon.svelte"
    import Skeleton from "$lib/components/ui/Skeleton.svelte"
    import { dashboard } from "$lib/dashboard/stores/user.svelte"
    import { uiStore } from "$lib/stores/ui.svelte"
    import { Option } from "effect"
    import { onDestroy, onMount } from "svelte"
    import { slide } from "svelte/transition"
    import { navigation } from "../Sidebar/navigation"
 import ProfileCard from "$lib/dashboard/components/SideCard.svelte"
    
    interface Props {
      onclose: () => void
    }
    
    const { onclose }: Props = $props()
    
    let isNewUser = $state(false)
    let intervalId: ReturnType<typeof setInterval>
    let expandedSections = $state<Set<string>>(new Set())
    
    function toggleSection(itemTitle: string) {
      if (expandedSections.has(itemTitle)) {
        expandedSections.delete(itemTitle)
      } else {
        expandedSections.add(itemTitle)
      }
      expandedSections = new Set(expandedSections) // Trigger reactivity
    }
    
    function checkIfNewUser() {
      Option.match(dashboard.user, {
        onNone: () => {
          isNewUser = false
        },
        onSome: (user) => {
          const createdAt = new Date(user.created_at).getTime()
          const oneHourAgo = Date.now() - (60 * 10 * 1000)
          isNewUser = createdAt > oneHourAgo
        },
      })
    }
    
    $effect(() => {
      if (Option.isSome(dashboard.user)) {
        checkIfNewUser()
      }
    })
    
    onMount(() => {
      checkIfNewUser()
      intervalId = setInterval(checkIfNewUser, 60 * 1000)
    })
    
    onDestroy(() => {
      if (intervalId) {
        clearInterval(intervalId)
      }
    })
    
    beforeNavigate(onclose)
    </script>
    
    <div
      class="md:hidden border-b border-zinc-900 bg-zinc-950"
      transition:slide={{ duration: 200 }}
    >
      <!-- Profile Section -->
       <ProfileCard />
    
      <!-- Navigation Section -->
      <nav class="px-2 py-4 space-y-1">
        {#each navigation as section}
          {#if section.title !== "Developer" || uiStore.showDeveloperPages}
            {#if section.title && section.title !== "More Union"}
              <div class="px-3 py-2 text-xs font-semibold text-zinc-400 uppercase tracking-wide">
                {section.title}
              </div>
            {/if}
    
            {#each section.items as item}
              {#if item.subroutes}
                <!-- Expandable item with subroutes -->
                <button
                  class="flex items-center gap-3 px-3 py-2 text-sm text-zinc-300 hover:text-white hover:bg-zinc-800 rounded-md transition-colors w-full text-left"
                  onclick={() => toggleSection(item.title)}
                >
                  <item.icon class="w-5 h-5 flex-shrink-0" />
                  <span class="flex-1">{item.title}</span>
                  <!-- Chevron icon -->
                  <svg
                    class="w-4 h-4 transition-transform duration-200 {expandedSections.has(item.title) ? 'rotate-180' : ''}"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M19 9l-7 7-7-7"
                    />
                  </svg>
                </button>
    
                {#if expandedSections.has(item.title)}
                  <div
                    class="ml-8 space-y-1 overflow-hidden"
                    transition:slide={{ duration: 200 }}
                  >
                    {#each item.subroutes as subroute}
                      {#if !subroute.editions || subroute.editions.includes(uiStore.edition)}
                        <a
                          href={subroute.path}
                          class="flex items-center gap-2 px-3 py-1.5 text-sm text-zinc-400 hover:text-zinc-300 hover:bg-zinc-800/50 rounded-md transition-colors"
                          onclick={onclose}
                        >
                          <span>{subroute.title}</span>
                          {#if subroute.new}
                            <span
                              class="inline-flex items-center px-1.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800"
                            >
                              New
                            </span>
                          {/if}
                        </a>
                      {/if}
                    {/each}
                  </div>
                {/if}
              {:else}
                <!-- Regular navigation item -->
                <a
                  href={item.path}
                  target={item.external ? "_blank" : undefined}
                  rel={item.external ? "noopener noreferrer" : undefined}
                  class="flex items-center gap-3 px-3 py-2 text-sm text-zinc-300 hover:text-white hover:bg-zinc-800 rounded-md transition-colors"
                  onclick={onclose}
                >
                  <item.icon class="w-5 h-5 flex-shrink-0" />
                  <span>{item.title}</span>
                  {#if item.external}
                    <svg
                      class="w-3 h-3 ml-auto"
                      fill="currentColor"
                      viewBox="0 0 20 20"
                    >
                      <path
                        fill-rule="evenodd"
                        d="M4.25 5.5a.75.75 0 00-.75.75v8.5c0 .414.336.75.75.75h8.5a.75.75 0 00.75-.75v-4a.75.75 0 011.5 0v4A2.25 2.25 0 0112.75 17h-8.5A2.25 2.25 0 012 14.75v-8.5A2.25 2.25 0 014.25 4h5a.75.75 0 010 1.5h-5z"
                        clip-rule="evenodd"
                      />
                      <path
                        fill-rule="evenodd"
                        d="M6.194 12.753a.75.75 0 001.06.053L16.5 4.44v2.81a.75.75 0 001.5 0v-4.5a.75.75 0 00-.75-.75h-4.5a.75.75 0 000 1.5h2.553l-9.056 8.194a.75.75 0 00-.053 1.06z"
                        clip-rule="evenodd"
                      />
                    </svg>
                  {/if}
                </a>
              {/if}
            {/each}
          {/if}
        {/each}
      </nav>
    </div>
    