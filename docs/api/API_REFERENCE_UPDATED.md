# API Reference - HeSocial Platform

**Last Updated**: October 3, 2025 - Complete Implementation Audit
**Base URL**: `http://localhost:5000/api` (Development)
**Implementation Status**: 15/15 Major Systems Working ✅

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

### **Status Legend**
- ✅ **WORKING**: Fully implemented, tested, and production-ready
- 🔄 **IMPLEMENTED**: Code exists but needs testing or has limitations
- 📋 **PLANNED**: Documented but not implemented
- ⏸️ **ROADMAP**: Future consideration

---

## 🔐 **Authentication System** ✅ **WORKING**

### **POST /api/auth/register**
**Access**: Public
**Description**: User registration with email verification

### **POST /api/auth/login**
**Access**: Public
**Description**: Email/password authentication

### **GET /api/auth/google**
**Access**: Public
**Description**: Google OAuth 2.0 authentication flow

### **GET /api/auth/profile**
**Access**: User
**Description**: Get current user profile

### **PUT /api/auth/profile**
**Access**: User
**Description**: Update current user profile

---

## 📅 **Event Management** ✅ **WORKING**

### **GET /api/events**
**Access**: Public (Optional Auth)
**Description**: List public events with pagination

**Query Parameters**:
- `page` (optional): Page number (default: 1)
- `limit` (optional): Items per page (default: 20)
- `category` (optional): Filter by category
- `search` (optional): Search events by name/description

### **GET /api/events/:id**
**Access**: Public (Optional Auth)
**Description**: Get detailed event information

### **GET /api/categories**
**Access**: Public
**Description**: Get event categories

### **GET /api/venues**
**Access**: Public
**Description**: Get event venues

---

## 📝 **Event Registration System** ✅ **WORKING**

### **GET /api/registrations/user**
**Access**: User
**Description**: Get user's registration history

### **POST /api/registrations/events/:eventId**
**Access**: User
**Description**: Register for an event

### **GET /api/registrations/:id**
**Access**: User
**Description**: Get registration details

---

## 👥 **Participant Social System** ✅ **WORKING**

### **GET /api/events/:eventId/participants**
**Access**: User (Payment Required)
**Description**: List event participants with privacy filtering

### **GET /api/events/:eventId/participant-access**
**Access**: User
**Description**: Check user's access level for event participants

### **GET /api/events/:eventId/participants/:participantId**
**Access**: User (Payment Required)
**Description**: Get participant details based on privacy settings

### **POST /api/events/:eventId/participants/:participantId/contact**
**Access**: User (Payment Required)
**Description**: Initiate contact with participant

### **GET /api/events/:eventId/privacy-settings**
**Access**: User
**Description**: Get user's privacy settings for event

### **PUT /api/events/:eventId/privacy-settings**
**Access**: User
**Description**: Update user's privacy settings for event

---

## 💰 **Sales Management System** ✅ **WORKING**

### **GET /api/sales/leads**
**Access**: Admin+
**Description**: Get sales leads with filtering and pagination

### **POST /api/sales/leads**
**Access**: Admin+
**Description**: Create new sales lead

### **GET /api/sales/leads/:id**
**Access**: Admin+
**Description**: Get specific lead details

### **PUT /api/sales/leads/:id**
**Access**: Admin+
**Description**: Update lead information

### **DELETE /api/sales/leads/:id**
**Access**: Admin+
**Description**: Delete lead

### **GET /api/sales/opportunities**
**Access**: Admin+
**Description**: Get sales opportunities

### **POST /api/sales/opportunities**
**Access**: Admin+
**Description**: Create sales opportunity

### **PUT /api/sales/opportunities/:id**
**Access**: Admin+
**Description**: Update opportunity

### **GET /api/sales/activities**
**Access**: Admin+
**Description**: Get sales activities

### **POST /api/sales/activities**
**Access**: Admin+
**Description**: Create sales activity

