# Gemini Project Configuration

This file helps Gemini understand the HeSocial project structure and conventions.

## Project Overview

HeSocial is a social event platform with a monorepo structure.

- **Backend:** Located in the `backend` directory. It's a Node.js/Express application written in TypeScript.
- **Frontend:** Located in the `frontend` directory. It's a React application built with Vite and styled with Tailwind CSS.
- **Database:** The primary database schemas are in the `database` directory. The project uses PostgreSQL, but also has a DuckDB implementation for local development.

## Development Workflow

- The backend server can be started with `npm run dev` in the `backend` directory.
- The frontend development server can be started with `npm run dev` in the `frontend` directory.
