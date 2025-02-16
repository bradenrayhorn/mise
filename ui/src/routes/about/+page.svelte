<script lang="ts">
  import { sticky } from '$lib/actions/sticky';

  async function getUILicenses() {
    const res = await fetch('/licenses/ui-licenses.txt');
    const values = await res.text();

    return values;
  }
  async function getServerLicenses() {
    const res = await fetch('/licenses/server-licenses.json');
    const values = await res.json();

    return values.third_party_libraries;
  }

  const uiLicenses = getUILicenses();
  const serverLicenses = getServerLicenses();
</script>

<h1
  use:sticky
  class="font-serif text-fg-highlight text-3xl font-bold px-4 pb-6 ptsafe-6 sticky top-0 transition-all bg-base-500 stuck:bg-base-600 stuck:shadow-md"
>
  About
</h1>

<div class="prose px-4 pbsafe-4">
  <p>mise is a self-hosted recipe service.</p>

  <p>
    Source code is available on{' '}<a href="https://github.com/bradenrayhorn/mise">GitHub</a>.
    Licensed under
    <a href="https://github.com/bradenrayhorn/mise/blob/main/LICENSE">AGPLv3</a>.
  </p>

  <p>
    Built using <a href="https://svelte.dev">Svelte</a> and
    <a href="https://www.rust-lang.org/">Rust</a>.
  </p>

  <p>
    Logo icon from <a href="https://iconpark.oceanengine.com/home">Icon Park</a> under Apache 2.0 License.
  </p>

  <p>
    Other icons from <a href="https://github.com/Templarian/MaterialDesign">Material Design Icons</a
    > under Apache 2.0 License.
  </p>

  <p>Styled using Tailwind CSS.</p>

  <h2 class="my-8 text-2xl font-bold">Bundled libraries and licenses</h2>

  <div class="whitespace-pre-line my-2">
    {#await uiLicenses}
      Loading...
    {:then text}
      {text}
    {:catch}
      Could not load licenses.
    {/await}
  </div>

  <div class="whitespace-pre-line my-2">
    {#await serverLicenses}
      Loading...
    {:then licenses}
      {#each licenses as license (`${license.package_name}-${license.package_version}`)}
        <div>
          <p><b>{license.package_name} {license.package_version}</b></p>
          <p><a href={license.repository}>{license.repository}</a></p>
          <p>{license.license}</p>
          {#each license.licenses as l (l.license)}
            <div class="whitespace-pre-line">{l.text}</div>
          {/each}
        </div>
      {/each}
    {:catch}
      Could not load licenses.
    {/await}
  </div>
</div>
