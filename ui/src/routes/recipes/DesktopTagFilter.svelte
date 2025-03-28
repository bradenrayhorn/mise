<script lang="ts">
  import { getTags } from '$lib/api/load/tag';
  import { queryKeys } from '$lib/api/query-keys';
  import SingleTag from '$lib/components/SingleTag.svelte';
  import StreamedError from '$lib/components/StreamedError.svelte';
  import TagPicker from '$lib/components/TagPicker.svelte';
  import type { Tag } from '$lib/types/tag';
  import { createQuery } from '@tanstack/svelte-query';

  type Props = {
    defaultTagSet: Array<string>;
    onapplied: (event: { tag_ids: string[] }) => void;
  };
  let { defaultTagSet, onapplied }: Props = $props();

  const tagsQuery = createQuery<Array<Tag>>({
    queryKey: [queryKeys.tag.list],
    queryFn: () => getTags({ fetch }),
  });

  const tags = $derived($tagsQuery.data);

  function onDelete(id: string) {
    onapplied({
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
          name={tags.find((t) => t.id === id)?.name ?? ''}
          canDelete={true}
          onclick={() => {
            onDelete(id);
          }}
        />
      {/each}
    </div>
    <div class="mt-6 w-full text-right">
      <TagPicker
        tags={tags.filter((tag) => !defaultTagSet.includes(tag.id))}
        onselect={({ tagID }) => {
          onapplied({ tag_ids: [...defaultTagSet, tagID] });
        }}
      />
    </div>
  {/if}
</div>
