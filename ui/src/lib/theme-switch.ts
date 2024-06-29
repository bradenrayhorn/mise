export function setTheme(isDark: boolean) {
  document.documentElement.setAttribute('class', isDark ? 'dark' : '');
  localStorage.setItem('dark', isDark.toString());
}

export function isThemeDark() {
  return document.documentElement.getAttribute('class') === 'dark';
}

// self-contained function to be embedded in <head> of page
export function initTheme() {
  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
  const savedValue = localStorage.getItem('dark');
  const hasSavedValue = savedValue !== null;

  function setTheme(isDark: boolean) {
    document.documentElement.setAttribute('class', isDark ? 'dark' : '');
    localStorage.setItem('dark', isDark.toString());
  }

  setTheme(hasSavedValue ? savedValue === 'true' : mediaQuery.matches);
}
