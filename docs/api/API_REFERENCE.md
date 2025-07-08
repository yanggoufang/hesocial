# API Reference - HeSocial Platform

**Last Updated**: July 8, 2025  
**Base URL**: `http://localhost:3001/api` (Development)

## 🔐 **Authentication**

All API endpoints require authentication unless specified as **Public**.

### **Authentication Methods**
- **JWT Token**: Include in `Authorization: Bearer <token>` header
- **Development Token**: Use `dev-token-12345` for development testing
- **Cookie Authentication**: Automatic for browser requests

### **Role-Based Access**
- **Public**: No authentication required
- **User**: Authenticated user access
- **Admin**: Admin role required
- **Super Admin**: Super admin role required

---

## 📊 **VISITOR ANALYTICS** ✅ **NEW - July 8, 2025**

### **GET /api/analytics/visitors**
**Access**: Admin+  
**Description**: Get visitor analytics overview

**Query Parameters**:
- `days` (optional): Number of days to analyze (default: 30)

**Response**:
```json
{
  "success": true,
  "data": {
    "period_days": 30,
    "unique_visitors": 1250,
    "total_page_views": 4800,
    "converted_visitors": 85,
    "avg_pages_per_visitor": 3.84,
    "new_visitors": 320
  }
}
```

### **GET /api/analytics/visitors/daily**
**Access**: Admin+  
**Description**: Get daily visitor analytics breakdown

### **GET /api/analytics/pages/popular**
**Access**: Admin+  
**Description**: Get popular pages with conversion rates

### **GET /api/analytics/conversion**
**Access**: Admin+  
**Description**: Get conversion funnel analytics

### **GET /api/analytics/visitors/:visitorId**
**Access**: Admin+  
**Description**: Get detailed visitor journey

### **POST /api/analytics/events/track**
**Access**: Admin+  
**Description**: Track custom visitor events for Google Analytics integration

---

## 🔐 **AUTHENTICATION**

### **POST /api/auth/register** ✅ **WORKING**
**Access**: Public  
**Description**: Register new user account

### **POST /api/auth/login** ✅ **WORKING**
**Access**: Public  
**Description**: User login

### **GET /api/auth/profile** ✅ **WORKING**
**Access**: User+  
**Description**: Get current user profile

### **PUT /api/auth/profile** ✅ **WORKING**
**Access**: User+  
**Description**: Update user profile

### **GET /api/auth/google** ✅ **WORKING**
**Access**: Public  
**Description**: Google OAuth authentication

---

## 🎫 **EVENTS**

### **GET /api/events** ✅ **WORKING**
**Access**: Public  
**Description**: List all events with filtering and pagination

### **GET /api/events/:id** ✅ **WORKING**
**Access**: Public  
**Description**: Get specific event details

### **POST /api/events** ✅ **WORKING**
**Access**: Admin+  
**Description**: Create new event

### **PUT /api/events/:id** ✅ **WORKING**
**Access**: Admin+  
**Description**: Update event

### **DELETE /api/events/:id** ✅ **WORKING**
**Access**: Super Admin  
**Description**: Delete event

---

## 📝 **REGISTRATIONS**

### **GET /api/registrations/user** ✅ **WORKING**
**Access**: User+  
**Description**: Get current user's registration history

### **POST /api/registrations/events/:eventId** ✅ **WORKING**
**Access**: User+  
**Description**: Register for an event

### **GET /api/registrations/:id** ✅ **WORKING**
**Access**: User+  
**Description**: Get registration details

---

## 💼 **SALES MANAGEMENT**

### **GET /api/sales/leads** ✅ **WORKING**
**Access**: Admin+  
**Description**: Get leads with automatic scoring

### **GET /api/sales/opportunities** ✅ **WORKING**
**Access**: Admin+  
**Description**: Get opportunity pipeline tracking

### **GET /api/sales/metrics** ✅ **WORKING**
**Access**: Admin+  
**Description**: Get sales analytics and reporting

