# API Documentation - HeSocial Platform

**Generated**: 2025-10-03T04:15:57.651Z
**Total Endpoints**: 114
**Route Files**: 18

---

## 🔐 **Authentication**

All API endpoints require authentication unless specified as **Public**.

### **Authentication Methods**
- **JWT Token**: Include in `Authorization: Bearer <token>` header
- **Development Token**: Use `dev-token-12345` for development testing
- **Google OAuth 2.0**: Redirect-based authentication

### **Role-Based Access**
- **Public**: No authentication required
- **User**: Authenticated user access
- **Admin**: Admin role required
- **Super Admin**: Super admin role required

---

## ⚙️ **Administration**

### **GET** /api//backups
**Access**: Public
**Description**: GET /api/admin/backups
List available backups in R2
Requires admin authentication
**Example Request**:
```bash
curl -X GET /api//backups \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//database/stats
**Access**: Public
**Description**: GET /api/admin/database/stats
Get database statistics and information
**Example Request**:
```bash
curl -X GET /api//database/stats \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//periodic-backup/status
**Access**: Public
**Description**: GET /api/admin/periodic-backup/status
Get the status of periodic backups
Requires admin authentication
**Example Request**:
```bash
curl -X GET /api//periodic-backup/status \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **POST** /api//backup
**Access**: Public
**Description**: POST /api/admin/backup
Create a manual backup to R2
Requires admin authentication
**Example Request**:
```bash
curl -X POST /api//backup \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``


### **POST** /api//cleanup
**Access**: Public
**Description**: POST /api/admin/cleanup
Clean up old backups based on retention policy
Requires admin authentication
**Example Request**:
```bash
curl -X POST /api//cleanup \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``


### **POST** /api//database/checkpoint
**Access**: Public
**Description**: POST /api/admin/database/checkpoint
Manually trigger database checkpoint/flush
**Example Request**:
```bash
curl -X POST /api//database/checkpoint \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``


### **POST** /api//periodic-backup/start
**Access**: Public
**Description**: POST /api/admin/periodic-backup/start
Start periodic backups
Requires admin authentication
**Example Request**:
```bash
curl -X POST /api//periodic-backup/start \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``


### **POST** /api//periodic-backup/stop
**Access**: Public
**Description**: POST /api/admin/periodic-backup/stop
Stop periodic backups
Requires admin authentication
**Example Request**:
```bash
curl -X POST /api//periodic-backup/stop \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``


### **POST** /api//restore
**Access**: Public
**Description**: POST /api/admin/restore
Restore database from R2 backup
Requires super admin authentication
**Example Request**:
```bash
curl -X POST /api//restore \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``



---

## 🔐 **Authentication**

### **GET** /api//google
**Access**: Public
**Example Request**:
```bash
curl -X GET /api//google \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//google/callback
**Access**: Public
**Example Request**:
```bash
curl -X GET /api//google/callback \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//linkedin
**Access**: Public
**Example Request**:
```bash
curl -X GET /api//linkedin \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//linkedin/callback
**Access**: Public
**Example Request**:
```bash
curl -X GET /api//linkedin/callback \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//profile
**Access**: Public
**Example Request**:
```bash
curl -X GET /api//profile \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//validate
**Access**: Public
**Example Request**:
```bash
curl -X GET /api//validate \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **POST** /api//login
**Access**: Public
**Example Request**:
```bash
curl -X POST /api//login \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``


### **POST** /api//logout
**Access**: Public
**Example Request**:
```bash
curl -X POST /api//logout \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``


### **POST** /api//refresh
**Access**: Public
**Example Request**:
```bash
curl -X POST /api//refresh \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``


