<script lang="ts">
  import StreamedError from '$lib/components/StreamedError.svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { setQueryParameters } from '$lib/replace-query-parameter';
  import type { PageData } from './$types';
  import TagFilter from './TagFilter.svelte';

  export let data: PageData;

  $: hasCursor = $page.url.searchParams.has('cursor');
  $: searchParams = $page.url.searchParams;

  let searchValue = $page.url.searchParams.get('search') ?? '';
  $: tagValues = ($page.url.searchParams.get('tags') ?? '').split(',');
</script>

<h1 class="font-bold text-3xl font-serif mb-16">recipes</h1>

<a href="/recipes/new" class="text-sm text-primary-800 font-semibold">Add Recipe</a>

{#if hasCursor}
  <a href={`/recipes?${setQueryParameters(searchParams, { cursor: '' })}`}>Back to first page.</a>
{/if}

<div class="flex gap-4">
  <form>
    <label>
      Search
      <input type="text" class="border border-black" bind:value={searchValue} />
    </label>

    <a href={`/recipes?${setQueryParameters(searchParams, { cursor: '', search: searchValue })}`}
      >GO</a
    >
  </form>

  <TagFilter
    promisedTags={data.promisedTags}
    defaultTagSet={tagValues}
    on:applied={(event) => {
      goto(
        `/recipes?${setQueryParameters(searchParams, { cursor: '', tags: event.detail.tag_ids.join(',') })}`,
      );
    }}
  />
</div>

<div class="flex flex-col">
  {#await data.promisedRecipePage}
    <div>Loading...</div>
  {:then page}
    {#each page.data as recipe (recipe.id)}
      <a href={`/recipes/${recipe.id}`}>{recipe.title}</a>
    {:else}
      No recipe results.
    {/each}

    {#if page.next}
      <a href={`/recipes?${setQueryParameters(searchParams, { cursor: page.next })}`}>Next page</a>
    {/if}
  {:catch error}
    <StreamedError {error}>Could not load recipes.</StreamedError>
  {/await}
</div>
