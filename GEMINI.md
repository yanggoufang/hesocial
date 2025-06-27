# Gemini Project Configuration

This file helps Gemini understand the HeSocial project structure and conventions.

## Project Overview

HeSocial is a social event platform with a monorepo structure.

- **Backend:** Located in the `backend` directory. It's a Node.js/Express application written in TypeScript.
- **Frontend:** Located in the `frontend` directory. It's a React application built with Vite and styled with Tailwind CSS.
- **Database:** The project uses DuckDB as its primary database. The schema is defined in `database/duckdb-schema.sql`. For production, we plan to use Cloudflare R2 to persist the DuckDB database file.

## Development Workflow

- The backend server can be started with `npm run dev` in the `backend` directory.
- The frontend development server can be started with `npm run dev` in the `frontend` directory.

## Deployment Considerations

- When checking production deployments, always verify the commit ID to ensure the correct version is deployed.
