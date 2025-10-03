# Documentation Drift Monitoring Summary

**Date**: 2025-10-03
**Status**: 🚨 Issues Detected

## Summary
- **Issues Found**: 1
- **Warnings**: 0
- **Last Check**: 2025-10-03T04:18:20.184Z


## Issues Found

### 🚨 Errors (1)

1. **validation_failure**: Documentation validation failed to run
   - Command failed: npm run validate:docs
⚠️  123 undocumented routes found:
⚠️    POST /api//backup (admin.ts)
⚠️    POST /api//restore (admin.ts)
⚠️    GET /api//backups (admin.ts)
⚠️    POST /api//cleanup (admin.ts)
⚠️    GET /api//database/stats (admin.ts)
⚠️    POST /api//periodic-backup/start (admin.ts)
⚠️    POST /api//periodic-backup/stop (admin.ts)
⚠️    GET /api//periodic-backup/status (admin.ts)
⚠️    POST /api//database/checkpoint (admin.ts)
⚠️    GET /api//visitors (analyticsRoutes.ts)
⚠️    GET /api//visitors/daily (analyticsRoutes.ts)
⚠️    GET /api//pages/popular (analyticsRoutes.ts)
⚠️    GET /api//conversion (analyticsRoutes.ts)
⚠️    GET /api//visitors/:visitorId (analyticsRoutes.ts)
⚠️    GET /api//events/overview (analyticsRoutes.ts)
⚠️    GET /api//events/performance (analyticsRoutes.ts)
⚠️    GET /api//events/engagement (analyticsRoutes.ts)
⚠️    POST /api//events/track (analyticsRoutes.ts)
⚠️    GET /api//revenue/events (analyticsRoutes.ts)
⚠️    GET /api//engagement/members (analyticsRoutes.ts)
⚠️    GET /api//events/:id/performance (analyticsRoutes.ts)
⚠️    POST /api//register (authRoutes.ts)
⚠️    POST /api//login (authRoutes.ts)
⚠️    GET /api//profile (authRoutes.ts)
⚠️    PUT /api//profile (authRoutes.ts)
⚠️    POST /api//refresh (authRoutes.ts)
⚠️    GET /api//google (authRoutes.ts)
⚠️    GET /api//google/callback (authRoutes.ts)
⚠️    GET /api//linkedin (authRoutes.ts)
⚠️    GET /api//linkedin/callback (authRoutes.ts)
⚠️    POST /api//logout (authRoutes.ts)
⚠️    GET /api//validate (authRoutes.ts)
⚠️    GET /api// (categoryManagement.ts)
⚠️    GET /api//:id (categoryManagement.ts)
⚠️    POST /api// (categoryManagement.ts)
⚠️    PUT /api//:id (categoryManagement.ts)
⚠️    DELETE /api//:id (categoryManagement.ts)
⚠️    GET /api//status (deploymentRoutes.ts)
⚠️    GET /api//health (deploymentRoutes.ts)
⚠️    POST /api//deploy-visitor-tracking (deploymentRoutes.ts)
⚠️    POST /api//rollback (deploymentRoutes.ts)
⚠️    POST /api//emergency-apply-visitor-tracking (deploymentRoutes.ts)
⚠️    GET /api//migration-plans (deploymentRoutes.ts)
⚠️    POST /api//test-connection (deploymentRoutes.ts)
⚠️    POST /api//apply-visitor-tracking (emergencyRoutes.ts)
⚠️    GET /api//test-visitor-tracking (emergencyRoutes.ts)
⚠️    POST /api//fix-analytics-queries (emergencyRoutes.ts)
⚠️    GET /api//test-analytics (emergencyRoutes.ts)
⚠️    GET /api// (eventManagement.ts)
⚠️    GET /api//:id (eventManagement.ts)
⚠️    POST /api// (eventManagement.ts)
⚠️    PUT /api//:id (eventManagement.ts)
⚠️    DELETE /api//:id (eventManagement.ts)
⚠️    POST /api//:id/publish (eventManagement.ts)
⚠️    POST /api//:id/approve (eventManagement.ts)
⚠️    GET /api// (eventRoutes.ts)
⚠️    GET /api//categories (eventRoutes.ts)
⚠️    GET /api//venues (eventRoutes.ts)
⚠️    GET /api//:id (eventRoutes.ts)
⚠️    POST /api// (eventRoutes.ts)
⚠️    GET /api//database (health.ts)
⚠️    GET /api//r2-sync (health.ts)
⚠️    GET /api//full (health.ts)
⚠️    GET /api//events (index.ts)
⚠️    GET /api//events/:id (index.ts)
⚠️    GET /api//categories (index.ts)
⚠️    GET /api//venues (index.ts)
⚠️    GET /api//media/events/:eventId (index.ts)
⚠️    GET /api// (index.ts)
⚠️    GET /api//test (index.ts)
⚠️    GET /api//test (index.ts)
⚠️    GET /api// (index.ts)
⚠️    POST /api//events/:eventId/images (mediaRoutes.ts)
⚠️    POST /api//events/:eventId/documents (mediaRoutes.ts)
⚠️    GET /api//events/:eventId (mediaRoutes.ts)
⚠️    POST /api//venues/:venueId/images (mediaRoutes.ts)
⚠️    GET /api//venues/:venueId (mediaRoutes.ts)
⚠️    DELETE /api//:mediaId (mediaRoutes.ts)
⚠️    GET /api//download/:mediaId (mediaRoutes.ts)
⚠️    GET /api//stats (mediaRoutes.ts)
⚠️    POST /api//cleanup (mediaRoutes.ts)
⚠️    GET /api//events/:eventId/participants (participants.ts)
⚠️    GET /api//events/:eventId/participant-access (participants.ts)
⚠️    GET /api//events/:eventId/participants/:participantId (participants.ts)
⚠️    POST /api//events/:eventId/participants/:participantId/contact (participants.ts)
⚠️    PUT /api//events/:eventId/privacy-settings (participants.ts)
⚠️    GET /api//events/:eventId/privacy-settings (participants.ts)
⚠️    GET /api//:width/:height (placeholderRoutes.ts)
⚠️    GET /api//stats/:eventId (registrationRoutes.ts)
⚠️    GET /api//user (registrationRoutes.ts)
⚠️    POST /api// (registrationRoutes.ts)
⚠️    GET /api//:id (registrationRoutes.ts)
⚠️    PUT /api//:id (registrationRoutes.ts)
⚠️    DELETE /api//:id (registrationRoutes.ts)
⚠️    GET /api//leads (salesManagement.ts)
⚠️    GET /api//leads/:id (salesManagement.ts)
⚠️    POST /api//leads (salesManagement.ts)
⚠️    PUT /api//leads/:id (salesManagement.ts)
⚠️    DELETE /api//leads/:id (salesManagement.ts)
⚠️    GET /api//opportunities (salesManagement.ts)
⚠️    POST /api//opportunities (salesManagement.ts)
⚠️    PUT /api//opportunities/:id (salesManagement.ts)
⚠️    GET /api//activities (salesManagement.ts)
⚠️    POST /api//activities (salesManagement.ts)
⚠️    GET /api//metrics (salesManagement.ts)
⚠️    GET /api//pipeline/stages (salesManagement.ts)
⚠️    GET /api//team (salesManagement.ts)
⚠️    GET /api//health (systemHealthRoutes.ts)
⚠️    GET /api//health/detailed (systemHealthRoutes.ts)
⚠️    GET /api//metrics (systemHealthRoutes.ts)
⚠️    GET /api//diagnostics (systemHealthRoutes.ts)
⚠️    GET /api// (userManagement.ts)
⚠️    GET /api//:id (userManagement.ts)
⚠️    PUT /api//:id (userManagement.ts)
⚠️    DELETE /api//:id (userManagement.ts)
⚠️    POST /api//:id/verify (userManagement.ts)
⚠️    POST /api//:id/role (userManagement.ts)
⚠️    GET /api//stats/overview (userManagement.ts)
⚠️    GET /api// (venueManagement.ts)
⚠️    GET /api//:id (venueManagement.ts)
⚠️    POST /api// (venueManagement.ts)
⚠️    PUT /api//:id (venueManagement.ts)
⚠️    DELETE /api//:id (venueManagement.ts)
❌ Missing required database tables: event_registrations
⚠️  Version inconsistency detected:
⚠️    CLAUDE.md: Phase 1
⚠️    docs/systems/DEVELOPMENT_STATUS.md: Phase 11


## Recommendations

1. 🔴 **Fix documentation accuracy issues** (high priority)
   Run npm run validate:docs to identify and fix issues
   **Command**: `npm run validate:docs`

2. 🟢 **Run regular validation** (low priority)
   Run documentation validation regularly to prevent drift
   **Command**: `npm run validate:docs`
   **Frequency**: Weekly

3. 🟢 **Update API documentation** (low priority)
   Regenerate API documentation after route changes
   **Command**: `npm run generate:api-docs`
   **Frequency**: After route changes

## Quick Fix Commands

```bash
# Validate documentation accuracy
npm run validate:docs

# Generate API documentation
npm run generate:api-docs

# Run all validations
npm run validate:all
``

---
*This report was automatically generated by the documentation drift prevention system*
*Last updated: 2025-10-03T04:18:20.185Z
