<script lang="ts">
  import { type SuperForm, type Infer, arrayProxy } from 'sveltekit-superforms';
  import { type RecipeFormSchema } from './schema';
  import type { Tag } from '$lib/types/tag';
  import StreamedError from '$lib/components/StreamedError.svelte';
  import SingleTag from '$lib/components/SingleTag.svelte';
  import TagPicker from '$lib/components/TagPicker.svelte';
  import { createQuery } from '@tanstack/svelte-query';
  import { queryKeys } from '$lib/api/query-keys';
  import { getTags } from '$lib/api/load/tag';

  export let superform: SuperForm<Infer<RecipeFormSchema>>;
  const tagsQuery = createQuery<Array<Tag>>({
    queryKey: [queryKeys.tag.list],
    queryFn: () => getTags({ fetch }),
  });
  $: availableTags = $tagsQuery.data;

  const { values: tags } = arrayProxy(superform, 'tags');

  $: selectedTagsSet = new Set($tags.map((t) => t.id));
</script>

{#if $tagsQuery.isPending}
  Loading...
{:else if $tagsQuery.isError}
  <StreamedError error={$tagsQuery.error}>Failed to load tags.</StreamedError>
{:else if availableTags}
  <TagPicker
    canCreate={true}
    tags={availableTags.filter((t) => !selectedTagsSet.has(t.id))}
    on:select={({ detail: { tagID } }) => {
      const nextTag = availableTags.find((t) => t.id === tagID);
      if (nextTag) {
        $tags = [...$tags, nextTag];
      }
    }}
  />
{/if}

<ul class="flex flex-wrap gap-2" aria-label="Tags">
  {#each $tags as tag}
    <SingleTag
      name={tag.name}
      canDelete={true}
      on:click={() => {
        $tags = $tags.filter((t) => t.id !== tag.id);
      }}
    />
  {/each}
</ul>
