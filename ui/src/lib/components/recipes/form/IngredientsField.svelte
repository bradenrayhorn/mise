<script lang="ts">
  import IconTrash from '~icons/mdi/trash-outline';
  import { type SuperForm, type Infer, arrayProxy } from 'sveltekit-superforms';
  import { type RecipeFormSchema } from './schema';

  export let superform: SuperForm<Infer<RecipeFormSchema>>;

  const { values: blocks } = arrayProxy(superform, 'ingredient_blocks');
</script>

<div class="flex flex-col gap-10">
  {#each $blocks as _, i}
    <div>
      {#if $blocks.length > 1}
        <div class="font-serif font-bold mb-4 flex justify-between">
          {$blocks[i].title ? $blocks[i].title : `Section ${i + 1}`}

          <button
            class="text-lg"
            on:click|preventDefault={() => {
              const next = [...$blocks];
              next.splice(i, 1);
              $blocks = next;
            }}><IconTrash /></button
          >
        </div>

        <div class="mb-8">
          <label>
            <input
              class="input"
              bind:value={$blocks[i].title}
              placeholder="Title"
              aria-label="Title"
            />
          </label>
        </div>
      {/if}

      <div class="flex flex-col gap-2">
        {#each $blocks[i].ingredients as _, j}
          <input
            class="input"
            bind:value={$blocks[i].ingredients[j]}
            on:input={(e) => {
              const currentIngredients = $blocks[i].ingredients;
              if (
                e.currentTarget.value?.trim()?.length > 0 &&
                currentIngredients.length === j + 1
              ) {
                $blocks[i].ingredients = [...currentIngredients, ''];
              }
            }}
            on:keydown={(e) => {
              const currentIngredients = $blocks[i].ingredients;
              if (
                e.key === 'Backspace' &&
                currentIngredients.length > 1 &&
                currentIngredients.length - 1 !== j &&
                $blocks[i].ingredients[j] === ''
              ) {
                const next = [...currentIngredients];
                next.splice(j, 1);
                $blocks[i].ingredients = next;
              }
            }}
          />
        {/each}
      </div>
    </div>
  {/each}
</div>

<button
  class="text-sm text-text-300 text-right mt-4 w-full"
  on:click|preventDefault={() => {
    $blocks = [...$blocks, { title: '', ingredients: [''] }];
  }}>Add additional section</button
>
