/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_API_URL: string
  readonly VITE_GOOGLE_OAUTH_ENABLED: string
  readonly VITE_LINKEDIN_OAUTH_ENABLED: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}