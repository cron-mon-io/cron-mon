export enum ThemeName {
  Dark = 'dark',
  Light = 'light'
}

export enum ThemeIcon {
  ToDark = 'mdi-weather-night',
  ToLight = 'mdi-white-balance-sunny'
}

export interface Theme {
  name: ThemeName
  icon: ThemeIcon
}

export const THEMES: Record<string, Theme> = {
  [ThemeName.Dark]: {
    name: ThemeName.Dark,
    icon: ThemeIcon.ToLight
  },
  [ThemeName.Light]: {
    name: ThemeName.Light,
    icon: ThemeIcon.ToDark
  }
}

export function getThemeName(): ThemeName {
  const persisted = localStorage.getItem('theme')
  const themeName = persisted as ThemeName
  return persisted === null || ![ThemeName.Dark, ThemeName.Light].includes(themeName)
    ? ThemeName.Dark
    : themeName
}

export function setThemeName(themeName: ThemeName): void {
  localStorage.setItem('theme', themeName)
}