### **POST** /api//register
**Access**: Public
**Example Request**:
```bash
curl -X POST /api//register \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``


### **PUT** /api//profile
**Access**: Public


---

## 📊 **Business Intelligence**

### **GET** /api//conversion
**Access**: Public
**Description**: GET /api/analytics/conversion
Get conversion funnel analytics
**Example Request**:
```bash
curl -X GET /api//conversion \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//engagement/members
**Access**: Public
**Description**: GET /api/analytics/engagement/members
Get member engagement analytics
**Example Request**:
```bash
curl -X GET /api//engagement/members \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//events/:id/performance
**Access**: Public
**Description**: GET /api/analytics/events/:id/performance
Get individual event performance metrics
**Example Request**:
```bash
curl -X GET /api//events/:id/performance \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//events/engagement
**Access**: Public
**Description**: GET /api/analytics/events/engagement
Get event engagement metrics
**Example Request**:
```bash
curl -X GET /api//events/engagement \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//events/overview
**Access**: Public
**Description**: GET /api/analytics/events/overview
Get event analytics overview
**Example Request**:
```bash
curl -X GET /api//events/overview \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//events/performance
**Access**: Public
**Description**: GET /api/analytics/events/performance
Get event performance metrics
**Example Request**:
```bash
curl -X GET /api//events/performance \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//pages/popular
**Access**: Public
**Description**: GET /api/analytics/pages/popular
Get popular pages analytics
**Example Request**:
```bash
curl -X GET /api//pages/popular \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//revenue/events
**Access**: Public
**Description**: GET /api/analytics/revenue/events
Get revenue analytics for events
**Example Request**:
```bash
curl -X GET /api//revenue/events \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//visitors
**Access**: Public
**Description**: GET /api/analytics/visitors
Get visitor analytics overview
**Example Request**:
```bash
curl -X GET /api//visitors \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//visitors/:visitorId
**Access**: Public
**Description**: GET /api/analytics/visitors/:visitorId
Get detailed visitor journey
**Example Request**:
```bash
curl -X GET /api//visitors/:visitorId \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//visitors/daily
**Access**: Public
**Description**: GET /api/analytics/visitors/daily
Get daily visitor analytics
**Example Request**:
```bash
curl -X GET /api//visitors/daily \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **POST** /api//events/track
**Access**: Public
**Description**: POST /api/analytics/events/track
Track custom visitor events (for future Google Analytics integration)
**Example Request**:
```bash
curl -X POST /api//events/track \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``



---

## 🚀 **Deployment**

### **GET** /api//health
**Access**: Public
**Description**: GET /api/deployment/health
Get health status of all database environments
**Example Request**:
```bash
curl -X GET /api//health \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//migration-plans
**Access**: Public
**Description**: GET /api/deployment/migration-plans
Get available migration plans
**Example Request**:
```bash
curl -X GET /api//migration-plans \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//status
**Access**: Public
**Description**: GET /api/deployment/status
Get current deployment system status
**Example Request**:
```bash
curl -X GET /api//status \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **POST** /api//deploy-visitor-tracking
**Access**: Public
**Description**: POST /api/deployment/deploy-visitor-tracking
Deploy visitor tracking tables with zero downtime
**Example Request**:
```bash
curl -X POST /api//deploy-visitor-tracking \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``


### **POST** /api//emergency-apply-visitor-tracking
**Access**: Public
**Description**: POST /api/deployment/emergency-apply-visitor-tracking
Emergency: Apply visitor tracking schema directly (bypass blue-green)
Use only when blue-green system is not available
**Example Request**:
```bash
curl -X POST /api//emergency-apply-visitor-tracking \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``


### **POST** /api//rollback
**Access**: Public
**Description**: POST /api/deployment/rollback
Perform instant rollback to previous environment
**Example Request**:
```bash
curl -X POST /api//rollback \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``


### **POST** /api//test-connection
**Access**: Public
**Description**: POST /api/deployment/test-connection
Test database connections for both environments
**Example Request**:
```bash
curl -X POST /api//test-connection \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``



---

## 🚨 **Emergency Operations**

### **GET** /api//test-analytics
**Access**: Public
**Description**: GET /api/emergency/test-analytics
Test basic analytics queries
**Example Request**:
```bash
curl -X GET /api//test-analytics \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//test-visitor-tracking
**Access**: Public
**Description**: GET /api/emergency/test-visitor-tracking
Test if visitor tracking tables exist and are working
**Example Request**:
```bash
curl -X GET /api//test-visitor-tracking \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **POST** /api//apply-visitor-tracking
**Access**: Public
**Description**: POST /api/emergency/apply-visitor-tracking
Emergency: Apply visitor tracking schema directly
**Example Request**:
```bash
curl -X POST /api//apply-visitor-tracking \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``


### **POST** /api//fix-analytics-queries
**Access**: Public
**Description**: POST /api/emergency/fix-analytics-queries
Fix analytics queries for DuckDB compatibility
**Example Request**:
```bash
curl -X POST /api//fix-analytics-queries \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``



---

## 📅 **Event Management**

