import { error, redirect, type NumericRange } from '@sveltejs/kit';

const defaultError = 'Unknown error';

export const handleAPIError = async (res: Response, url: URL) => {
  const errorJson = await res.json().catch(async () => await res.text().catch(() => defaultError));
  const msg = errorJson?.error ?? defaultError;

  if (res.status >= 400 && res.status <= 599) {
    if (res.status === 401) {
      redirect(307, `/login/init?redirect_target=${url.pathname}`);
    }

    error(res.status as NumericRange<400, 599>, msg);
  }

  error(500, msg);
};
