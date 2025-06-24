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

- **Frontend**: React, TypeScript, Tailwind CSS
- **Backend**: Node.js, Express, TypeScript
- **Database**: PostgreSQL
- **Authentication**: OAuth 2.0 + JWT
- **Payments**: Stripe, Adyen, BitPay
- **Infrastructure**: AWS (ECS, RDS, CloudFront)

## Development

See [CLAUDE.md](./CLAUDE.md) for detailed development guidelines and architecture information.

## Security

This platform handles sensitive financial and personal data. All development must follow strict security protocols including:
- AES-256 encryption for data at rest
- TLS 1.3 for data in transit
- Multi-factor authentication
- GDPR/CCPA compliance