---

## 👥 **USER MANAGEMENT**

### **GET /api/users** ✅ **WORKING**
**Access**: Admin+  
**Description**: Get users with pagination and filtering

### **POST /api/users/:id/verify** ✅ **WORKING**
**Access**: Admin+  
**Description**: Verify user account

### **POST /api/users/:id/role** ✅ **WORKING**
**Access**: Super Admin  
**Description**: Update user role

---

## 🔧 **SYSTEM HEALTH**

### **GET /api/system/health** ✅ **WORKING**
**Access**: Admin+  
**Description**: Overall system status monitoring

### **GET /api/system/health/detailed** ✅ **WORKING**
**Access**: Admin+  
**Description**: Component-level health checks

### **GET /api/system/metrics** ✅ **WORKING**
**Access**: Admin+  
**Description**: Real-time system performance data

---

## 🛡️ **ADMIN OPERATIONS**

### **POST /api/admin/backup** ✅ **WORKING**
**Access**: Admin+  
**Description**: Create manual backup to R2

### **GET /api/admin/backups** ✅ **WORKING**
**Access**: Admin+  
**Description**: List available R2 backups

### **POST /api/admin/restore** ✅ **WORKING**
**Access**: Super Admin  
**Description**: Restore database from R2 backup

---

**Note**: All endpoints marked with ✅ **WORKING** are fully functional and tested. New visitor analytics endpoints added July 8, 2025.
PUT  /api/auth/profile     # Update user profile information
```

### OAuth Authentication
```bash
GET  /api/auth/google            # Initiate Google OAuth 2.0 authentication flow
GET  /api/auth/google/callback   # Google OAuth callback with user creation/matching
```

## Event Management Endpoints

### Events
```bash
GET    /api/events                    # List events with filtering and pagination (Public with role-based visibility)
POST   /api/events                    # Create new event (Admin+)
GET    /api/events/:id                # Get specific event details (Public with role-based data)
PUT    /api/events/:id                # Update event (Admin+)
DELETE /api/events/:id                # Delete event (Super Admin only)
POST   /api/events/:id/publish        # Publish event (Admin+)
POST   /api/events/:id/approve        # Approve event (Admin+)
```

### Venues
```bash
GET    /api/venues          # List venues (Public active venues, Admin sees all)
POST   /api/venues          # Create venue (Admin+)
GET    /api/venues/:id      # Get venue details (Public if active, Admin sees all)
PUT    /api/venues/:id      # Update venue (Admin+)
DELETE /api/venues/:id      # Delete venue (Super Admin only)
```

### Categories
```bash
GET    /api/categories      # List event categories (Public active categories, Admin sees all)
POST   /api/categories      # Create category (Admin+)
GET    /api/categories/:id  # Get category details (Public if active, Admin sees all)
PUT    /api/categories/:id  # Update category (Admin+)
DELETE /api/categories/:id  # Delete category (Super Admin only)
```

## Registration Endpoints

```bash
POST /api/registrations                    # Register for event with eligibility checking (User)
GET  /api/registrations/user/:userId       # Get user registrations with filtering and pagination (User owns, Admin all)
DELETE /api/registrations/:id              # Cancel registration (User owns, Admin all)
GET  /api/registrations/event/:eventId     # Get event registrations (Admin+ only)
PUT  /api/registrations/:id/approve        # Approve registration (Admin+ only)
PUT  /api/registrations/:id/reject         # Reject registration (Admin+ only)
GET  /api/registrations/stats              # Registration statistics and analytics (Admin+ only)
```

## Media Management Endpoints

### Event Media
```bash
POST /api/media/events/:eventId/images     # Upload event images with automatic thumbnails (Owner/Admin+)
POST /api/media/events/:eventId/documents  # Upload event documents with secure storage (Owner/Admin+)
GET  /api/media/events/:eventId            # Get event media with optional type filtering (Public with auth-aware URLs)
```

### Venue Media
```bash
POST /api/media/venues/:venueId/images     # Upload venue images with multiple sizes (Admin+ only)
GET  /api/media/venues/:venueId            # Get venue media gallery (Public)
```

### Media Operations
```bash
DELETE /api/media/:mediaId                 # Delete media file and R2 storage cleanup (Owner/Admin+)
GET    /api/media/download/:mediaId        # Generate signed URL for document download (Auth required)
GET    /api/media/stats                    # Media storage statistics and analytics (Admin+ only)
POST   /api/media/cleanup                  # Clean up orphaned media files (Super Admin only)
```

## Sales Management Endpoints

```bash
GET    /api/sales/leads               # List leads with advanced filtering and pagination
GET    /api/sales/leads/:id           # Get specific lead details with activity history
POST   /api/sales/leads               # Create lead with automatic scoring algorithm
PUT    /api/sales/leads/:id           # Update lead information and status
DELETE /api/sales/leads/:id           # Delete lead (Admin+ only)