### **GET** /api//
**Access**: Public
**Description**: GET /api/events
Get all events with pagination, filtering, and search
Public endpoint with role-based data visibility
**Example Request**:
```bash
curl -X GET /api// \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//
**Access**: Public
**Example Request**:
```bash
curl -X GET /api// \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//:id
**Access**: Public
**Description**: GET /api/events/:id
Get event by ID with full details
**Example Request**:
```bash
curl -X GET /api//:id \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//:id
**Access**: Public
**Example Request**:
```bash
curl -X GET /api//:id \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//categories
**Access**: Public
**Example Request**:
```bash
curl -X GET /api//categories \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//venues
**Access**: Public
**Example Request**:
```bash
curl -X GET /api//venues \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **POST** /api//
**Access**: Public
**Description**: POST /api/events
Create new event (Admin only)
**Example Request**:
```bash
curl -X POST /api// \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``


### **POST** /api//
**Access**: Public
**Example Request**:
```bash
curl -X POST /api// \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``


### **POST** /api//:id/approve
**Access**: Public
**Description**: POST /api/events/:id/approve
Approve event for publishing (Admin only)
**Example Request**:
```bash
curl -X POST /api//:id/approve \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``


### **POST** /api//:id/publish
**Access**: Public
**Description**: POST /api/events/:id/publish
Publish event (Admin only)
**Example Request**:
```bash
curl -X POST /api//:id/publish \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``


### **PUT** /api//:id
**Access**: Public
**Description**: PUT /api/events/:id
Update event (Admin only)

### **DELETE** /api//:id
**Access**: Public
**Description**: DELETE /api/events/:id
Delete event (Super Admin only)


---

## 📡 **General**

### **GET** /api//
**Access**: Public
**Description**: GET /api/categories
Get all event categories
**Example Request**:
```bash
curl -X GET /api// \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//
**Access**: Public
**Description**: GET /api/users
Get all users with pagination and filtering
Requires admin authentication
**Example Request**:
```bash
curl -X GET /api// \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//
**Access**: Public
**Description**: GET /api/venues
Get all venues with pagination and filtering
**Example Request**:
```bash
curl -X GET /api// \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//:id
**Access**: Public
**Description**: GET /api/categories/:id
Get category by ID
**Example Request**:
```bash
curl -X GET /api//:id \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//:id
**Access**: Public
**Description**: GET /api/users/:id
Get user by ID
Requires admin authentication
**Example Request**:
```bash
curl -X GET /api//:id \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//:id
**Access**: Public
**Description**: GET /api/venues/:id
Get venue by ID
**Example Request**:
```bash
curl -X GET /api//:id \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//:width/:height
**Access**: Public
**Example Request**:
```bash
curl -X GET /api//:width/:height \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//stats/overview
**Access**: Public
**Description**: GET /api/users/stats
Get user statistics
Requires admin authentication
**Example Request**:
```bash
curl -X GET /api//stats/overview \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **POST** /api//
**Access**: Public
**Description**: POST /api/categories
Create new event category (Admin only)
**Example Request**:
```bash
curl -X POST /api// \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``


### **POST** /api//
**Access**: Public
**Description**: POST /api/venues
Create new venue (Admin only)
**Example Request**:
```bash
curl -X POST /api// \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``


### **POST** /api//:id/role
**Access**: Public
**Description**: POST /api/users/:id/role
Update user role
Requires super admin authentication
**Example Request**:
```bash
curl -X POST /api//:id/role \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``


### **POST** /api//:id/verify
**Access**: Public
**Description**: POST /api/users/:id/verify
Verify user account
Requires admin authentication
**Example Request**:
```bash
curl -X POST /api//:id/verify \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``


### **PUT** /api//:id
**Access**: Public
**Description**: PUT /api/categories/:id
Update event category (Admin only)

### **PUT** /api//:id
**Access**: Public
**Description**: PUT /api/users/:id
Update user information
Requires admin authentication

### **PUT** /api//:id
**Access**: Public
**Description**: PUT /api/venues/:id
Update venue (Admin only)

### **DELETE** /api//:id
**Access**: Public
**Description**: DELETE /api/categories/:id
Delete event category (Super Admin only)

### **DELETE** /api//:id
**Access**: Public
**Description**: DELETE /api/users/:id
Delete user (soft delete by setting a deleted flag or hard delete)
Requires super admin authentication

### **DELETE** /api//:id
**Access**: Public
**Description**: DELETE /api/venues/:id
Delete venue (Super Admin only)


---

## 📸 **Media Management**

