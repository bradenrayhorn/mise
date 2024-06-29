<script lang="ts">
  import IconChef from '~icons/mdi/chef-hat';
  import IconSad from '~icons/mdi/emoticon-sad-outline';
  import IconRight from '~icons/mdi/chevron-right';
  import StreamedError from '$lib/components/StreamedError.svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { setQueryParameters } from '$lib/replace-query-parameter';
  import type { PageData } from './$types';
  import TagFilter from './TagFilter.svelte';
  import RecipeImage from '$lib/components/RecipeImage.svelte';
  import DesktopTagFilter from './DesktopTagFilter.svelte';

  export let data: PageData;

  $: hasCursor = $page.url.searchParams.has('cursor');
  $: searchParams = $page.url.searchParams;

  let searchValue = $page.url.searchParams.get('search') ?? '';
  $: tagValues = ($page.url.searchParams.get('tags') ?? '').split(',').filter((t) => t);
</script>

<div class="w-full min-h-dvh mx-auto flex flex-col justify-between">
  <div class="p-4 mb-4 flex justify-between items-center">
    <h1 class="font-bold text-3xl font-serif">Recipes</h1>

    <a href="/recipes/new" class="text-sm text-primary-800 font-semibold">Add</a>
  </div>

  <div class="flex grow">
    <div class="px-6 py-4 flex-col w-80 hidden md:flex">
      <div class="font-bold mb-4">Filter</div>
      <DesktopTagFilter
        defaultTagSet={tagValues}
        promisedTags={data.promisedTags}
        on:applied={(event) => {
          goto(
            `/recipes?${setQueryParameters(searchParams, { cursor: '', tags: event.detail.tag_ids.join(',') })}`,
            {
              invalidateAll: false,
            },
          );
        }}
      />
    </div>

    <div class="flex flex-col justify-between h-full grow">
      <div>
        <div class="flex gap-2 px-4 md:py-4">
          <form
            class="grow"
            method="POST"
            on:submit={(e) => {
              e.preventDefault();
              goto(
                `/recipes?${setQueryParameters(searchParams, { cursor: '', search: searchValue })}`,
              );
            }}
          >
            <input
              type="text"
              class="input"
              aria-label="Search"
              placeholder="Search"
              bind:value={searchValue}
            />

            <button type="submit" class="hidden">Search</button>
          </form>

          <div class="items-center flex md:hidden">
            <TagFilter
              promisedTags={data.promisedTags}
              defaultTagSet={tagValues}
              on:applied={(event) => {
                goto(
                  `/recipes?${setQueryParameters(searchParams, { cursor: '', tags: event.detail.tag_ids.join(',') })}`,
                  {
                    invalidateAll: false,
                  },
                );
              }}
            />
          </div>
        </div>

        <div class="flex flex-col">
          {#await data.promisedRecipePage}
            <div>Loading...</div>
          {:then page}
            <div class="flex flex-col pl-4 md:px-4">
              {#each page.data as recipe (recipe.id)}
                <a
                  class="flex items-center border-b-neutral-100 border-b py-3 pr-4 md:px-4 hover:bg-base-200 hover:shadow transition"
                  href={`/recipes/${recipe.id}`}
                >
                  {#if recipe.image_id}
                    <img
                      src={`/api/v1/images/${recipe.image_id}`}
                      alt={recipe.title}
                      class="w-16 h-16 shrink-0 object-cover shadow-inner rounded"
                    />
                  {:else}
                    <div class="w-16 h-16 shrink-0 rounded overflow-hidden text-4xl">
                      <RecipeImage title={recipe.title} />
                    </div>
                  {/if}
                  <span class="ml-4 font-semibold line-clamp-2">{recipe.title}</span>
                </a>
              {:else}
                <div class="flex flex-col items-center pt-12">
                  <div class="flex flex-col items-center justify-center text-text-300 mb-6">
                    <IconChef class="text-6xl -mb-4 -ml-1" />
                    <IconSad class="text-5xl" />
                  </div>
                  <span>Sorry, Chef.</span>
                  <span>No recipes found.</span>
                </div>
              {/each}
            </div>

            <div class="w-full flex justify-between py-3 px-4">
              {#if hasCursor}
                <a
                  href={`/recipes?${setQueryParameters(searchParams, { cursor: '' })}`}
                  class="text-sm">Back to first</a
                >
              {:else}
                <div />
              {/if}

              {#if page.next}
                <a
                  class="flex items-center text-sm"
                  href={`/recipes?${setQueryParameters(searchParams, { cursor: page.next })}`}
                  >Next<IconRight /></a
                >
              {:else}
                <div />
              {/if}
            </div>
          {:catch error}
            <StreamedError {error}>Could not load recipes.</StreamedError>
          {/await}
        </div>
      </div>
    </div>
  </div>
  <div
    class="bg-base-100 w-full px-4 py-2 mt-3 border-b-primary-500 border-b-4 flex justify-between text-xs"
  >
    <div>mise</div>
    <a href="/settings">Settings</a>
  </div>
</div>
