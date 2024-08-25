import { APIRequestContext, expect } from '@playwright/test';

export async function createTag({
  user,
  name,
  request,
}: {
  user: string;
  name: string;
  request: APIRequestContext;
}) {
  const response = await request.post('/api/v1/tags', {
    data: { name: `${user}${name}` },
  });

  expect(response.ok()).toBeTruthy();
  return (await response.json()).data;
}

export async function createRecipe({
  user,
  request,
  recipe,
}: {
  user: string;
  request: APIRequestContext;
  recipe: {
    title: string;
    notes?: string;
    ingredients?: { [title: string]: string[] };
    instructions?: { [title: string]: string[] };
    tagIDs?: string[];
  };
}) {
  const response = await request.post('/api/v1/recipes', {
    data: {
      title: `${user}${recipe.title}`,
      notes: recipe.notes,
      tag_ids: recipe.tagIDs ?? [],
      ingredients: Object.entries(recipe.ingredients ?? {}).map(([title, items]) => ({
        title,
        ingredients: items,
      })),
      instructions: Object.entries(recipe.instructions ?? {}).map(([title, items]) => ({
        title,
        instructions: items,
      })),
    },
  });

  expect(response.ok()).toBeTruthy();
  return (await response.json()).data;
}
