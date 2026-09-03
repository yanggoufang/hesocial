# Gemini Frontend Configuration

This file provides specific instructions for interacting with the frontend of the HeSocial project.

## Technology Stack

-   **Language:** TypeScript
-   **Framework:** React 18 with Vite
-   **Styling:** Tailwind CSS
-   **State Management:** React Hooks / Context API
-   **Routing:** React Router DOM
-   **API Communication:** Axios
-   **Package Manager:** npm

## Project Structure

-   `src/components`: Reusable UI components (e.g., `Navbar.tsx`, `EventForm.tsx`).
-   `src/pages`: Top-level page components that correspond to routes (e.g., `HomePage.tsx`, `AdminDashboard.tsx`).
-   `src/hooks`: Custom React hooks for shared logic (e.g., `useAuth.tsx`).
-   `src/services`: Modules for making API calls to the backend (e.g., `eventService.ts`).
-   `src/types`: TypeScript type definitions.
-   `src/styles`: Global CSS and Tailwind configuration.

## Key Commands

-   **Install dependencies:** `npm install`
-   **Run development server:** `npm run dev` (typically on `http://localhost:3000`)
-   **Build for production:** `npm run build`
-   **Run tests:** `npm test`
-   **Lint files:** `npm run lint`
-   **Check types:** `npm run typecheck`