### **GET** /api//download/:mediaId
**Access**: Public
**Description**: GET /api/media/download/:mediaId
Download a document (generates signed URL)
Requires authentication
**Example Request**:
```bash
curl -X GET /api//download/:mediaId \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//events/:eventId
**Access**: Public
**Description**: GET /api/media/events/:eventId
Get all media for an event
Public endpoint but respects event visibility
**Example Request**:
```bash
curl -X GET /api//events/:eventId \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//stats
**Access**: Public
**Description**: GET /api/media/stats
Get media statistics for admin dashboard
Requires admin authentication
**Example Request**:
```bash
curl -X GET /api//stats \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//venues/:venueId
**Access**: Public
**Description**: GET /api/media/venues/:venueId
Get all media for a venue
Public endpoint
**Example Request**:
```bash
curl -X GET /api//venues/:venueId \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **POST** /api//cleanup
**Access**: Public
**Description**: POST /api/media/cleanup
Clean up orphaned media files
Requires super admin authentication
**Example Request**:
```bash
curl -X POST /api//cleanup \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``


### **POST** /api//events/:eventId/documents
**Access**: Public
**Description**: POST /api/media/events/:eventId/documents
Upload documents for an event
Requires authentication and proper permissions
**Example Request**:
```bash
curl -X POST /api//events/:eventId/documents \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``


### **POST** /api//events/:eventId/images
**Access**: Public
**Description**: POST /api/media/events/:eventId/images
Upload images for an event
Requires authentication and proper permissions
**Example Request**:
```bash
curl -X POST /api//events/:eventId/images \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``


### **POST** /api//venues/:venueId/images
**Access**: Public
**Description**: POST /api/media/venues/:venueId/images
Upload images for a venue
Requires admin authentication
**Example Request**:
```bash
curl -X POST /api//venues/:venueId/images \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``


### **DELETE** /api//:mediaId
**Access**: Public
**Description**: DELETE /api/media/:mediaId
Delete a media file
Requires authentication and proper permissions


---

## 📝 **Registration System**

### **GET** /api//:id
**Access**: Public
**Example Request**:
```bash
curl -X GET /api//:id \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//stats/:eventId
**Access**: Public
**Example Request**:
```bash
curl -X GET /api//stats/:eventId \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//user
**Access**: Public
**Example Request**:
```bash
curl -X GET /api//user \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **POST** /api//
**Access**: Public
**Example Request**:
```bash
curl -X POST /api// \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``


### **PUT** /api//:id
**Access**: Public

### **DELETE** /api//:id
**Access**: Public


---

## 💰 **Sales Management**

### **GET** /api//activities
**Access**: Public
**Example Request**:
```bash
curl -X GET /api//activities \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//leads
**Access**: Public
**Example Request**:
```bash
curl -X GET /api//leads \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//leads/:id
**Access**: Public
**Example Request**:
```bash
curl -X GET /api//leads/:id \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//metrics
**Access**: Public
**Example Request**:
```bash
curl -X GET /api//metrics \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//opportunities
**Access**: Public
**Example Request**:
```bash
curl -X GET /api//opportunities \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//pipeline/stages
**Access**: Public
**Example Request**:
```bash
curl -X GET /api//pipeline/stages \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//team
**Access**: Public
**Example Request**:
```bash
curl -X GET /api//team \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **POST** /api//activities
**Access**: Public
**Example Request**:
```bash
curl -X POST /api//activities \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``


### **POST** /api//leads
**Access**: Public
**Example Request**:
```bash
curl -X POST /api//leads \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``


### **POST** /api//opportunities
**Access**: Public
**Example Request**:
```bash
curl -X POST /api//opportunities \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``


### **PUT** /api//leads/:id
**Access**: Public

### **PUT** /api//opportunities/:id
**Access**: Public

### **DELETE** /api//leads/:id
**Access**: Public


---

## 👥 **Social Features**

### **GET** /api//events/:eventId/participant-access
**Access**: Public
**Description**: GET /api/events/:eventId/participant-access
Check viewer's access level for viewing participants
**Example Request**:
```bash
curl -X GET /api//events/:eventId/participant-access \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//events/:eventId/participants
**Access**: Public
**Description**: GET /api/events/:eventId/participants
Get filtered list of event participants based on viewer's access level
**Example Request**:
```bash
curl -X GET /api//events/:eventId/participants \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//events/:eventId/participants/:participantId
**Access**: Public
**Description**: GET /api/events/:eventId/participants/:participantId
Get detailed information about a specific participant (if access allows)
**Example Request**:
```bash
curl -X GET /api//events/:eventId/participants/:participantId \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//events/:eventId/privacy-settings
**Access**: Public
**Description**: GET /api/events/:eventId/privacy-settings
Get user's current privacy settings for an event
**Example Request**:
```bash
curl -X GET /api//events/:eventId/privacy-settings \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **POST** /api//events/:eventId/participants/:participantId/contact
**Access**: Public
**Description**: POST /api/events/:eventId/participants/:participantId/contact
Initiate contact with a participant (if allowed)
**Example Request**:
```bash
curl -X POST /api//events/:eventId/participants/:participantId/contact \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json" \  -d '{"key": "value"}'
``

**Example Response**:
```json
{
  "success": true,
  "message": "Operation successful",
  "data": {}
}
``


