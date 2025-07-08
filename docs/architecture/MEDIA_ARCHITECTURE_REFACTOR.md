# Media Management Architecture Refactoring Plan

**Date:** July 7, 2025

**Author:** Gemini

## 1. Problem Statement

The current media management system has a flawed architecture. The `mediaController` is responsible for both file operations (uploading, deleting) and business logic (permission checks, database queries). This violates the Single Responsibility Principle, increases coupling between components, and makes the system difficult to test and maintain.

This architectural flaw is the root cause of the recent build failures related to `DuckDBConnection` and `prepare` methods.

## 2. Proposed Solution: Separation of Concerns & Dedicated Media Table

We will refactor the architecture to achieve a clear Separation of Concerns and introduce a dedicated `event_media` table to manage all media associated with an event. This is a robust, scalable, and professional solution.

### 2.1. Database Schema Redesign

-   **Create a New `event_media` Table:** This table will store each media item as its own record, providing a flexible "media pack" for each event.
    -   `id` (Primary Key)
    -   `event_id` (A foreign key linking back to the `events` table)
    -   `source_type` (An enum: `'upload'` or `'url'` to support both uploaded files and external links)
    -   `file_path_or_url` (A text field to store either the R2 file path or the external image URL)
    -   `type` (An enum: `'image'` or `'document'`)
    -   `mime_type` (e.g., `'image/jpeg'`)
    -   `uploaded_by` (A foreign key to the `users` table, nullable for URL-based images)
    -   `sort_order` (To allow for custom ordering in a gallery)
    -   `created_at`, `updated_at`
-   **Remove `images` from the `events` Table:** The `images` column will be completely removed from the `events` table.

### 2.2. `eventController` Responsibilities:

-   **Authorization Middleware:** The `eventController` will export new middleware functions (e.g., `canUploadEventMedia`, `canDeleteEventMedia`).
-   **Database Queries:** These middleware functions will contain all the necessary database queries to fetch event data.
-   **Permission Checks:** They will perform all authorization logic (e.g., is the user an admin? Is the user the event organizer?).
-   **Data Injection:** If authorization is successful, the middleware will attach the relevant `event` object to the Express `req` object (e.g., `req.event = eventData`) before passing control to the next handler.

### 2.3. `mediaController` Responsibilities:

-   **Simplified Logic:** The `mediaController` will be stripped of all database queries and permission checks.
-   **Smart Media Handling:** The `POST /api/media/events/:eventId/images` endpoint will be enhanced to handle both file uploads and external URLs:
    -   If the request contains a file upload, it will create a new row in `event_media` with `source_type = 'upload'`.
    -   If the request contains a JSON body with an `imageUrl`, it will create a new row with `source_type = 'url'`.
-   **Trust in Middleware:** It will assume that if its functions are executed, the preceding middleware has already authorized the user.

## 3. Implementation Plan: Zero-Downtime Rolling Deployment

To ensure the application remains available and consistent throughout the refactoring, we will follow a professional, multi-phase rolling deployment strategy.

### Phase 1: Additive & Backward-Compatible Changes

1.  **Create an Additive Migration:** Create a new database migration that *only adds* the new `event_media` table. It will **not** drop the old `images` column.
2.  **Update Application Code (`v2`):**
    *   **Write to Both:** When creating or updating an event's media, the new code will write to *both* the new `event_media` table *and* the old `events.images` column.
    *   **Read from New, Fallback to Old:** When reading event media, the new code will prioritize the `event_media` table but will fall back to reading from `events.images` if the new table has no data for that event.
3.  **Deploy `v2` Code:** Deploy this new version of the application. At this point, both the old code (`v1`) and new code (`v2`) can run simultaneously against the same database without errors.

---

### Phase 2: Data Migration (Backfill)

1.  **Run a Data Migration Script:** After `v2` is fully deployed and stable, run a one-time script. This script will read all the data from the old `events.images` column and populate the new `event_media` table for all existing events.
2.  **Verify Data:** Once this is complete, all data will exist in both the old and new locations, ensuring consistency.

---

### Phase 3: Deprecate the Old Column

1.  **Update Application Code (`v3`):** Create a new version of the code that *only* reads from and writes to the new `event_media` table. All logic related to the old `events.images` column will be removed.
2.  **Deploy `v3` Code:** Deploy this new version, which replaces all `v2` instances. The application now exclusively uses the new, superior schema.

---

### Phase 4: Final Cleanup

1.  **Create a Final, Destructive Migration:** Now that no running code depends on the old `events.images` column, it is finally safe to drop it. Create a new, simple migration that just contains `ALTER TABLE events DROP COLUMN images;`.
2.  **Run the Final Migration:** This cleans up the database schema, completing the process with zero downtime.

## 4. Benefits

-   **Resolves Build Errors:** This refactoring will eliminate the database-related TypeScript errors that are currently breaking the build.
-   **Flexible Media Support:** The system will now support both uploaded images and external image URLs.
-   **Improved Architecture:** Adheres to the Single Responsibility Principle.
-   **Reduced Coupling:** The `mediaController` will no longer be tightly coupled to the event data model.
-   **Enhanced Testability:** The `mediaController` can be unit-tested without needing to mock the database.
-   **Increased Scalability:** The system will be easier to extend and maintain in the future.
