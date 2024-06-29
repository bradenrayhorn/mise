<script lang="ts">
  import IconTag from '~icons/mdi/tag-multiple-outline';
  import IconTagActive from '~icons/mdi/tag-multiple';
  import IconClose from '~icons/mdi/close-thick';
  import type { Tag } from '$lib/types/tag';
  import { createDialog, melt } from '@melt-ui/svelte';
  import { createEventDispatcher } from 'svelte';
  import TagFilterOption from './TagFilterOption.svelte';
  import StreamedError from '$lib/components/StreamedError.svelte';
  import { fade, slide } from 'svelte/transition';

  export let promisedTags: Promise<Array<Tag>>;
  export let defaultTagSet: Array<string>;

  $: hasTagsApplied = defaultTagSet.length > 0;

  const dispatch = createEventDispatcher();

  const {
    elements: { trigger, portalled, overlay, content, title, close },
    states: { open },
  } = createDialog({
    onOpenChange: function ({ next }) {
      if (next) {
        nextTags = Object.fromEntries(defaultTagSet.map((id) => [id, true])); // reset tag selection
      }
      return next;
    },
    forceVisible: true,
  });

  let nextTags: { [id: string]: boolean } = Object.fromEntries(
    defaultTagSet.map((id) => [id, true]),
  );

  function onApply() {
    $open = false;
    dispatch('applied', {
      tag_ids: Object.entries(nextTags)
        .filter(([k, v]) => v && k)
        .map(([k]) => k),
    });
  }

  function onClear() {
    $open = false;
    dispatch('applied', { tag_ids: [] });
  }
</script>

<button use:melt={$trigger} class="text-2xl">
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
      class="fixed z-40 bg-black/50 top-0 bottom-0 right-0 left-0"
      transition:fade={{ duration: 100 }}
    />
    <div
      use:melt={$content}
      class="fixed z-50 bottom-0 left-0 right-0 h-[min(clamp(24rem,50dvh,100dvh),100dvh)] bg-base-100 rounded-t-xl flex flex-col"
      transition:slide={{ duration: 300 }}
    >
      <div class="flex items-center p-4 border-b-neutral-100 border-b mb-4 shrink-0">
        <div class="flex-1 flex items-center">
          <button use:melt={$close} class="rounded-full bg-neutral-100 text-neutral-700 p-1"
            ><IconClose /></button
          >
        </div>
        <h2 use:melt={$title} class="font-semibold">Tags</h2>
        <div class="flex-1" />
      </div>

      <div class="flex flex-col flex-1 gap-2 overflow-y-auto px-4">
        {#await promisedTags}
          Loading...
        {:then tags}
          {#each tags as tag (tag.id)}
            <TagFilterOption {tag} bind:isChecked={nextTags[tag.id]} />
          {:else}
            No tags available.
          {/each}
        {:catch error}
          <StreamedError {error}>Failed to load tags.</StreamedError>
        {/await}
      </div>

      <div class="shrink-0 p-4 flex gap-2">
        <button
          disabled={!hasTagsApplied}
          on:click={onClear}
          class="border-primary-800 disabled:border-neutral-300 disabled:text-neutral-400 dark:border-primary-200 border-2 rounded w-full py-2 font-semibold"
          >Clear</button
        >
        <button
          on:click={onApply}
          class="bg-primary-800 text-neutral-50 dark:bg-primary-200 dark:text-neutral-950 rounded w-full py-2 font-semibold"
          >Apply</button
        >
      </div>
    </div>
  </div>
{/if}