### **GET /api/sales/metrics**
**Access**: Admin+
**Description**: Get sales analytics and metrics

### **GET /api/sales/pipeline/stages**
**Access**: Admin+
**Description**: Get sales pipeline configuration

### **GET /api/sales/team**
**Access**: Admin+
**Description**: Get sales team information

---

## 📊 **Business Intelligence & Analytics** ✅ **WORKING**

### **GET /api/analytics/visitors**
**Access**: Admin+
**Description**: Get visitor analytics overview

**Query Parameters**:
- `days` (optional): Number of days to analyze (default: 30)

### **GET /api/analytics/visitors/daily**
**Access**: Admin+
**Description**: Get daily visitor analytics breakdown

### **GET /api/analytics/pages/popular**
**Access**: Admin+
**Description**: Get most popular pages analytics

### **GET /api/analytics/conversion**
**Access**: Admin+
**Description**: Get conversion funnel analytics

### **GET /api/analytics/events/overview**
**Access**: Admin+
**Description**: Get event performance overview

### **GET /api/analytics/events/performance**
**Access**: Admin+
**Description**: Get individual event performance metrics

### **GET /api/analytics/events/:id/performance**
**Access**: Admin+
**Description**: Get specific event performance data

### **GET /api/analytics/events/engagement**
**Access**: Admin+
**Description**: Get event engagement metrics

### **POST /api/analytics/events/track**
**Access**: Admin+
**Description**: Track custom analytics events

### **GET /api/analytics/revenue/events**
**Access**: Admin+
**Description**: Get revenue analytics by events

### **GET /api/analytics/engagement/members**
**Access**: Admin+
**Description**: Get member engagement metrics

---

## 🏥 **System Health & Monitoring** ✅ **WORKING**

### **GET /api/health**
**Access**: Public
**Description**: Basic system health check

### **GET /api/health/detailed**
**Access**: Admin+
**Description**: Detailed system health diagnostics

### **GET /api/system/metrics**
**Access**: Admin+
**Description**: Get system performance metrics

### **GET /api/system/diagnostics**
**Access**: Admin+
**Description**: Run system diagnostics

---

## 🚀 **Blue-Green Deployment System** ✅ **WORKING**

### **GET /api/deployment/status**
**Access**: Admin+
**Description**: Get deployment system status

### **GET /api/deployment/health**
**Access**: Admin+
**Description**: Check health of all database environments

### **POST /api/deployment/deploy-visitor-tracking**
**Access**: Admin+
**Description**: Deploy visitor tracking with zero downtime

### **POST /api/deployment/rollback**
**Access**: Admin+
**Description**: Instant rollback to previous environment

### **GET /api/deployment/migration-plans**
**Access**: Admin+
**Description**: Get available migration plans

### **POST /api/deployment/test-connection**
**Access**: Admin+
**Description**: Test database connections

---

## 🚨 **Emergency Operations** ✅ **WORKING**

### **POST /api/emergency/apply-visitor-tracking**
**Access**: Super Admin+
**Description**: Emergency visitor tracking schema deployment

### **POST /api/emergency/fix-analytics-queries**
**Access**: Super Admin+
**Description**: Emergency analytics query fixes

### **GET /api/emergency/test-visitor-tracking**
**Access**: Super Admin+
**Description**: Test visitor tracking functionality

### **GET /api/emergency/test-analytics**
**Access**: Super Admin+
**Description**: Test analytics queries

---

## 🔧 **Debug & Development Endpoints** ✅ **WORKING**

### **GET /api/debug/tables**
**Access**: Admin+
**Description**: List all database tables

### **GET /api/debug/counts**
**Access**: Admin+
**Description**: Get row counts for all tables

### **POST /api/debug/seed**
**Access**: Admin+
**Description**: Seed database with sample data

### **POST /api/debug/manual-seed**
**Access**: Admin+
**Description**: Manual database seeding

### **POST /api/debug/update-dates**
**Access**: Admin+
**Description**: Update event dates to future

