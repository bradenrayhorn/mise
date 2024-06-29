<script lang="ts">
  import SingleTag from '$lib/components/SingleTag.svelte';
  import StreamedError from '$lib/components/StreamedError.svelte';
  import TagPicker from '$lib/components/TagPicker.svelte';
  import type { Tag } from '$lib/types/tag';
  import { melt } from '@melt-ui/svelte';
  import { createEventDispatcher } from 'svelte';

  export let promisedTags: Promise<Array<Tag>>;
  export let defaultTagSet: Array<string>;

  const dispatch = createEventDispatcher();

  function onDelete(id: string) {
    dispatch('applied', {
      tag_ids: defaultTagSet.filter((tagId) => tagId !== id),
    });
  }
</script>

<div class="w-full">
  {#await promisedTags}
    Loading...
  {:then tags}
    <div class="flex flex-wrap gap-2">
      {#each defaultTagSet as id (id)}
        <SingleTag
          canDelete={true}
          on:click={() => {
            onDelete(id);
          }}
        >
          {tags.find((t) => t.id === id)?.name}
        </SingleTag>
      {/each}
    </div>
    <div class="mt-6 w-full text-right">
      <TagPicker
        tags={tags.filter((tag) => !defaultTagSet.includes(tag.id))}
        on:select={({ detail: { tagID } }) => {
          dispatch('applied', { tag_ids: [...defaultTagSet, tagID] });
        }}
      />
    </div>
  {:catch error}
    <StreamedError {error}>Failed to load tags.</StreamedError>
  {/await}
</div>
