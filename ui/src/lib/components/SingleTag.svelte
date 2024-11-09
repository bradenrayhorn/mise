<script lang="ts">
  import type { MouseEventHandler } from 'svelte/elements';
  import DeleteIcon from '~icons/mdi/close';

  type Props = {
    canDelete: boolean;
    name: string;
    onclick?: MouseEventHandler<HTMLButtonElement>;
  };

  let { canDelete, name, onclick }: Props = $props();
</script>

<li
  class="rounded-full bg-tag-bg text-tag-fg py-1 font-semibold text-sm w-fit flex"
  class:px-4={!canDelete}
  class:pl-4={canDelete}
>
  {name}
  {#if canDelete}
    <button
      class="ml-2 px-2 border-l border-tag-divider"
      aria-label={`Delete tag ${name}`}
      type="button"
      {onclick}
    >
      <DeleteIcon />
    </button>
  {/if}
</li>
