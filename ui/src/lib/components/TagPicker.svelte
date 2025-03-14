<script lang="ts">
  import type { Tag } from '$lib/types/tag';
  import * as menu from '@zag-js/menu';
  import TagModal from './recipes/form/TagModal.svelte';
  import { useMachine, normalizeProps, portal } from '@zag-js/svelte';

  type Props = {
    tags: Array<Tag>;
    canCreate?: boolean;
    onselect: (event: { tagID: string }) => void;
  };

  let { tags, onselect, canCreate = false }: Props = $props();

  const id = $props.id();
  const service = useMachine(menu.machine, {
    id,
    typeahead: true,
    positioning: { sameWidth: true },
    onSelect: (details) => {
      if (!details.value.startsWith('*')) {
        onselect({ tagID: details.value });
      }
    },
  });
  const api = $derived(menu.connect(service, normalizeProps));
</script>

<button
  {...api.getTriggerProps()}
  data-active={api.open}
  class="btn-solid btn-gray w-full text-sm text-center py-1"
>
  Add Tag
</button>

{#if api.open}
  <div
    class="z-10 shadow rounded bg-base-600 p-1 min-w-40"
    use:portal
    {...api.getPositionerProps()}
  >
    <div class="overflow-y-auto flex flex-col max-h-60" {...api.getContentProps()}>
      {#each tags as tag (tag.id)}
        <button
          class="pl-3 data-[highlighted]:bg-base-primaryHover py-1 text-left"
          {...api.getItemProps({ value: tag.id })}
        >
          {tag.name}
        </button>
      {:else}
        No options
      {/each}

      {#if canCreate}
        <div class="h-px shrink-0 bg-divider-default my-2" {...api.getSeparatorProps()}></div>

        <TagModal {...api.getItemProps({ value: '*open-tag-modal' })} />
      {/if}
    </div>
  </div>
{/if}
