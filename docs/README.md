# HeSocial Documentation

This directory contains all project documentation organized by category.

## Directory Structure

```
docs/
├── README.md                        # This file - documentation index
├── setup/                           # Setup and configuration guides
├── specifications/                  # Platform specifications and requirements
├── development/                     # Development tools and processes
└── R2_BACKUP_IMPLEMENTATION.md     # Completed implementation results
```

## Setup Documentation (`setup/`)

Configuration and setup guides for services:

- **[CLOUDFLARE_R2_SETUP.md](./setup/CLOUDFLARE_R2_SETUP.md)** - ✅ Production-ready R2 backup system setup

## Implementation Results

Completed implementations and their results:

- **[R2_BACKUP_IMPLEMENTATION.md](./R2_BACKUP_IMPLEMENTATION.md)** - ✅ COMPLETED R2 backup system implementation results

## Specifications (`specifications/`)

Platform specifications and requirements:

- **[SocialEventPlatform_Specification_HighEnd.md](./specifications/SocialEventPlatform_Specification_HighEnd.md)** - High-end social platform specifications

## Development (`development/`)

Development tools and processes:

- **[GEMINI.md](./development/GEMINI.md)** - Gemini integration documentation

## Main Documentation

Key documentation files in the project root:

- **[../README.md](../README.md)** - Project overview and quick start
- **[../CLAUDE.md](../CLAUDE.md)** - AI assistant guidance and architecture information

## Implementation Status

### ✅ Completed Features
- **Cloudflare R2 Backup System** - Production-ready with graceful shutdown auto-backup and manual API endpoints
- **DuckDB Integration** - Embedded database with environment separation

### 📋 Current Architecture
- **Frontend**: React + TypeScript + Vite + Tailwind CSS
- **Backend**: Node.js + Express + TypeScript + DuckDB
- **Database**: DuckDB with Cloudflare R2 backup system
- **Authentication**: OAuth 2.0 + JWT (Google, LinkedIn)
- **Payments**: Stripe integration

## Getting Help

For development guidance, see [CLAUDE.md](../CLAUDE.md) which contains:
- Development commands and workflows
- Project architecture details
- Task management guidelines
- Code conventions and best practices