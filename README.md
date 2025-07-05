# HeSocial - High-End Social Event Platform

A luxury social event platform designed for affluent individuals aged 45-65, facilitating premium networking events such as private dinners, yacht parties, and art appreciation gatherings.

## Quick Start

```bash
# Install all dependencies
npm run setup

# Start development servers (frontend + backend)
npm run dev

# Build for production
npm run build

# Run tests
npm run test

# Run linting
npm run lint

# Type checking
npm run typecheck
```

## Project Structure

```
hesocial/
├── frontend/          # React + TypeScript frontend
├── backend/           # Node.js + Express backend
├── database/          # Database schemas and migrations
├── docs/              # Documentation
├── CLAUDE.md          # AI assistant guidance
└── package.json       # Root package configuration
```

## Technology Stack

- **Frontend**: React, TypeScript, Tailwind CSS, Vite
- **Backend**: Node.js, Express, TypeScript
- **Database**: DuckDB with Cloudflare R2 backup system
- **Authentication**: OAuth 2.0 + JWT (Google, LinkedIn)
- **Payments**: Stripe integration
- **Infrastructure**: Cloudflare R2 for database persistence

## Documentation

- **[Development Guide](./CLAUDE.md)** - AI assistant guidance and architecture information
- **[Setup Documentation](./docs/setup/)** - Database and service setup guides
- **[Project Plans](./docs/plans/)** - Implementation plans and roadmaps
- **[Specifications](./docs/specifications/)** - Platform specifications and requirements

## Database & Backup

The platform uses DuckDB as the primary database with Cloudflare R2 for production-ready backup and persistence:

- **Automatic Backups**: Created on graceful server shutdown
- **Manual Backups**: Available via admin API endpoints
- **Environment Separation**: Development and production buckets
- **Backup Endpoints**: 
  - `POST /api/admin/backup` - Create manual backup
  - `GET /api/admin/backups` - List available backups

## Security

This platform handles sensitive financial and personal data. All development must follow strict security protocols including:
- AES-256 encryption for data at rest
- TLS 1.3 for data in transit
- Multi-factor authentication
- GDPR/CCPA compliance