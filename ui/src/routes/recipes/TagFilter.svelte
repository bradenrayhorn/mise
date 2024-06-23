<script lang="ts">
  import type { Tag } from '$lib/types/tag';
  import { createDialog, melt } from '@melt-ui/svelte';
  import { createEventDispatcher } from 'svelte';
  import TagFilterOption from './TagFilterOption.svelte';
  import StreamedError from '$lib/components/StreamedError.svelte';

  export let promisedTags: Promise<Array<Tag>>;
  export let defaultTagSet: Array<string>;

  const dispatch = createEventDispatcher();

  const {
    elements: { trigger, portalled, overlay, content, title, close },
    states: { open },
  } = createDialog({
    onOpenChange: function ({ next }) {
      nextTags = Object.fromEntries(defaultTagSet.map((id) => [id, true])); // reset tag selection
      return next;
    },
  });

  let nextTags: { [id: string]: boolean } = Object.fromEntries(
    defaultTagSet.map((id) => [id, true]),
  );

  function onApply() {
    dispatch('applied', {
      tag_ids: Object.entries(nextTags)
        .filter(([k, v]) => v && k)
        .map(([k]) => k),
    });
    $open = false;
  }
</script>

{#await promisedTags then tags}
  <button use:melt={$trigger}>T</button>

  <!-- TODO - when styling for mobile, instead of a modal bring form up as a drawer from bottom -->

  {#if $open}
    <div use:melt={$portalled}>
      <div use:melt={$overlay} />
      <div use:melt={$content}>
        <h2 use:melt={$title}>Filter by tag</h2>
        <div class="flex flex-col gap-2">
          {#each tags as tag (tag.id)}
            <TagFilterOption {tag} bind:isChecked={nextTags[tag.id]} />
          {:else}
            No tags available.
          {/each}
        </div>
        <button use:melt={$close}>Cancel</button>
        <button on:click={onApply}>Apply</button>
      </div>
    </div>
  {/if}
{:catch error}
  <StreamedError {error}>Failed to load tags.</StreamedError>
{/await}
