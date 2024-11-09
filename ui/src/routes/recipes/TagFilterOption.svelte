<script lang="ts">
  import type { Tag } from '$lib/types/tag';
  import { createCheckbox, createSync, melt } from '@melt-ui/svelte';
  import IconCheck from '~icons/mdi/check-bold';

  interface Props {
    tag: Tag;
    isChecked: boolean;
  }

  let { tag, isChecked = $bindable() }: Props = $props();

  const {
    elements: { root, input },
    helpers: { isChecked: localIsChecked },
    states,
  } = createCheckbox({
    defaultChecked: false,
  });

  const sync = createSync(states);
  sync.checked(isChecked, (v) => (isChecked = !!v));
</script>

<button
  use:melt={$root}
  class="w-full flex gap-2 items-center justify-between"
  class:font-semibold={$localIsChecked}
>
  <input use:melt={$input} />

  {tag.name}

  {#if $localIsChecked}
    <IconCheck />
  {/if}
</button>
