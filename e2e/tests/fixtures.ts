import { expect, test as base } from '@playwright/test';
import { uid } from 'uid';

export const test = base.extend<{ login: string }, {}>({
  login: [
    async ({ page, request }, use) => {
      const username = uid();
      const initResponse = await request.get('/auth/init');
      const loginResponse = await request.get(`${initResponse.url()}&username=${username}`);

      expect(loginResponse.ok()).toBeTruthy();

      const state = await request.storageState();
      page.context().addCookies(state.cookies);

      await use(username);
    },
    { scope: 'test' },
  ],
});
