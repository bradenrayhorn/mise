<script lang="ts">
  import IconTag from '~icons/mdi/tag-multiple-outline';
  import IconTagActive from '~icons/mdi/tag-multiple';
  import IconClose from '~icons/mdi/close';
  import type { Tag } from '$lib/types/tag';
  import { createDialog, melt } from '@melt-ui/svelte';
  import TagFilterOption from './TagFilterOption.svelte';
  import StreamedError from '$lib/components/StreamedError.svelte';
  import { fade, slide } from 'svelte/transition';
  import { queryKeys } from '$lib/api/query-keys';
  import { getTags } from '$lib/api/load/tag';
  import { createQuery } from '@tanstack/svelte-query';

  type Props = {
    defaultTagSet: Array<string>;
    onapplied: (event: { tag_ids: string[] }) => void;
  };
  const { defaultTagSet, onapplied }: Props = $props();

  const tagsQuery = createQuery<Array<Tag>>({
    queryKey: [queryKeys.tag.list],
    queryFn: () => getTags({ fetch }),
  });
  const tags = $derived($tagsQuery.data);

  const hasTagsApplied = $derived(defaultTagSet.length > 0);

  const {
    elements: { trigger, portalled, overlay, content, title, close },
    states: { open },
  } = createDialog({
    closeOnOutsideClick: false,
    onOpenChange: function ({ next }) {
      if (next) {
        nextTags = Object.fromEntries(defaultTagSet.map((id) => [id, true])); // reset tag selection
      }
      return next;
    },
  });

  let nextTags: { [id: string]: boolean } = $state(
    Object.fromEntries(defaultTagSet.map((id) => [id, true])),
  );

  function onApply() {
    $open = false;
    onapplied({
      tag_ids: Object.entries(nextTags)
        .filter(([k, v]) => v && k)
        .map(([k]) => k),
    });
  }

  function onClear() {
    $open = false;
    onapplied({ tag_ids: [] });
  }
</script>

<button use:melt={$trigger} class="text-2xl" aria-label="Tag filter">
  {#if hasTagsApplied}
    <IconTagActive />
  {:else}
    <IconTag />
  {/if}
</button>

{#if $open}
  <div use:melt={$portalled}>
    <div
      use:melt={$overlay}
      class="fixed z-40 bg-base-backdrop top-0 bottom-0 right-0 left-0"
      aria-hidden="true"
      onclick={(e) => {
        e.stopPropagation();
        $open = false;
      }}
      transition:fade={{ duration: 100 }}
    ></div>
    <div
      use:melt={$content}
      class="fixed z-50 bottom-0 left-0 right-0 h-[min(clamp(24rem,50dvh,100dvh),100dvh)] bg-base-500 rounded-t-xl flex flex-col"
      transition:slide={{ duration: 300 }}
    >
      <div class="flex items-center p-4 border-b-divider-default border-b mb-4 shrink-0">
        <div class="flex-1 flex items-center">
          <button use:melt={$close} class="rounded-full text-fg-muted p-1 text-lg"
            ><IconClose /></button
          >
        </div>
        <h2 use:melt={$title} class="font-semibold">Tags</h2>
        <div class="flex-1"></div>
      </div>

      <div class="flex flex-col flex-1 gap-2 overflow-y-auto px-4">
        {#if $tagsQuery.isPending}
          Loading...
        {:else if $tagsQuery.isError}
          <StreamedError error={$tagsQuery.error}>Failed to load tags.</StreamedError>
        {:else if tags !== undefined}
          {#each tags as tag (tag.id)}
            <TagFilterOption {tag} bind:isChecked={nextTags[tag.id]} />
          {:else}
            No tags available.
          {/each}
        {/if}
      </div>

      <div class="shrink-0 p-4 flex gap-2">
        <button onclick={onClear} disabled={!hasTagsApplied} class="btn-solid btn-gray btn-sm grow">
          Clear
        </button>
        <button onclick={onApply} class="btn-solid btn-primary btn-sm grow">Apply</button>
      </div>
    </div>
  </div>
{/if}
