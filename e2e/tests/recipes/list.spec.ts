import { expect } from '@playwright/test';
import { createRecipe, createTag } from '@tests/apis';
import { test } from '@tests/fixtures';
import { addTagFilter, removeTagFilter } from '@tests/helpers';

test('can filter by tag and search', async ({ login: user, page, request, isMobile }) => {
  const tagA = await createTag({ user, request, name: 'Alpha' });
  const tagB = await createTag({ user, request, name: 'Beta' });
  const tagC = await createTag({ user, request, name: 'Gamma' });

  await createRecipe({
    user,
    request,
    recipe: {
      title: 'Rho',
      tagIDs: [tagA, tagB],
    },
  });
  await createRecipe({
    user,
    request,
    recipe: {
      title: 'RhoTwo',
      tagIDs: [],
    },
  });

  await createRecipe({
    user,
    request,
    recipe: {
      title: 'Sigma',
      tagIDs: [tagA],
    },
  });

  await createRecipe({
    user,
    request,
    recipe: {
      title: 'Tau',
      tagIDs: [tagB, tagC],
    },
  });

  // go to page

  await page.goto('/recipes');
  const recipeList = page.getByRole('region', { name: 'Recipe list' });

  // filter by Tag A
  await addTagFilter({ page, user, isMobile, tags: ['Alpha'] });

  await expect(recipeList.getByRole('link')).toHaveText([`${user}Rho`, `${user}Sigma`]);

  // add Tag B
  await addTagFilter({ page, user, isMobile, tags: ['Beta'] });

  await expect(recipeList.getByRole('link')).toHaveText([`${user}Rho`]);

  // remove Tag A
  await removeTagFilter({ page, user, isMobile, tags: ['Alpha'] });
  await expect(recipeList.getByRole('link')).toHaveText([`${user}Rho`, `${user}Tau`]);

  // add search for Rho
  await page.getByLabel('Search').fill(`${user}Rho`);
  await page.getByLabel('Search').press('Enter');
  await expect(recipeList.getByRole('link')).toHaveText([`${user}Rho`]);

  // remove Tag filter
  await removeTagFilter({ page, user, isMobile, tags: ['Beta'] });
  await expect(recipeList.getByRole('link')).toHaveText([`${user}Rho`, `${user}RhoTwo`]);
});

test('can use pagination', async ({ login: user, page, request, isMobile }) => {
  const tagA = await createTag({ user, request, name: 'Alpha' });

  // these recipes should never show up - verifying search and filter remain applied
  await createRecipe({ user, request, recipe: { title: `00`, tagIDs: [tagA] } });
  await createRecipe({ user, request, recipe: { title: `Prefix1`, tagIDs: [] } });

  // add 25 recipes, page size is 10
  for (let i = 10; i < 35; i++) {
    await createRecipe({ user, request, recipe: { title: `Prefix${i}`, tagIDs: [tagA] } });
  }

  // go to page
  await page.goto('/recipes');
  const recipeList = page.getByRole('region', { name: 'Recipe list' });

  // add tag and search, these should be persisted through the navigation
  await addTagFilter({ page, user, isMobile, tags: ['Alpha'] });
  await page.getByLabel('Search').fill(`${user}Prefix`);
  await page.getByLabel('Search').press('Enter');

  // check page 1
  await expect(recipeList.getByRole('link')).toHaveText(
    [...Array(10).keys()].map((i) => `${user}Prefix${i + 10}`),
  );

  // next page (2)
  await page.getByRole('link', { name: 'Next' }).click();

  await expect(recipeList.getByRole('link')).toHaveText(
    [...Array(10).keys()].map((i) => `${user}Prefix${i + 20}`),
  );

  // next page (3), this is the last page
  await page.getByRole('link', { name: 'Next' }).click();

  await expect(recipeList.getByRole('link')).toHaveText(
    [...Array(5).keys()].map((i) => `${user}Prefix${i + 30}`),
  );

  // go back to page 1
  await expect(page.getByRole('link', { name: 'Next' })).not.toBeVisible();
  await page.getByRole('link', { name: 'Back to first' }).click();

  await expect(recipeList.getByRole('link')).toHaveText(
    [...Array(10).keys()].map((i) => `${user}Prefix${i + 10}`),
  );

  // search and tag are still here
  await expect(page.getByLabel('Search')).toHaveValue(`${user}Prefix`);

  if (isMobile) {
    await page.getByRole('button', { name: 'Tag filter' }).click();
    const dialog = page.getByRole('dialog', { name: 'Tags' });
    await expect(dialog.getByRole('checkbox', { name: `${user}Alpha` })).toBeChecked();
  } else {
    await expect(page.getByRole('button', { name: `Delete tag ${user}Alpha` })).toBeVisible();
  }
});

test('mobile - can clear tag filters', async ({ login: user, page, request, isMobile }) => {
  if (!isMobile) return;

  const tagA = await createTag({ user, request, name: 'Alpha' });
  const tagB = await createTag({ user, request, name: 'Beta' });

  await createRecipe({
    user,
    request,
    recipe: {
      title: 'Rho',
      tagIDs: [tagA, tagB],
    },
  });
  await createRecipe({
    user,
    request,
    recipe: {
      title: 'RhoTwo',
      tagIDs: [],
    },
  });

  // go to page
  await page.goto('/recipes');
  const recipeList = page.getByRole('region', { name: 'Recipe list' });

  await page.getByLabel('Search').fill(`${user}Rho`);
  await page.getByLabel('Search').press('Enter');

  // add filter
  await addTagFilter({ page, user, isMobile, tags: ['Alpha', 'Beta'] });
  await expect(recipeList.getByRole('link')).toHaveText([`${user}Rho`]);

  // clear filter
  await page.reload(); // The clear button is disabled without refreshing first. This doesn't seem to a bug in actual testing.
  await page.getByRole('button', { name: 'Tag filter' }).click();
  const dialog = page.getByRole('dialog', { name: 'Tags' });
  await dialog.getByRole('button', { name: 'Clear' }).click();

  await expect(recipeList.getByRole('link')).toHaveText([`${user}Rho`, `${user}RhoTwo`]);
});