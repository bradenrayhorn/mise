<script lang="ts">
  import type { DetailedRecipe } from '$lib/types/recipe';

  export let recipe: DetailedRecipe;
</script>

<div class="flex flex-col md:flex-row gap-8 px-2">
  <div class="flex-1 flex flex-col">
    {#if recipe.image_id}
      <img
        src={`/api/v1/images/${recipe.image_id}`}
        alt="recipe"
        class="w-full h-56 object-cover"
      />
    {/if}

    <div>{recipe.title}</div>

    <div>{recipe.notes ?? ''}</div>

    <div class="flex flex-col">
      {#each recipe.tags as tag (tag.id)}
        <div>{tag.name}</div>
      {/each}
    </div>
  </div>

  <div class="flex-1">
    <h2>Ingredients</h2>

    {#each recipe.ingredient_blocks as b}
      {#if b.title}
        <div>
          {b.title}
        </div>
      {/if}

      {#each b.ingredients as ingredient}
        <div>
          - {ingredient}
        </div>
      {/each}
    {/each}
  </div>

  <div class="flex-1">
    <h2>Instructions</h2>

    {#each recipe.instruction_blocks as b}
      {#if b.title}
        <div>
          {b.title}
        </div>
      {/if}

      {#each b.instructions as instruction, i}
        <div>
          {i + 1}. {instruction}
        </div>
      {/each}
    {/each}
  </div>
</div>
