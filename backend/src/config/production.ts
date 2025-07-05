import logger from '../utils/logger.js';

export interface ProductionConfig {
  server: {
    port: number;
    host: string;
    nodeEnv: string;
    frontendUrl: string;
  };
  database: {
    path: string;
    backupEnabled: boolean;
    backupRetentionDays: number;
  };
  security: {
    jwtSecret: string;
    jwtExpiresIn: string;
    corsOrigins: string[];
    rateLimitWindow: number;
    rateLimitMaxRequests: number;
  };
  cloudflare: {
    r2: {
      enabled: boolean;
      accountId?: string;
      accessKeyId?: string;
      secretAccessKey?: string;
      bucketName?: string;
      endpoint?: string;
      region?: string;
      backupPath?: string;
    };
  };
  stripe: {
    enabled: boolean;
    secretKey?: string;
    webhookSecret?: string;
    publishableKey?: string;
  };
  oauth: {
    google: {
      enabled: boolean;
      clientId?: string;
      clientSecret?: string;
      redirectUri?: string;
    };
    linkedin: {
      enabled: boolean;
      clientId?: string;
      clientSecret?: string;
      redirectUri?: string;
    };
  };
  logging: {
    level: string;
    format: string;
    enableConsole: boolean;
    enableFile: boolean;
  };
}

export interface ConfigValidationResult {
  isValid: boolean;
  errors: string[];
  warnings: string[];
}

export class ProductionConfigService {
  private config: ProductionConfig;

  constructor() {
    this.config = this.loadConfiguration();
  }

  /**
   * Load configuration from environment variables
   */
  private loadConfiguration(): ProductionConfig {
    return {
      server: {
        port: parseInt(process.env.PORT || '5000', 10),
        host: process.env.HOST || '0.0.0.0',
        nodeEnv: process.env.NODE_ENV || 'development',
        frontendUrl: process.env.FRONTEND_URL || 'http://localhost:3000'
      },
      database: {
        path: process.env.DB_PATH || './hesocial.duckdb',
        backupEnabled: process.env.BACKUP_ENABLED === 'true',
        backupRetentionDays: parseInt(process.env.BACKUP_RETENTION_DAYS || '30', 10)
      },
      security: {
        jwtSecret: process.env.JWT_SECRET || '',
        jwtExpiresIn: process.env.JWT_EXPIRES_IN || '7d',
        corsOrigins: process.env.CORS_ORIGINS?.split(',') || ['http://localhost:3000'],
        rateLimitWindow: parseInt(process.env.RATE_LIMIT_WINDOW || '900000', 10), // 15 minutes
        rateLimitMaxRequests: parseInt(process.env.RATE_LIMIT_MAX_REQUESTS || '100', 10)
      },
      cloudflare: {
        r2: {
          enabled: process.env.BACKUP_ENABLED === 'true',
          accountId: process.env.R2_ACCOUNT_ID,
          accessKeyId: process.env.R2_ACCESS_KEY_ID,
          secretAccessKey: process.env.R2_SECRET_ACCESS_KEY,
          bucketName: process.env.R2_BUCKET_NAME,
          endpoint: process.env.R2_ENDPOINT,
          region: process.env.R2_REGION || 'auto',
          backupPath: process.env.R2_BACKUP_PATH || '/backups/'
        }
      },
      stripe: {
        enabled: !!(process.env.STRIPE_SECRET_KEY && process.env.STRIPE_WEBHOOK_SECRET),
        secretKey: process.env.STRIPE_SECRET_KEY,
        webhookSecret: process.env.STRIPE_WEBHOOK_SECRET,
        publishableKey: process.env.STRIPE_PUBLISHABLE_KEY
      },
      oauth: {
        google: {
          enabled: !!(process.env.GOOGLE_CLIENT_ID && process.env.GOOGLE_CLIENT_SECRET),
          clientId: process.env.GOOGLE_CLIENT_ID,
          clientSecret: process.env.GOOGLE_CLIENT_SECRET,
          redirectUri: process.env.GOOGLE_REDIRECT_URI
        },
        linkedin: {
          enabled: !!(process.env.LINKEDIN_CLIENT_ID && process.env.LINKEDIN_CLIENT_SECRET),
          clientId: process.env.LINKEDIN_CLIENT_ID,
          clientSecret: process.env.LINKEDIN_CLIENT_SECRET,
          redirectUri: process.env.LINKEDIN_REDIRECT_URI
        }
      },
      logging: {
        level: process.env.LOG_LEVEL || 'info',
        format: process.env.LOG_FORMAT || 'combined',
        enableConsole: process.env.LOG_CONSOLE !== 'false',
        enableFile: process.env.LOG_FILE === 'true'
      }
    };
  }

