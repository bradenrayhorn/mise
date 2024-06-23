<script lang="ts">
  import { type SuperForm, type Infer, arrayProxy } from 'sveltekit-superforms';
  import { type RecipeFormSchema } from './schema';

  export let superform: SuperForm<Infer<RecipeFormSchema>>;

  const { values: blocks } = arrayProxy(superform, 'ingredient_blocks');
</script>

{#each $blocks as _, i}
  {#if $blocks.length > 1}
    <div>
      <label>
        <span>Title {i}</span>
        <input class="border-black border" bind:value={$blocks[i].title} />
      </label>

      <button
        on:click|preventDefault={() => {
          const next = [...$blocks];
          next.splice(i, 1);
          $blocks = next;
        }}>X</button
      >
    </div>
  {/if}

  {#each $blocks[i].ingredients as _, j}
    <div>
      <input
        class="border-black border"
        bind:value={$blocks[i].ingredients[j]}
        on:input={(e) => {
          const currentIngredients = $blocks[i].ingredients;
          if (e.currentTarget.value?.trim()?.length > 0 && currentIngredients.length === j + 1) {
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
    </div>
  {/each}
{/each}

<button
  on:click|preventDefault={() => {
    $blocks = [...$blocks, { title: '', ingredients: [''] }];
  }}>Add additional component</button
>
