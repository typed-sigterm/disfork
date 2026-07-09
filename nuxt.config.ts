if (!import.meta.env.DISFORK_GITHUB_APP_CLIENT_ID)
  throw new Error('GITHUB_APP_CLIENT_ID is not set in environment variables');

export default defineNuxtConfig({
  app: {
    head: {
      htmlAttrs: { lang: 'en' },
      title: 'DisFork',
    },
  },

  modules: [
    '@nuxt/ui',
  ],

  css: ['~/main.css'],

  ssr: false,

  compatibilityDate: '2026-07-09',

  runtimeConfig: {
    githubAppClientId: import.meta.env.DISFORK_GITHUB_APP_CLIENT_ID,
  },

  devServer: {
    port: 9908,
  },

  fonts: {
    provider: 'bunny',
    providers: {
      google: false,
      googleicons: false,
    },
  },

  icon: {
    clientBundle: {
      scan: true,
    },
  },
});
