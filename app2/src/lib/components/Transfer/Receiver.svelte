<script lang="ts">
  import Input from "$lib/components/ui/Input.svelte";
  import {getTransfer} from "$lib/components/Transfer/transfer.svelte.ts";

  const {transfer} = getTransfer()
</script>

<Input
        type="text"
        id="receiver"
        label="receiver"
        required={true}
        disabled={!transfer.raw.destination}
        autocorrect="off"
        spellcheck="false"
        autocomplete="off"
        data-field="receiver"
        placeholder="Enter destination address"
        value={transfer.raw.receiver}
        oninput={event => transfer.raw.updateField('receiver', event)}
        class="text-center"
/>