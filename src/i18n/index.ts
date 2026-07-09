import i18next from "i18next";
import { initReactI18next } from "react-i18next";

import setupEn from "./locales/en/setup.json";
import setupEs from "./locales/es/setup.json";

export const languages = ["en", "es"] as const;
export type Language = (typeof languages)[number];

export const languageNames = {
  en: "English",
  es: "Español",
} satisfies Record<Language, string>;

const languageStorageKey = "maresme-mc-setup-language";

function isLanguage(value: string | null | undefined): value is Language {
  return languages.some((language) => language === value);
}

function getInitialLanguage() {
  if (typeof window === "undefined") {
    return "en";
  }

  const browserLanguages = [
    ...(window.navigator.languages ?? []),
    window.navigator.language,
  ];
  const browserLanguage = browserLanguages
    .map((language) => language.split("-")[0])
    .find(isLanguage);

  if (browserLanguage) {
    return browserLanguage;
  }

  const storedLanguage = window.localStorage.getItem(languageStorageKey);
  return isLanguage(storedLanguage) ? storedLanguage : "en";
}

void i18next.use(initReactI18next).init({
  fallbackLng: "en",
  interpolation: {
    escapeValue: false,
  },
  lng: getInitialLanguage(),
  ns: ["setup"],
  defaultNS: "setup",
  resources: {
    en: {
      setup: setupEn,
    },
    es: {
      setup: setupEs,
    },
  },
  supportedLngs: languages,
});

i18next.on("languageChanged", (language) => {
  if (!isLanguage(language) || typeof document === "undefined") {
    return;
  }

  document.documentElement.lang = language;
  window.localStorage.setItem(languageStorageKey, language);
});

if (typeof document !== "undefined") {
  document.documentElement.lang = i18next.language;
}