### **PUT** /api//events/:eventId/privacy-settings
**Access**: Public
**Description**: PUT /api/events/:eventId/privacy-settings
Update user's privacy settings for a specific event


---

## 🏥 **System Monitoring**

### **GET** /api//database
**Access**: Public
**Description**: GET /api/health/database
Database health check with detailed status
**Example Request**:
```bash
curl -X GET /api//database \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//diagnostics
**Access**: Public
**Description**: GET /api/system/diagnostics
Run system diagnostics tests
**Example Request**:
```bash
curl -X GET /api//diagnostics \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//full
**Access**: Public
**Description**: GET /api/health/full
Comprehensive health check including database and R2 sync
**Example Request**:
```bash
curl -X GET /api//full \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//health
**Access**: Public
**Description**: GET /api/system/health
Comprehensive system health dashboard data
**Example Request**:
```bash
curl -X GET /api//health \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//health/detailed
**Access**: Public
**Description**: GET /api/system/health/detailed
Detailed health check with all components
**Example Request**:
```bash
curl -X GET /api//health/detailed \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//metrics
**Access**: Public
**Description**: GET /api/system/metrics
System performance metrics
**Example Request**:
```bash
curl -X GET /api//metrics \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``


### **GET** /api//r2-sync
**Access**: Public
**Description**: GET /api/health/r2-sync
R2 backup service health check and status
**Example Request**:
```bash
curl -X GET /api//r2-sync \  -H "Authorization: Bearer <token>" \  -H "Content-Type: application/json"
``

**Example Response**:
```json
{
  "success": true,
  "data": {}
}
``



---

## 📊 **API Statistics**

### **Endpoint Summary**
- **Total Endpoints**: 114
- **Route Files**: 18

### **Methods Distribution**
- **DELETE**: 7 endpoints
- **GET**: 64 endpoints
- **POST**: 34 endpoints
- **PUT**: 9 endpoints

### **Files Distribution**
- **admin.ts**: 9 endpoints
- **analyticsRoutes.ts**: 12 endpoints
- **authRoutes.ts**: 11 endpoints
- **categoryManagement.ts**: 5 endpoints
- **deploymentRoutes.ts**: 7 endpoints
- **emergencyRoutes.ts**: 4 endpoints
- **eventManagement.ts**: 7 endpoints
- **eventRoutes.ts**: 5 endpoints
- **health.ts**: 3 endpoints
- **main.ts**: 0 endpoints
- **mediaRoutes.ts**: 9 endpoints
- **participants.ts**: 6 endpoints
- **placeholderRoutes.ts**: 1 endpoints
- **registrationRoutes.ts**: 6 endpoints
- **salesManagement.ts**: 13 endpoints
- **systemHealthRoutes.ts**: 4 endpoints
- **userManagement.ts**: 7 endpoints
- **venueManagement.ts**: 5 endpoints

### **Categories Distribution**
- **Administration**: 9 endpoints
- **Authentication**: 11 endpoints
- **Business Intelligence**: 12 endpoints
- **Deployment**: 7 endpoints
- **Emergency Operations**: 4 endpoints
- **Event Management**: 12 endpoints
- **General**: 18 endpoints
- **Media Management**: 9 endpoints
- **Registration System**: 6 endpoints
- **Sales Management**: 13 endpoints
- **Social Features**: 6 endpoints
- **System Monitoring**: 7 endpoints
## 📝 **Usage Examples**

### **Authentication Example**
```bash
# Login and get token
curl -X POST http://localhost:5000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "password"}'

# Use token for authenticated requests
curl -X GET http://localhost:5000/api/events \
  -H "Authorization: Bearer <your-jwt-token>"
```

### **Error Handling**
All API endpoints return consistent error responses:

```json
{
  "success": false,
  "error": "Error description",
  "details": {}
}
```

### **HTTP Status Codes**
- `200` - Success
- `201` - Created
- `400` - Bad Request
- `401` - Unauthorized
- `403` - Forbidden
- `404` - Not Found
- `500` - Internal Server Error

---

**Generated**: 2025-10-03T04:15:57.658Z
**Auto-Update**: This documentation is generated automatically from route files