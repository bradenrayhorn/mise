<script lang="ts">
  import { type SuperForm, type Infer, arrayProxy } from 'sveltekit-superforms';
  import { type RecipeFormSchema } from './schema';
  import type { Tag } from '$lib/types/tag';
  import { createDropdownMenu, melt } from '@melt-ui/svelte';
  import TagModal from './TagModal.svelte';
  import StreamedError from '$lib/components/StreamedError.svelte';

  export let superform: SuperForm<Infer<RecipeFormSchema>>;
  export let promisedTags: Promise<Array<Tag>>;

  const { values: tags } = arrayProxy(superform, 'tags');

  $: selectedTagsSet = new Set($tags.map((t) => t.id));

  const {
    elements: { menu, item, trigger, arrow, separator },
  } = createDropdownMenu();
</script>

{#each $tags as tag}
  <div>{tag.name}</div>
{/each}

{#await promisedTags}
  <button disabled>Add Tag</button>
{:then availableTags}
  <button use:melt={$trigger}>Add Tag</button>
  <div use:melt={$menu}>
    {#each availableTags.filter((t) => !selectedTagsSet.has(t.id)) as tag (tag.id)}
      <button
        use:melt={$item}
        on:click={() => {
          let next = [...$tags, tag];
          $tags = next;
        }}>{tag.name}</button
      >
    {/each}
    <div use:melt={$separator} />
    <TagModal element={item} />
    <div use:melt={$arrow} />
  </div>
{:catch error}
  <StreamedError {error}>Failed to load tags.</StreamedError>
{/await}
