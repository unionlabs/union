<script lang="ts">
  import {getPublicHash} from "$lib/supabase";
  import {getState} from "$lib/state/index.svelte.ts";
  import type {KeyEvent} from "$lib/state/terminal.svelte.ts";
  import {sleep} from "$lib/utils/utils.ts";
  import {onMount} from "svelte";
  import {beforeNavigate} from "$app/navigation";

  const {terminal} = getState();

  let selectedButton = $state(0);
  let showButtons = $state(true);

  const buttons = $state([
    {text: "Tweet your attestation", action: "tweet"},
    {text: "View contributions", action: "view"}
  ]);

  let unsubscribe: (() => void) | undefined;
  let subscriptionTimeout: NodeJS.Timeout | undefined;
  onMount(() => {
    terminal.updateHistory("Thank you!");
    terminal.updateHistory("Your contribution is complete. Thank you for securing the Union network. Tweet your cryptographic attestation for extra transparency.");


    subscriptionTimeout = setTimeout(() => {
      unsubscribe = terminal.keys.subscribe((event) => {
        if (event) {
          if (event.type === "keydown" && terminal.tab === 1) {
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
      });
    }, 200);
    return (() => {
      if (subscriptionTimeout) {
        clearTimeout(subscriptionTimeout);
      }
      if (unsubscribe) {
        unsubscribe();
      }
    })
  });

  async function shareOnTwitter() {
    showButtons = false
    terminal.updateHistory("Preparing tweet...", {duplicate: true});
    let hash = await getPublicHash();
    await sleep(2000)
    terminal.updateHistory("Opening X (twitter)...", {duplicate: true});
    await sleep(2000)
    let url = `https://ceremony.union.build/contributions/${hash}`;
    const tweetText = `I just contributed to the @union_build Trusted Setup Ceremony, to secure its ZK circuit for trustless, decentralized interoperability. \n\nI attest to my contribution. My public key hash is: \n\n${url}\n\n#JoinTheUnion`;
    const twitterIntentUrl = new URL("https://twitter.com/intent/tweet");
    twitterIntentUrl.searchParams.append("text", tweetText);
    window.open(twitterIntentUrl.toString(), "_blank");
    showButtons = true
  }

  function triggerAction(index: number) {
    if (buttons[index].action === "tweet") {
      shareOnTwitter()
    } else if (buttons[index].action === "view") {
      terminal.setTab(3)
    }
  }
</script>

{#if showButtons}
  {#each buttons as btn, index}
    <button
            class="block outline-none focus:ring-2 focus:ring-transparent focus:border-none"
            class:text-union-accent-500={selectedButton === index}
            onclick={() => triggerAction(index)}
    >
      &gt; {btn.text}
    </button>
  {/each}
{/if}