  /**
   * Get the configuration
   */
  getConfig(): ProductionConfig {
    return this.config;
  }

  /**
   * Validate configuration with comprehensive checks
   */
  validateConfiguration(): ConfigValidationResult {
    const errors: string[] = [];
    const warnings: string[] = [];

    // Critical validations (will prevent server startup)
    this.validateCriticalConfig(errors);
    
    // Warning validations (will log warnings but allow startup)
    this.validateOptionalConfig(warnings);

    return {
      isValid: errors.length === 0,
      errors,
      warnings
    };
  }

  /**
   * Validate critical configuration that prevents startup
   */
  private validateCriticalConfig(errors: string[]): void {
    const { server, security } = this.config;

    // Server configuration
    if (!server.port || server.port < 1 || server.port > 65535) {
      errors.push('Invalid PORT: must be between 1 and 65535');
    }
    if (!server.frontendUrl) {
      errors.push('FRONTEND_URL is required');
    }
    if (!server.frontendUrl.startsWith('http://') && !server.frontendUrl.startsWith('https://')) {
      errors.push('FRONTEND_URL must start with http:// or https://');
    }

    // Security configuration
    if (!security.jwtSecret) {
      errors.push('JWT_SECRET is required');
    }
    if (security.jwtSecret && security.jwtSecret.length < 32) {
      errors.push('JWT_SECRET must be at least 32 characters long');
    }
    if (security.jwtSecret && security.jwtSecret === 'your-secret-key-here') {
      errors.push('JWT_SECRET cannot be the default value');
    }

    // Production-specific validations
    if (server.nodeEnv === 'production') {
      if (!server.frontendUrl.startsWith('https://')) {
        errors.push('FRONTEND_URL must use HTTPS in production');
      }
      if (security.jwtSecret && security.jwtSecret.length < 64) {
        errors.push('JWT_SECRET should be at least 64 characters in production');
      }
    }
  }

  /**
   * Validate optional configuration (warnings only)
   */
  private validateOptionalConfig(warnings: string[]): void {
    const { database, cloudflare, stripe, oauth } = this.config;

    // Database backup warnings
    if (!database.backupEnabled) {
      warnings.push('Database backup is disabled (BACKUP_ENABLED=false)');
    }
    if (database.backupEnabled && !cloudflare.r2.enabled) {
      warnings.push('Backup is enabled but R2 is not configured');
    }

    // R2 configuration warnings
    if (cloudflare.r2.enabled) {
      const requiredR2Fields = ['accountId', 'accessKeyId', 'secretAccessKey', 'bucketName', 'endpoint'];
      const missingR2Fields = requiredR2Fields.filter(field => !cloudflare.r2[field as keyof typeof cloudflare.r2]);
      if (missingR2Fields.length > 0) {
        warnings.push(`R2 backup enabled but missing: ${missingR2Fields.map(f => `R2_${f.toUpperCase()}`).join(', ')}`);
      }
    }

    // Stripe configuration warnings
    if (!stripe.enabled) {
      warnings.push('Stripe payment processing is disabled');
    }
    if (stripe.enabled && !stripe.publishableKey) {
      warnings.push('Stripe is enabled but STRIPE_PUBLISHABLE_KEY is missing');
    }

    // OAuth configuration warnings
    if (!oauth.google.enabled && !oauth.linkedin.enabled) {
      warnings.push('No OAuth providers configured (Google or LinkedIn)');
    }
    if (oauth.google.enabled && !oauth.google.redirectUri) {
      warnings.push('Google OAuth enabled but GOOGLE_REDIRECT_URI is missing');
    }
    if (oauth.linkedin.enabled && !oauth.linkedin.redirectUri) {
      warnings.push('LinkedIn OAuth enabled but LINKEDIN_REDIRECT_URI is missing');
    }

    // CORS configuration warnings
    if (this.config.security.corsOrigins.includes('*')) {
      warnings.push('CORS is configured to allow all origins (security risk)');
    }
  }