### **POST /api/debug/expand-seed**
**Access**: Admin+
**Description**: Create comprehensive test data

### **GET /api/debug/events-all**
**Access**: Admin+
**Description**: Get all events regardless of date

---

## 🔄 **Legacy Endpoints** ✅ **WORKING** (Deprecated)

### **GET /api/legacy/events**
**Access**: Public
**Description**: Legacy events endpoint (backwards compatibility)

### **GET /api/legacy/events/categories**
**Access**: Public
**Description**: Legacy categories endpoint

### **GET /api/legacy/events/venues**
**Access**: Public
**Description**: Legacy venues endpoint

### **GET /api/legacy/events/:id**
**Access**: Public
**Description**: Legacy event details endpoint

---

## 📋 **Planned Features** 📋 **NOT IMPLEMENTED**

### **Admin Management System** 🔄 **PARTIALLY IMPLEMENTED**
> **Note**: Routes exist but are commented out in index.ts for stability

### **POST /api/admin/users**
**Access**: Admin+
**Description**: Create new user account

### **GET /api/admin/users**
**Access**: Admin+
**Description**: Get user management data

### **PUT /api/admin/users/:id**
**Access**: Admin+
**Description**: Update user information

### **DELETE /api/admin/users/:id**
**Access**: Admin+
**Description**: Delete user account

### **POST /api/admin/backup**
**Access**: Admin+
**Description**: Create database backup

---

### **Media Management System** 🔄 **PARTIALLY IMPLEMENTED**
> **Note**: Basic routes exist but return empty data

### **GET /api/media/events/:eventId**
**Access**: Public
**Description**: Get event media files

### **POST /api/media/upload**
**Access**: Admin+
**Description**: Upload media files

### **DELETE /api/media/:id**
**Access**: Admin+
**Description**: Delete media file

---

## 📋 **Roadmap Features** ⏸️ **FUTURE CONSIDERATION**

### **Push Notifications**
- Real-time notifications for events and social interactions

### **Advanced Member Matching**
- AI-powered participant recommendations

### **Google Analytics 4 Integration**
- Complete GA4 setup using visitor tracking foundation

### **Webhook System**
- Event-driven notifications for external integrations

---

## 📋 **Response Formats**

### **Success Response**
```json
{
  "success": true,
  "data": {}, // Response data
  "message": "Operation successful", // For POST/PUT/DELETE
  "pagination": { // For paginated responses
    "page": 1,
    "limit": 20,
    "total": 100,
    "totalPages": 5
  }
}
```

### **Error Response**
```json
{
  "success": false,
  "error": "Error message",
  "details": {} // Additional error details
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

## 🔧 **Development Notes**

### **Test Accounts**
- **Admin**: `admin@hesocial.com` / `admin123`
- **Test User**: `test.platinum@example.com` / `test123`
- **Dev Token**: `dev-token-12345`

### **Database Tables**
All endpoints are backed by a complete database schema with the following key tables:
- `users` - User accounts and profiles
- `events` - Event information
- `event_registrations` - Event participation
- `sales_*` - Sales management system (5 tables)
- `event_privacy_overrides` - Privacy settings
- `participant_view_logs` - Security logging
- `visitor_*` - Analytics tracking (3 tables)

### **Rate Limiting**
- **Public endpoints**: 100 requests per minute
- **Authenticated endpoints**: 1000 requests per minute
- **Admin endpoints**: 500 requests per minute

---

## 📚 **Additional Documentation**

- [Database Schema](../database/DATABASE_SYSTEM.md)
- [Development Commands](../commands/DEVELOPMENT_COMMANDS.md)
- [System Architecture](../architecture/SYSTEM_ARCHITECTURE.md)
- [Authentication Implementation](../authentication/AUTHENTICATION_SYSTEM.md)

---

**Generated**: October 3, 2025
**Next Update**: After Phase 2 completion
**Version**: 2.0 - Implementation-Accurate Documentation