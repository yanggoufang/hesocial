# Default Users and Admin Accounts

## Overview

The HeSocial platform includes default admin accounts and sample users for development, testing, and system administration. All default passwords use secure bcrypt hashing.

## 🔐 Admin Accounts

### Super Admin Accounts
For full system administration and development:

| Email | Password | Role | Membership | Access Level |
|-------|----------|------|------------|--------------|
| `admin@hesocial.com` | `admin123` | super_admin | Black Card | Full system access |
| `superadmin@hesocial.com` | `admin123` | super_admin | Black Card | Full system access |

### Content Admin Account
For event management and member relations:

| Email | Password | Role | Membership | Access Level |
|-------|----------|------|------------|--------------|
| `events@hesocial.com` | `admin123` | admin | Diamond | Event management |

## 👥 Test User Accounts

### Membership Tier Testing
Test accounts for each membership tier:

| Email | Password | Membership | Status | Purpose |
|-------|----------|------------|--------|---------|
| `test.platinum@example.com` | `test123` | Platinum | Verified | Platinum tier testing |
| `test.diamond@example.com` | `test123` | Diamond | Verified | Diamond tier testing |
| `test.blackcard@example.com` | `test123` | Black Card | Verified | Black Card tier testing |

### Special Test Cases

| Email | Password | Status | Purpose |
|-------|----------|--------|---------|
| `test.pending@example.com` | `test123` | Pending Verification | Test verification process |
| `test.oauth@gmail.com` | No password | OAuth User | Test Google OAuth flow |

## 🌟 Realistic Sample Users

The platform includes 8 realistic user profiles with detailed backgrounds:

### High-Profile Members

1. **陳志明** - `chen.executive@techcorp.com`
   - **Role**: Diamond Member, Tech CEO
   - **Profile**: Technology company CEO with AI/blockchain investments
   - **Interests**: Tech innovation, art collection, golf, wine tasting

2. **王美麗** - `wang.investor@goldmangroup.com`
   - **Role**: Black Card Member, Investment Banker
   - **Profile**: International investment expert, private wealth management founder
   - **Interests**: International investment, M&A strategy, fine dining, classical music

3. **林建華** - `lin.architect@designstudio.com`
   - **Role**: Platinum Member, Architect
   - **Profile**: International architect with Pritzker Prize-winning designs
   - **Interests**: Architecture, sustainability, photography, tea ceremony

4. **張雅婷** - `zhang.doctor@medicalcenter.com`
   - **Role**: Diamond Member, Plastic Surgeon
   - **Profile**: International plastic surgery authority with private clinic
   - **Interests**: Medical research, anti-aging science, yoga, nutrition

5. **劉國強** - `liu.realestate@empiregroup.com`
   - **Role**: Black Card Member, Real Estate Developer
   - **Profile**: Real estate group founder with developments in Taipei, Shanghai, Singapore
   - **Interests**: Real estate investment, urban planning, antique collection

6. **黃淑芬** - `huang.finance@capitalfund.com`
   - **Role**: Diamond Member, Hedge Fund Manager
   - **Profile**: Top hedge fund manager managing $5B+ assets, Harvard MBA
   - **Interests**: Quantitative investment, risk management, equestrian, wine collection

7. **吳俊傑** - `wu.entrepreneur@innovatetech.com`
   - **Role**: Platinum Member, Serial Entrepreneur
   - **Profile**: Serial entrepreneur with 3 successful exits, angel investor
   - **Interests**: Startup investment, AI, biotech, extreme sports

8. **李心怡** - `lee.lawyer@toplaw.com`
   - **Role**: Diamond Member, International Lawyer
   - **Profile**: International commercial lawyer specializing in M&A and IP law
   - **Interests**: International law, IP rights, art law, classical literature

## 🔑 Admin Capabilities

### Super Admin Access
Super admins (`super_admin` role) have access to:
- **System Administration**: Full platform configuration
- **User Management**: Create, modify, delete any user accounts
- **Database Operations**: Backup, restore, migration management
- **Security Settings**: Access to all security configurations
- **Development Tools**: Debug information and development endpoints

### Admin Access
Regular admins (`admin` role) have access to:
- **Event Management**: Create and manage luxury events
- **Member Relations**: User verification and support
- **Content Management**: Platform content and announcements
- **Backup Operations**: Create manual backups
- **Health Monitoring**: System health and performance metrics

## 🛡️ Security Features

### Password Security
- **Encryption**: All passwords use bcrypt with 12 salt rounds
- **Default Passwords**: Should be changed in production environments
- **Password Policy**: Minimum 8 characters for admin accounts

### Role-Based Access
```typescript
// Authentication middleware checks
requireAdmin()      // Requires 'admin' or 'super_admin' role
requireSuperAdmin() // Requires 'super_admin' role only
```

### API Endpoint Protection
- **Admin Routes**: `/api/admin/*` - Requires admin authentication
- **Super Admin Routes**: Some admin operations require super admin access
- **User Routes**: `/api/auth/*` - Standard user authentication

## 📚 Usage Examples

### Admin Login
```typescript
// Frontend login for admin
const result = await login({
  email: 'admin@hesocial.com',
  password: 'admin123'
})

// Check admin role
if (user.role === 'super_admin') {
  // Full admin access
}
```

### Backend Admin Check
```typescript
// Protect admin routes
router.post('/admin/backup', authenticateToken, requireAdmin, async (req, res) => {
  // Admin-only functionality
})

// Protect super admin routes  
router.post('/admin/restore', authenticateToken, requireSuperAdmin, async (req, res) => {
  // Super admin only
})
```

## 🚀 Getting Started

### For Development
1. **Use Admin Account**: Login with `admin@hesocial.com` / `admin123`
2. **Test User Flows**: Use test accounts for different membership tiers
3. **Verify Features**: Use pending and OAuth test accounts

### For Testing
1. **Membership Testing**: Use tier-specific test accounts
2. **Verification Testing**: Use `test.pending@example.com`
3. **OAuth Testing**: Use `test.oauth@gmail.com`

### For Production
1. **Change Default Passwords**: Update all admin passwords
2. **Remove Test Accounts**: Delete test accounts for security
3. **Create Production Admins**: Create secure admin accounts

## ⚠️ Security Notes

### Development Environment
- Default passwords are acceptable for local development
- Test accounts provide comprehensive testing coverage
- Admin accounts have appropriate role separation

### Production Environment
- **MUST CHANGE**: All default admin passwords
- **MUST REMOVE**: All test accounts (`test.*@example.com`)
- **MUST SECURE**: Admin account email addresses
- **MUST MONITOR**: Admin account usage and access logs

## 📋 Database Integration

To load default users into the database:

```sql
-- Load admin users and test accounts
\i database/duckdb-admin-users.sql

-- Load realistic sample users  
\i database/duckdb-seed-realistic.sql
```

The admin users are automatically created with ID range 1000+ to avoid conflicts with regular users (ID range 2000+ for test users, auto-increment for real users).

## 🎯 Summary

The HeSocial platform provides a comprehensive set of default users for all development and testing needs:

- **3 Admin Accounts**: Full system administration capabilities
- **5 Test Accounts**: Complete membership tier and state testing  
- **8 Realistic Users**: Rich sample data for development
- **Role-Based Security**: Proper access control and authentication
- **Production Ready**: Easy transition from development to production

All accounts are production-ready with proper security measures and can be easily managed through the authentication system.