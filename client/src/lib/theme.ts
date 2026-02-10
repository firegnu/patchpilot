import type { ThemeMode } from '../types/app';

type ResolvedTheme = 'light' | 'dark';

const getResolvedTheme = (mode: ThemeMode, media: MediaQueryList): ResolvedTheme =>
  mode === 'system' ? (media.matches ? 'dark' : 'light') : mode;

const applyResolvedTheme = (theme: ResolvedTheme): void => {
  document.documentElement.setAttribute('data-theme', theme);
  document.documentElement.style.colorScheme = theme;
};

export const applyThemeMode = (mode: ThemeMode): (() => void) => {
  if (typeof window === 'undefined') {
    return () => undefined;
  }

  const media = window.matchMedia('(prefers-color-scheme: dark)');
  const syncTheme = (): void => applyResolvedTheme(getResolvedTheme(mode, media));
  syncTheme();

  if (mode !== 'system') {
    return () => undefined;
  }

  const onChange = (): void => syncTheme();
  media.addEventListener('change', onChange);
  return () => media.removeEventListener('change', onChange);
};
