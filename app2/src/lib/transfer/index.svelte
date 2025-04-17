<script lang="ts">
  import Normal from './normal/index.svelte';
  import Multisig from './multisig/index.svelte';
  import Tabbed from './tabbed/index.svelte';

  type Props = {
    mode: 'normal' | 'multisig' | 'tabs';
  };

  let { mode }: Props = $props();

  const Component = $derived(
    mode === 'normal'
      ? Normal
      : mode === 'multisig'
        ? Multisig
        : Tabbed
  );
</script>

<Component />

