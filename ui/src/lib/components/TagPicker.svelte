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
  class="btn-solid btn-gray w-full text-sm text-center py-1"
  data-active={$open}
>
  Add Tag
</button>

{#if open}
  <div
    class="z-10 max-h-60 overflow-y-auto flex flex-col shadow rounded bg-base-600 p-1 min-w-40"
    use:melt={$menu}
  >
    {#each tags as tag (tag.id)}
      <button
        class="pl-3 data-[highlighted]:bg-base-primaryHover py-1 text-left"
        use:melt={$item}
        on:m-click={() => {
          onSelect(tag.id);
        }}
      >
        {tag.name}
      </button>
    {:else}
      No options
    {/each}

    {#if canCreate}
      <div class="h-px shrink-0 bg-divider-default my-2" use:melt={$separator} />

      <TagModal element={item} />
    {/if}
  </div>
{/if}
