<script lang="ts">
  import type { Tag } from '$lib/types/tag';
  import { createDropdownMenu, melt } from '@melt-ui/svelte';
  import { createEventDispatcher } from 'svelte';
  import TagModal from './recipes/form/TagModal.svelte';

  export let tags: Array<Tag>;
  export let canCreate: boolean = false;

  const dispatch = createEventDispatcher();

  const {
    elements: { trigger, menu, item, separator },
    states: { open },
  } = createDropdownMenu({
    loop: true,
  });

  function onSelect(id: string) {
    dispatch('select', { tagID: id });
  }
</script>

<button
  use:melt={$trigger}
  class="w-full rounded bg-neutral-100 text-sm font-semibold text-center py-1 hover:bg-neutral-200 transition-colors"
>
  Add Tag
</button>

{#if open}
  <div
    class="z-10 max-h-60 overflow-y-auto flex-col shadow rounded bg-base-200 p-1 min-w-40"
    use:melt={$menu}
  >
    {#each tags as tag (tag.id)}
      <div
        class="pl-3 data-[highlighted]:bg-primary-100 text-text-200 py-1"
        use:melt={$item}
        on:m-click={() => {
          onSelect(tag.id);
        }}
      >
        {tag.name}
      </div>
    {:else}
      No options
    {/each}

    {#if canCreate}
      <div class="h-px bg-primary-200 my-2" use:melt={$separator} />

      <TagModal element={item} />
    {/if}
  </div>
{/if}
