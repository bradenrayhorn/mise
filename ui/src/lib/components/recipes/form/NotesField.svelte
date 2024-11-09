<script lang="ts">
  import { formFieldProxy, type Infer, type SuperForm } from 'sveltekit-superforms';
  import type { RecipeFormSchema } from './schema';

  interface Props {
    superform: SuperForm<Infer<RecipeFormSchema>>;
  }

  const { superform }: Props = $props();

  const { value, errors } = formFieldProxy(superform, 'notes');
</script>

<label>
  <span>Notes</span>
  <textarea
    class="input resize-none h-40"
    bind:value={$value}
    aria-invalid={$errors ? 'true' : undefined}
  ></textarea>
</label>
{#if $errors}<span class="invalid">{$errors}</span>{/if}
