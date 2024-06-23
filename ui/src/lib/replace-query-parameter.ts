export function setQueryParameters(
  params: URLSearchParams,
  nextParams: { [key: string]: string },
): URLSearchParams {
  const next = new URLSearchParams(params);
  Object.entries(nextParams).forEach(([k, v]) => {
    if (v.length === 0) {
      next.delete(k);
    } else {
      next.set(k, v);
    }
  });

  return next;
}
