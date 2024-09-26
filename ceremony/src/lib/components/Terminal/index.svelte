<script lang="ts">
  import {getState} from "$lib/state/index.svelte.ts"
  import {logout, user} from "$lib/state/session.svelte.ts"
  import Contributions from "./Contributors.svelte"
  import {goto} from "$app/navigation"
  import {onDestroy} from "svelte"
  import Print from "$lib/components/Terminal/Print.svelte"
  import Activity from "$lib/components/Terminal/Activity.svelte"
  import {cn} from "$lib/utils/utils.ts"
  import Button from "$lib/components/Terminal/Button.svelte"
  import TaskBar from "$lib/components/Terminal/TaskBar.svelte"

  const { terminal, contributor } = getState()

  let {children} = $props()

  const changeTab = async (tab: number) => {
    if (tab === 4 && terminal.hash) {
      terminal.setTab(tab)
      await goto(`/0____0/${terminal.hash}`)
    } else if (tab <= 3) {
      terminal.setTab(tab)
      await goto("/")
    }
  }

  const unsubscribe = terminal.keys.subscribe(event => {
    if (event) {
      if (event.type === "keydown" && (event.shiftKey || event.ctrlKey)) {
        if (event.key === "!") {
          changeTab(1)
        } else if (event.key === "@") {
          changeTab(2)
        } else if (event.key === "#") {
          changeTab(3)
        } else if (event.key === "$") {
          changeTab(4)
        } else if (event.key === "X") {
          logout(terminal)
        }
      }
    }
  })

  onDestroy(unsubscribe)

  function autoScroll(node: HTMLElement) {
    const scroll = () => {
      node.scrollTop = node.scrollHeight
    }

    const observer = new MutationObserver(scroll)
    observer.observe(node, {childList: true, subtree: true})

    return {
      destroy() {
        observer.disconnect()
      }
    }
  }
</script>


<div class="flex flex-col w-full h-full sm:max-h-[700px] bg-black/50 backdrop-blur-lg max-w-4xl sm:border border-[#48494C] rounded-lg overflow-hidden drop-shadow-2xl shadow-black">

  <!--TOP BAR-->
  <div class="w-full flex justify-between bg-neutral-800/80 px-4 py-2">
    <div class="flex items-center gap-2">
      <svg class="size-5" viewBox="0 0 192 192" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path fill-rule="evenodd" clip-rule="evenodd"
              d="M136 128C136 163.346 107.346 192 72 192C36.6538 192 8 163.346 8 128H40C40 145.673 54.3269 160 72 160C89.6731 160 104 145.673 104 128H136Z"
              fill="#fff"/>
        <path fill-rule="evenodd" clip-rule="evenodd" d="M40 80H8V112H40V80ZM136 80H104V112H136V80Z" fill="#fff"/>
        <path fill-rule="evenodd" clip-rule="evenodd"
              d="M152 112L184 112L184 80L152 80L152 112ZM56 112L88 112L88 80L56 80L56 112Z" fill="#fff"/>
        <path fill-rule="evenodd" clip-rule="evenodd"
              d="M56 64C56 28.6538 84.6538 -8.68512e-06 120 -5.59506e-06C155.346 -2.50499e-06 184 28.6538 184 64L152 64C152 46.3269 137.673 32 120 32C102.327 32 88 46.3269 88 64L56 64Z"
              fill="#fff"/>
      </svg>

      <Button onclick={() => changeTab(1)} class={cn(terminal.tab === 1 ? "text-union-accent-500" : "text-white")}>CORE
      </Button>
      <Button onclick={() => changeTab(2)} class={cn(terminal.tab === 2 ? "text-union-accent-500" : "text-white")}>LOGS
      </Button>
      <Button onclick={() => changeTab(3)} class={cn(terminal.tab === 3 ? "text-union-accent-500" : "text-white")}>USERS
      </Button>
      {#if terminal.hash !== undefined}
        <Button onclick={() => changeTab(4)} class={cn(terminal.tab === 4 ? "text-union-accent-500" : "text-white")}>
          {#if terminal.hash}
            {terminal.hash.slice(0, 5)}...{terminal.hash.slice(-5)}
          {:else}
            {terminal.hash}
          {/if}
        </Button>
      {/if}
    </div>
    {#if user.session?.user}
      <Button class="text-white" onclick={() => logout(terminal)}>Log out</Button>
    {/if}
  </div>

  <!--TERMINAL-->
  <div class="overflow-y-scroll h-full p-4" use:autoScroll>
    <div class="text-union-accent-50 w-full">
      {#if terminal.tab === 1}
        <div class="flex flex-col">
          {#each terminal.history as text}
            <Print>{text}</Print>
          {/each}
        </div>
        <Print><br></Print>
        {@render children()}
      {:else if terminal.tab === 2}
        <Activity/>
      {:else if terminal.tab === 3}
        <Contributions/>
      {:else if terminal.tab === 4 && terminal.hash}
        {@render children()}
      {/if}
    </div>
  </div>

  <TaskBar/>

</div>



