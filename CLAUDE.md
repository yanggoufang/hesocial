# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a high-end social event platform targeting affluent individuals aged 45-65 with high net worth (NT$5M+ annual income or NT$30M+ net assets). The platform facilitates luxury social events like private dinners, yacht parties, and art appreciation gatherings.

## Planned Technology Stack

### Frontend
- **Framework**: React with TypeScript (CDN loaded)
- **Styling**: Tailwind CSS with luxury color palette:
  - Gold: #D4AF37
  - Deep Blue: #1B2951  
  - Platinum: #E5E4E2
  - Champagne: #F7E7CE
  - Midnight Black: #0C0C0C
- **Key Libraries**: React Router, Axios, React Player, Three.js (for AR previews)

### Backend
- **Primary**: Node.js + Express + TypeScript
- **Secondary**: Python + Django (for AI recommendation system)
- **API**: GraphQL for complex queries
- **Authentication**: OAuth 2.0 (LinkedIn, Google) + JWT

### Database
- **Type**: PostgreSQL (AWS RDS Aurora)
- **Key Tables**: Users (with membership tiers), Financial Verification, Events, Registrations

### Infrastructure
- **Cloud**: AWS (ECS Fargate, RDS Aurora, ElastiCache Redis, CloudFront CDN)
- **CI/CD**: GitHub Actions
- **SSL**: AWS Certificate Manager

## Development Commands

Once the project is initialized, typical commands will be:

```bash
# Frontend development
npm run dev          # Start development server
npm run build        # Build for production
npm run lint         # Run ESLint
npm run typecheck    # TypeScript type checking

# Backend development  
npm run server       # Start Express server
npm run test         # Run unit tests
npm run test:watch   # Run tests in watch mode

# Database operations
npm run migrate      # Run database migrations
npm run seed         # Seed development data
```

## Key Architecture Principles

### Security & Privacy
- AES-256 encryption for data at rest
- TLS 1.3 for data in transit
- Multi-factor authentication (SMS + authenticator)
- GDPR/CCPA compliance with data minimization
- User data soft deletion (30-day retention)

### User Management
- Three-tier membership: Platinum, Diamond, Black Card
- Financial verification required for registration
- Privacy levels 1-5 for profile visibility
- Admin approval process for new users

### Event Structure
Events follow a premium model with these key fields:
- Event name, date/time, registration deadline
- Venue (luxury locations)
- Pricing (typically NT$6,000-10,000+)
- Exclusivity level (VIP/VVIP/Invitation Only)
- Dress code (1-5 star rating system)
- Privacy guarantees and exclusive services

### Payment Integration
- Stripe Connect for high-value transactions
- Adyen for international payments
- BitPay for cryptocurrency
- Support for installment payments (6-24 months)

## Code Style Guidelines

- Use TypeScript throughout
- Follow luxury UX patterns with elegant animations (0.6s transitions)
- Implement responsive design for high-end devices (iPhone 15 Pro Max, Samsung Galaxy S24 Ultra)
- Support 4K/8K resolution and HDR10+ content
- Maintain strict type safety and error handling

## Testing Strategy

- Unit tests for all business logic
- Integration tests for payment flows
- Security penetration testing
- Load testing for 2000+ concurrent VIP users
- Performance targets: <800ms page load, <50ms database response

## Deployment Considerations

- Multi-region AWS deployment for low latency
- CDN for global content delivery
- Automated scaling based on user load
- Blue-green deployment strategy
- Comprehensive monitoring and alerting