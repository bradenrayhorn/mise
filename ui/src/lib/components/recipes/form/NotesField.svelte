<script lang="ts">
  import { formFieldProxy, type Infer, type SuperForm } from 'sveltekit-superforms';
  import type { RecipeFormSchema } from './schema';

  export let superform: SuperForm<Infer<RecipeFormSchema>>;

  const { value, errors } = formFieldProxy(superform, 'notes');
</script>

<label>
  <span>Notes</span>
  <textarea
    class="input resize-none h-40"
    bind:value={$value}
    aria-invalid={$errors ? 'true' : undefined}
  />
</label>
{#if $errors}<span class="invalid">{$errors}</span>{/if}
