import { expect } from '@playwright/test';
import { createTag } from '@tests/apis';
import { test } from '@tests/fixtures';
import { assertRecipeView, fillRecipeForm } from '@tests/helpers';

test('can close page', async ({ login: _, page }) => {
  await page.goto('/recipes');

  await page.getByRole('button', { name: 'More options' }).click();
  await page.getByRole('menuitem', { name: 'Add recipe', exact: true }).click();

  await page.getByRole('link', { name: 'Cancel' }).click();

  await expect(page.getByRole('textbox', { name: 'Search' })).toBeVisible();
});

test('can add new simple recipe', async ({ login: user, page, request }) => {
  await createTag({ user, request, name: 'Good' });
  await createTag({ user, request, name: 'Quick' });

  // go to add form
  await page.goto('/recipes/new');

  // fill out form
  await fillRecipeForm({
    user,
    page,
    recipe: {
      title: 'Dinner',
      image: 'fixtures/recipe-image.jpg',
      notes: 'Tastes good!',
      ingredients: { '': ['Apple sauce', 'Corn starch'] },
      instructions: { '': ['Stir the apple sauce and corn starch.', 'Bake at 350.'] },
      tags: ['Good', 'Quick'],
    },
  });

  await page.getByRole('button', { name: 'Save' }).click();

  // back to recipes page
  await page.getByRole('textbox', { name: 'Search' }).fill(`${user}Dinner`);
  await page.getByRole('textbox', { name: 'Search' }).press('Enter');

  await page.getByRole('link', { name: `${user}Dinner` }).click();

  // verify details
  await assertRecipeView({
    user,
    page,
    recipe: {
      title: 'Dinner',
      image: true,
      notes: 'Tastes good!',
      ingredients: { Ingredients: ['Apple sauce', 'Corn starch'] },
      instructions: { Instructions: ['Stir the apple sauce and corn starch.', 'Bake at 350.'] },
      tags: ['Good', 'Quick'],
    },
  });
});

test('can add complex recipe', async ({ login: user, page, request }) => {
  await createTag({ user, request, name: 'Good' });

  // go to add form
  await page.goto('/recipes/new');

  // fill out form
  await fillRecipeForm({
    user,
    page,
    recipe: {
      title: 'Glazed Bread',
      image: 'fixtures/recipe-image.jpg',
      notes: 'Tastes good!',
      ingredients: { Glaze: ['Milk', 'Water'], Bread: ['Flour', 'Eggs'] },
      instructions: { Glaze: ['Stir', 'Heat'], Bread: ['Stir', 'Bake'] },
      tags: ['Good'],
    },
  });

  await page.getByRole('button', { name: 'Save' }).click();

  // back to recipes page
  await page.getByRole('textbox', { name: 'Search' }).fill(`${user}Glazed Bread`);
  await page.getByRole('textbox', { name: 'Search' }).press('Enter');

  await page.getByRole('link', { name: `${user}Glazed Bread` }).click();

  // verify details
  await assertRecipeView({
    user,
    page,
    recipe: {
      title: 'Glazed Bread',
      image: true,
      notes: 'Tastes good!',
      ingredients: { Glaze: ['Milk', 'Water'], Bread: ['Flour', 'Eggs'] },
      instructions: { Glaze: ['Stir', 'Heat'], Bread: ['Stir', 'Bake'] },
      tags: ['Good'],
    },
  });
});

test('can add recipe with minimal details', async ({ login: user, page, request }) => {
  await createTag({ user, request, name: 'Good' });

  // go to add form
  await page.goto('/recipes/new');

  // fill out form
  await fillRecipeForm({
    user,
    page,
    recipe: {
      title: 'Glazed Bread',
      ingredients: {},
      instructions: {},
      tags: ['Good'],
    },
  });

  // remove tag
  await page.getByRole('button', { name: `Delete tag ${user}Good` }).click();

  await page.getByRole('button', { name: 'Save' }).click();

  // back to recipes page
  await page.getByRole('textbox', { name: 'Search' }).fill(`${user}Glazed Bread`);
  await page.getByRole('textbox', { name: 'Search' }).press('Enter');

  await page.getByRole('link', { name: `${user}Glazed Bread` }).click();

  // verify details
  await assertRecipeView({
    user,
    page,
    recipe: {
      title: 'Glazed Bread',
      image: false,
      ingredients: {},
      instructions: {},
      tags: [],
    },
  });
});