  /**
   * Log configuration status with detailed information
   */
  logConfigurationStatus(): void {
    const validation = this.validateConfiguration();
    const { server, database, cloudflare, stripe, oauth } = this.config;

    logger.info('='.repeat(60));
    logger.info('⚙️  PRODUCTION CONFIGURATION');
    logger.info('='.repeat(60));

    // Server info
    logger.info(`Server: ${server.host}:${server.port} (${server.nodeEnv})`);
    logger.info(`Frontend: ${server.frontendUrl}`);
    logger.info(`Database: ${database.path}`);

    // Feature status
    logger.info('');
    logger.info('📋 FEATURE STATUS:');
    logger.info(`  Backup: ${database.backupEnabled ? '✅ Enabled' : '❌ Disabled'}`);
    logger.info(`  R2 Storage: ${cloudflare.r2.enabled ? '✅ Enabled' : '❌ Disabled'}`);
    logger.info(`  Stripe: ${stripe.enabled ? '✅ Enabled' : '❌ Disabled'}`);
    logger.info(`  Google OAuth: ${oauth.google.enabled ? '✅ Enabled' : '❌ Disabled'}`);
    logger.info(`  LinkedIn OAuth: ${oauth.linkedin.enabled ? '✅ Enabled' : '❌ Disabled'}`);

    // Validation results
    if (validation.errors.length > 0) {
      logger.info('');
      logger.error('❌ CONFIGURATION ERRORS:');
      validation.errors.forEach(error => {
        logger.error(`   • ${error}`);
      });
    }

    if (validation.warnings.length > 0) {
      logger.info('');
      logger.warn('⚠️  CONFIGURATION WARNINGS:');
      validation.warnings.forEach(warning => {
        logger.warn(`   • ${warning}`);
      });
    }

    if (validation.isValid) {
      logger.info('');
      logger.info('✅ Configuration is valid and ready for production');
    } else {
      logger.info('');
      logger.error('❌ Configuration has critical errors that must be fixed');
    }

    logger.info('='.repeat(60));
  }

  /**
   * Get environment-specific suggestions for missing configuration
   */
  getConfigurationSuggestions(): string[] {
    const suggestions: string[] = [];
    const validation = this.validateConfiguration();

    if (validation.errors.length > 0) {
      suggestions.push('Critical configuration errors found:');
      validation.errors.forEach(error => {
        suggestions.push(`  - ${error}`);
      });
    }

    if (validation.warnings.length > 0) {
      suggestions.push('');
      suggestions.push('Optional configuration improvements:');
      validation.warnings.forEach(warning => {
        suggestions.push(`  - ${warning}`);
      });
    }

    // Add specific suggestions based on environment
    if (this.config.server.nodeEnv === 'production') {
      suggestions.push('');
      suggestions.push('Production environment detected. Consider:');
      suggestions.push('  - Use HTTPS for all URLs');
      suggestions.push('  - Enable backup with R2 storage');
      suggestions.push('  - Configure proper CORS origins');
      suggestions.push('  - Set up monitoring and logging');
    }

    return suggestions;
  }

  /**
   * Check if configuration is production-ready
   */
  isProductionReady(): boolean {
    const validation = this.validateConfiguration();
    const { server, database, security } = this.config;

    if (!validation.isValid) {
      return false;
    }

    // Additional production readiness checks
    if (server.nodeEnv === 'production') {
      return (
        server.frontendUrl.startsWith('https://') &&
        security.jwtSecret.length >= 64 &&
        database.backupEnabled
      );
    }

    return true;
  }
}

// Export singleton instance
export const productionConfig = new ProductionConfigService();