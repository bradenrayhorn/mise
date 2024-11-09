<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { setQueryParameters } from '$lib/replace-query-parameter';

  let queryParams = $derived($page.url.searchParams);
  let searchValue = $state($page.url.searchParams.get('search') ?? '');

  page.subscribe((newPage) => {
    searchValue = newPage.url.searchParams.get('search') ?? '';
  });
</script>

<form
  class="grow"
  method="POST"
  onsubmit={(e) => {
    e.preventDefault();
    goto(`/recipes?${setQueryParameters(queryParams, { cursor: '', search: searchValue })}`);
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
