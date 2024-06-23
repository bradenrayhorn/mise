<script lang="ts">
  import type { Tag } from '$lib/types/tag';
  import { createCheckbox, createSync, melt } from '@melt-ui/svelte';

  export let tag: Tag;
  export let isChecked: boolean;

  const {
    elements: { root, input },
    helpers: { isChecked: localIsChecked },
    states,
  } = createCheckbox({
    defaultChecked: false,
  });

  const sync = createSync(states);
  $: sync.checked(isChecked, (v) => (isChecked = !!v));
</script>

<div>
  <button use:melt={$root}>
    {#if $localIsChecked}
      X
    {/if}
    <input use:melt={$input} />

    {tag.name}
  </button>
</div>
