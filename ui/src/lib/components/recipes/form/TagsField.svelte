<script lang="ts">
  import { type SuperForm, type Infer, arrayProxy } from 'sveltekit-superforms';
  import { type RecipeFormSchema } from './schema';
  import type { Tag } from '$lib/types/tag';
  import StreamedError from '$lib/components/StreamedError.svelte';
  import SingleTag from '$lib/components/SingleTag.svelte';
  import TagPicker from '$lib/components/TagPicker.svelte';

  export let superform: SuperForm<Infer<RecipeFormSchema>>;
  export let promisedTags: Promise<Array<Tag>>;

  const { values: tags } = arrayProxy(superform, 'tags');

  $: selectedTagsSet = new Set($tags.map((t) => t.id));
</script>

{#await promisedTags}
  Loading...
{:then availableTags}
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
{:catch error}
  <StreamedError {error}>Failed to load tags.</StreamedError>
{/await}

<div class="flex flex-wrap gap-2">
  {#each $tags as tag}
    <SingleTag
      canDelete={true}
      on:click={() => {
        $tags = $tags.filter((t) => t.id !== tag.id);
      }}>{tag.name}</SingleTag
    >
  {/each}
</div>
