# Gemini Project Configuration

This file helps Gemini understand the HeSocial project structure and conventions.

## Project Overview

HeSocial is a high-end social event platform for affluent individuals (NT$5M+ income, NT$30M+ assets). It's designed to facilitate luxury events like private dinners, yacht parties, and art appreciation gatherings.

The project is a monorepo with the following structure:

-   **Backend:** Located in the `backend` directory. It's a Node.js/Express application written in TypeScript.
-   **Frontend:** Located in the `frontend` directory. It's a React application built with Vite and styled with Tailwind CSS.
-   **Database:** The project uses DuckDB as its primary database. The schema is defined in `database/duckdb-schema.sql`. For production, Cloudflare R2 is used to persist the DuckDB database file.

📖 **For a full overview, see [docs/PROJECT_OVERVIEW.md](docs/PROJECT_OVERVIEW.md)**

## Development Workflow

The project uses convenient root-level commands for most common tasks:

-   `npm run setup`: Install all dependencies for frontend and backend.
-   `npm run dev`: Start both frontend and backend development servers concurrently.
-   `npm run build`: Build both frontend and backend for production.
-   `npm run test`: Run all tests.

Alternatively, you can run commands in the individual `frontend` or `backend` directories.

### Test Accounts

-   **Admin:** `admin@hesocial.com` / `admin123`
-   **Test User:** `test.platinum@example.com` / `test123`

## Deployment Considerations

-   The application is hosted on Render.com. The backend is `hesocial-api` and the frontend is `hesocial-frontend`.
-   When checking production deployments, always verify the commit ID to ensure the correct version is deployed.
