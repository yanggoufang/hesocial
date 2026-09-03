# Gemini Backend Configuration

This file provides specific instructions for interacting with the backend of the HeSocial project.

## Technology Stack

-   **Language:** TypeScript
-   **Framework:** Node.js with Express
-   **Database:** DuckDB
-   **Storage:** Cloudflare R2 for production database persistence
-   **Package Manager:** npm

## Development Modes

The backend has two primary development modes:

-   **Full Mode (with R2):** `npm run dev`
    -   This is the primary development command. It runs the server with a connection to Cloudflare R2 for database backups and persistence, mirroring the production setup.
-   **DuckDB-Only Mode:** `npm run dev:duckdb`
    -   This command runs the server using only a local DuckDB file (`hesocial.duckdb`). It's faster for local development and does not require R2 credentials.

## Key Commands

-   **Install dependencies:** `npm install`
-   **Run development server (Full Mode):** `npm run dev`
-   **Run development server (DuckDB-Only):** `npm run dev:duckdb`
-   **Build for production:** `npm run build`
-   **Run tests:** `npm test`
-   **Run migrations:** `npm run migrate`
-   **Seed database:** `npm run seed`

## Code Style

-   Follow existing coding style and conventions.
-   Use Prettier for code formatting.
-   Use ESLint for linting.
-   Adhere to strict TypeScript rules.
