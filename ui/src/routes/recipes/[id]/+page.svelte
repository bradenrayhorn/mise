<script lang="ts">
  import SingleTag from '$lib/components/SingleTag.svelte';
  import type { PageData } from './$types';
  import CloseIconButton from '$lib/components/CloseIconButton.svelte';

  export let data: PageData;
  $: recipe = data.recipe;
</script>

<div class="absolute top-1 left-1 z-10 flex items-center">
  <CloseIconButton href={data.backURL} />
</div>

<div class="w-full h-full flex flex-col pb-8">
  <div class="flex justify-between items-baseline px-4 md:px-8 lg:px-12 pb-8 pt-12">
    <div class="text-3xl font-serif font-bold pr-4">{recipe.title}</div>

    <a href={`/recipes/${data.id}/edit`} class="btn-link text-sm">Edit</a>
  </div>

  <div class="flex flex-col md:flex-row gap-8 px-4 md:px-8 lg:px-12">
    {#if recipe.image_id || recipe.tags.length > 0 || recipe.rich_notes}
      <div class="flex-1 flex flex-col">
        {#if recipe.image_id}
          <img
            src={`/api/v1/images/${recipe.image_id}`}
            alt="recipe"
            class="w-full max-w-80 h-56 object-cover rounded shadow-inner"
          />
        {/if}

        <div class="flex my-6 flex-wrap gap-2">
          {#each recipe.tags as tag (tag.id)}
            <SingleTag canDelete={false}>{tag.name}</SingleTag>
          {/each}
        </div>

        {#if recipe.rich_notes}
          <div class="prose">
            <!-- eslint-disable svelte/no-at-html-tags -->
            {@html recipe.rich_notes}
          </div>
        {/if}
      </div>
    {/if}

    <div class="flex-1">
      <h2 class="font-serif text-xl font-bold mb-4 md:mb-6">Ingredients</h2>

      <div class="flex flex-col gap-4">
        {#each recipe.ingredient_blocks as b}
          <div class="flex flex-col">
            {#if b.title}
              <div class="font-serif font-bold mb-2">
                {b.title}
              </div>
            {/if}

            <ul class="list-disc ml-4 flex flex-col gap-1">
              {#each b.ingredients as ingredient}
                <li>
                  {ingredient}
                </li>
              {/each}
            </ul>
          </div>
        {/each}
      </div>
    </div>

    <div class="flex-1">
      <h2 class="font-serif text-xl font-bold mb-4 md:mb-6">Instructions</h2>

      <div class="flex flex-col gap-4">
        {#each recipe.instruction_blocks as b}
          <div class="flex flex-col">
            {#if b.title}
              <div class="font-serif font-bold mb-2">
                {b.title}
              </div>
            {/if}

            <ol class="list-decimal ml-4 flex flex-col gap-1">
              {#each b.rich_instructions as instruction}
                <li>
                  <!-- eslint-disable svelte/no-at-html-tags -->
                  <div class="prose">{@html instruction}</div>
                </li>
              {/each}
            </ol>
          </div>
        {/each}
      </div>
    </div>
  </div>
</div>
