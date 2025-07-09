# Media Management Architecture Refactoring Plan

**Date:** July 7, 2025

**Author:** Gemini

## 1. Problem Statement

The current media management system has a flawed architecture. The `mediaController` is responsible for both file operations (uploading, deleting) and business logic (permission checks, database queries). This violates the Single Responsibility Principle, increases coupling between components, and makes the system difficult to test and maintain.

This architectural flaw is the root cause of the recent build failures related to `DuckDBConnection` and `prepare` methods.

## 2. Proposed Solution: Separation of Concerns & Dedicated Media Table

We will refactor the architecture to achieve a clear Separation of Concerns and introduce a dedicated `event_media` table to manage all media associated with an event. This is a robust, scalable, and professional solution.

### 2.1. Database Schema Redesign

**Target Schema (Final):**
```sql
event_media (
    id VARCHAR PRIMARY KEY,
    event_id VARCHAR NOT NULL,
    source_type VARCHAR(10) CHECK (source_type IN ('upload', 'url')),
    file_path_or_url TEXT NOT NULL,
    type VARCHAR(20) CHECK (type IN ('image', 'document')),
    mime_type VARCHAR(100),
    original_filename VARCHAR,
    file_size BIGINT,
    thumbnail_path TEXT,
    sort_order INTEGER DEFAULT 0,
    uploaded_by VARCHAR,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
)
```

**Current Schema (Migration 004):**
```sql
event_media (
    id VARCHAR PRIMARY KEY,
    event_id VARCHAR NOT NULL,
    type VARCHAR(20) CHECK (type IN ('image', 'document')), ✅
    file_path VARCHAR NOT NULL,                              ❌ → file_path_or_url
    thumbnail_path TEXT,                                     ✅ Extra
    original_filename VARCHAR NOT NULL,                      ✅ Extra  
    file_size BIGINT NOT NULL,                              ✅ Extra
    mime_type VARCHAR(100) NOT NULL,                        ✅
    uploaded_by VARCHAR NOT NULL,                           ✅
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,         ✅
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP          ✅
    -- MISSING: source_type, sort_order
)
```

**Rolling Plan:**
-   **Phase 1:** Add missing columns via Migration 005 (additive only)
-   **Phase 2-3:** Gradually migrate from `events.images` to `event_media` 
-   **Phase 4:** Remove `events.images` column completely

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

### Phase 1: Additive & Backward-Compatible Changes ✅ **IN PROGRESS**

**Current Status: 60% Complete**

✅ **Completed:**
1.  **Create Basic `event_media` Table:** Migration 004 created initial `event_media` table with core fields
2.  **Basic Media Upload:** File upload functionality working through MediaService + R2 storage
3.  **Permission System:** Admin/organizer authorization partially implemented

❌ **Still Required:**
1.  **Add Missing Columns:** Create migration 005 to add `source_type` and `sort_order` columns
2.  **Implement Dual-Write Logic:** Update MediaService to write to *both* `event_media` table *and* `events.images` column
3.  **Implement Dual-Read Logic:** Update controllers to read from `event_media` with fallback to `events.images`
4.  **Add URL Media Support:** Extend media endpoints to handle external URLs alongside file uploads
5.  **Deploy v2 Code:** Once dual-read/write implemented, both old and new code can coexist

---

### Phase 2: Data Migration (Backfill) ⏳ **PLANNED**

**Target: After Phase 1 Complete**

1.  **Create Backfill Script:** Build data migration utility to read from `events.images` array and populate `event_media` table
2.  **Run Migration:** Execute backfill for all existing events with `images` data  
3.  **Verify Consistency:** Ensure all data exists in both locations with matching counts
4.  **Performance Check:** Validate system performance with dual-storage approach

---

### Phase 3: Deprecate the Old Column ⏳ **FUTURE**

**Target: After Phase 2 Complete + 2 weeks stability**

1.  **Update Application Code (`v3`):** Remove all `events.images` read/write logic, use only `event_media` table
2.  **Update Controllers:** Simplify event controllers to only query `event_media` 
3.  **Update MediaService:** Remove dual-write logic, write only to `event_media`
4.  **Deploy `v3` Code:** Replace all instances with new single-source logic
5.  **Monitor:** Ensure no errors or missing data after deployment

---

### Phase 4: Final Cleanup ⏳ **FINAL**

**Target: After Phase 3 + 1 month production stability**

1.  **Create Migration 006:** `ALTER TABLE events DROP COLUMN images`
2.  **Final Cleanup:** Remove any remaining legacy code references
3.  **Documentation Update:** Update API docs and schema documentation
4.  **Performance Baseline:** Establish new performance metrics without legacy column

## 4. Benefits

-   **Resolves Build Errors:** This refactoring will eliminate the database-related TypeScript errors that are currently breaking the build.
-   **Flexible Media Support:** The system will now support both uploaded images and external image URLs.
-   **Improved Architecture:** Adheres to the Single Responsibility Principle.
-   **Reduced Coupling:** The `mediaController` will no longer be tightly coupled to the event data model.
-   **Enhanced Testability:** The `mediaController` can be unit-tested without needing to mock the database.
-   **Increased Scalability:** The system will be easier to extend and maintain in the future.
