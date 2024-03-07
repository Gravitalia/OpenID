import { isDevelopment } from "std-env";

export default defineNuxtConfig({
  app: {
    keepalive: true,
    head: {
      charset: "utf-8",
      viewport: "width=device-width,initial-scale=1",
      title: "Gravitalia Account",
      htmlAttrs: {
        lang: "en",
      },
      meta: [
        { property: "og:type", content: "website" },
        { property: "og:site_name", content: "Gravitalia Account" },
        { property: "og:title", content: "Gravitalia Account" },
        { property: "og:image", content: "/favicon.webp" },
        {
          name: "og:description",
          content: "One account, multiple services. Your Gravitalia Account links services, without them knowing everything about you.",
        },
        { name: "theme-color", content: "#8b5cf6" },
        { name: "robots", content: "index, follow" },
        { name: "twitter:card", content: "summary" },
        { name: "twitter:site", content: "@gravitalianews" },
        {
          name: "description",
          content: "One account, multiple services. Your Gravitalia Account links services, without them knowing everything about you.",
        },
      ],
      script: [
        {
          innerHTML: !isDevelopment
            ? '"serviceWorker"in navigator&&navigator.serviceWorker.register("/sw.js",{scope:"/"})'
            : "",
        },
      ],
      bodyAttrs: {
        class: "dark:bg-zinc-900 dark:text-white font-sans",
      },
    },
  },

  ssr: true,
  components: true,
  sourcemap: isDevelopment,

  modules: [
    "@pinia/nuxt",
    "@unocss/nuxt",
    "@nuxt/image",
    [
      "@nuxtjs/turnstile",
      {
        siteKey: "0x4AAAAAAABG7Pcx4-fniaty",
      },
    ],
    [
      "@nuxtjs/color-mode",
      {
        preference: "system",
        fallback: "light",
        hid: "color-script",
        globalName: "__NUXT_COLOR_MODE__",
        componentName: "ColorScheme",
        classPrefix: "",
        classSuffix: "",
        storageKey: "mode",
      },
    ],
    [
      "@nuxtjs/i18n",
      {
        defaultLocale: "en",
        strategy: "no_prefix",
        lazy: false,
        langDir: "locales",
        compilation: {
          strictMessage: false,
          escapeHtml: true,
        },
        detectBrowserLanguage: {
          useCookie: true,
          cookieKey: "locale",
          redirectOn: "root",
          fallbackLocale: "en",
          alwaysRedirect: true,
        },
        locales: [
          {
            code: "en",
            iso: "en-US",
            file: "en-US.json",
            name: "English",
          },
          {
            code: "fr",
            iso: "fr-FR",
            file: "fr-FR.json",
            name: "Français",
          },
        ],
        baseUrl: "https://account.gravitalia.com",
      },
    ],
    "~/modules/purge-comments",
  ],

  devtools: { enabled: true },
  runtimeConfig: {
    public: {
      API_URL: "http://localhost:1111",
    },
    email: "support@gravitalia.com",
  },

  routeRules: {
    // No JS.
    "/terms": { experimentalNoScripts: true },
    "/privacy": { experimentalNoScripts: true },
  },

  pinia: {
    storesDirs: ["./stores/**"],
  },

  experimental: {
    headNext: true,
    payloadExtraction: false,
    inlineSSRStyles: false,
    renderJsonPayloads: true,
    viewTransition: true,
  },

  vue: {
    defineModel: true,
    propsDestructure: true,
  },
});
