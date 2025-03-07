<script lang="ts">
  import Input from "$lib/components/ui/Input.svelte";
  import {getTransfer} from "../../../routes/transfer/transfer.svelte.ts";

  const {transfer} = getTransfer()
</script>

<Input
        type="text"
        id="receiver"
        label="receiver"
        required={true}
        disabled={!transfer.url.destination}
        autocorrect="off"
        spellcheck="false"
        autocomplete="off"
        data-field="receiver"
        placeholder="Enter destination address"
        value={transfer.url.receiver}
        oninput={event => transfer.url.updateField('receiver', event)}
        class="text-center"
/>