GET    /api/sales/opportunities       # List opportunities with pipeline tracking
POST   /api/sales/opportunities       # Create formal sales opportunity
PUT    /api/sales/opportunities/:id   # Update opportunity stage and details

GET    /api/sales/activities          # List sales activities with filtering
POST   /api/sales/activities          # Log sales interaction or outcome

GET    /api/sales/metrics             # Comprehensive sales analytics and KPIs
GET    /api/sales/pipeline/stages     # Get configurable pipeline stages
GET    /api/sales/team                # Sales team structure and performance
```

## User Management Endpoints

```bash
GET    /api/users                    # List users with pagination and filtering (Admin+)
GET    /api/users/:id                # Get specific user details
PUT    /api/users/:id                # Update user information (Admin+)
DELETE /api/users/:id                # Delete user (Super Admin only)
POST   /api/users/:id/verify         # Approve/reject user verification (Admin+)
POST   /api/users/:id/role           # Update user role (Super Admin only)
GET    /api/users/stats/overview     # User statistics and analytics
```

## Admin & System Endpoints

### Backup Management
```bash
POST /api/admin/backup          # Create manual backup to R2 (Admin+)
GET  /api/admin/backups         # List available R2 backups with metadata (Admin+)
POST /api/admin/restore         # Restore from specific R2 backup (Super Admin only)
POST /api/admin/cleanup         # Clean up old backups (Admin+)
```

### Health Monitoring
```bash
GET /api/health                 # Basic API health check
GET /api/health/database        # Database connection and query performance
GET /api/health/r2-sync         # R2 backup service status and connectivity
GET /api/health/full            # Comprehensive system status with all components
```

### System Health Dashboard
```bash
GET /api/system/health          # Comprehensive system health dashboard data (Admin+)
GET /api/system/health/detailed # Detailed health check with all components (Admin+)
GET /api/system/metrics         # System performance metrics and resource usage (Admin+)
GET /api/system/diagnostics     # Run system diagnostics tests (Admin+)
```

## Utility Endpoints

### Placeholder Images
```bash
GET /api/placeholder/:width/:height   # Generates a placeholder image with the specified dimensions
```

## Access Control Levels

### Public Endpoints
- Event discovery (GET /api/events)
- Venue information (GET /api/venues)
- Category listing (GET /api/categories)
- Basic health checks (GET /api/health)

### User Endpoints (Authentication Required)
- Profile management
- Event registration
- Personal registration history

### Admin Endpoints (Admin+ Role Required)
- Event creation and management
- Venue and category management
- User verification and management
- Backup operations
- System health monitoring

### Super Admin Endpoints (Super Admin Only)
- User role management
- Critical operations (delete events, restore backups)
- System cleanup operations

## Error Response Format

```json
{
  "success": false,
  "error": "Error message",
  "details": "Additional error details (development only)"
}
```

## Success Response Format

```json
{
  "success": true,
  "data": { /* Response data */ },
  "meta": { /* Pagination/metadata (optional) */ }
}
```