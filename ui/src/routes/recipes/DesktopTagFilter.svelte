<script lang="ts">
  import { getTags } from '$lib/api/load/tag';
  import { queryKeys } from '$lib/api/query-keys';
  import SingleTag from '$lib/components/SingleTag.svelte';
  import StreamedError from '$lib/components/StreamedError.svelte';
  import TagPicker from '$lib/components/TagPicker.svelte';
  import type { Tag } from '$lib/types/tag';
  import { createQuery } from '@tanstack/svelte-query';
  import { createEventDispatcher } from 'svelte';

  const tagsQuery = createQuery<Array<Tag>>({
    queryKey: [queryKeys.tag.list],
    queryFn: () => getTags({ fetch }),
  });

  $: tags = $tagsQuery.data;
  export let defaultTagSet: Array<string>;

  const dispatch = createEventDispatcher();

  function onDelete(id: string) {
    dispatch('applied', {
      tag_ids: defaultTagSet.filter((tagId) => tagId !== id),
    });
  }
</script>

<div class="w-full">
  {#if $tagsQuery.isPending}
    Loading...
  {:else if $tagsQuery.isError}
    <StreamedError error={$tagsQuery.error}>Failed to load tags.</StreamedError>
  {:else if tags !== undefined}
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
  {/if}
</div>
