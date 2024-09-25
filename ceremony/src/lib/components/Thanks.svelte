<script lang="ts">
  import { getPublicHash } from "$lib/supabase";
  import { onDestroy, onMount } from "svelte";
  import { getState } from "$lib/state/index.svelte.ts";
  import type { KeyEvent } from "$lib/state/terminal.svelte.ts";

  const { terminal } = getState();

  let hash = $state(undefined);
  let selectedButton = $state(0);

  const buttons = $state([
    { text: "Tweet your attestation", action: shareOnTwitter },
    { text: "View contributions", action: () => terminal.setTab(3) }
  ]);

  onMount(() => {
    terminal.updateHistory("Thank you!");
    terminal.updateHistory("Your contribution is complete. Thank you for securing the Union network. Tweet your cryptographic attestation for extra transparency.");
  });

  const unsubscribe = terminal.keys.subscribe((event) => {
    if (event) {
      handleKeydown(event);
    }
  });

  onDestroy(unsubscribe);

  async function getHash() {
    hash = await getPublicHash();
  }

  $effect(() => {
    getHash();
  });

  function shareOnTwitter() {
    let url = `https://ceremony.union.build/contributions/${hash}`;
    const tweetText = `I just contributed to the @union_build Trusted Setup Ceremony, to secure its ZK circuit for trustless, decentralized interoperability. \n\nI attest to my contribution. My public key hash is: \n\n${url}\n\n#JoinTheUnion`;
    const twitterIntentUrl = new URL("https://twitter.com/intent/tweet");
    twitterIntentUrl.searchParams.append("text", tweetText);
    window.open(twitterIntentUrl.toString(), "_blank");
  }

  function handleKeydown(event: KeyEvent) {
    if (event.type === "keydown") {
      switch (event.key) {
        case 'ArrowUp':
          selectedButton = (selectedButton - 1 + buttons.length) % buttons.length;
          break;
        case 'ArrowDown':
          selectedButton = (selectedButton + 1) % buttons.length;
          break;
        case 'Enter':
          triggerAction(selectedButton);
          break;
      }
    }
  }

  function triggerAction(index: number) {
    buttons[index].action();
  }
</script>

{#each buttons as btn, index}
  <button
          class="block outline-none focus:ring-2 focus:ring-transparent focus:border-none"
          class:text-union-accent-500={selectedButton === index}
          onclick={() => triggerAction(index)}
          tabindex={selectedButton === index ? 0 : -1}
  >
    &gt; {btn.text}
  </button>
{/each}