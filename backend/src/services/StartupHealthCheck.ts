import { R2BackupService } from './r2-backup.js';
import { duckdb } from '../database/duckdb-connection.js';
import { MigrationService } from './MigrationService.js';
import logger from '../utils/logger.js';

export interface HealthCheckResult {
  component: string;
  status: 'healthy' | 'warning' | 'error';
  message: string;
  details?: any;
}

export interface HealthCheckSummary {
  overall: 'healthy' | 'warning' | 'error';
  timestamp: string;
  results: HealthCheckResult[];
  criticalErrors: string[];
  warnings: string[];
}

export class StartupHealthCheck {
  private r2BackupService: R2BackupService;
  private migrationService: MigrationService;

  constructor() {
    this.r2BackupService = new R2BackupService();
    this.migrationService = new MigrationService();
  }

  /**
   * Run comprehensive health checks on all system components
   */
  async runHealthChecks(): Promise<HealthCheckSummary> {
    logger.info('Starting comprehensive health checks...');
    
    const results: HealthCheckResult[] = [];
    const criticalErrors: string[] = [];
    const warnings: string[] = [];

    // Check environment variables
    const envCheck = await this.checkEnvironmentVariables();
    results.push(envCheck);
    if (envCheck.status === 'error') {
      criticalErrors.push(envCheck.message);
    } else if (envCheck.status === 'warning') {
      warnings.push(envCheck.message);
    }

    // Check database connectivity
    const dbCheck = await this.checkDatabaseConnection();
    results.push(dbCheck);
    if (dbCheck.status === 'error') {
      criticalErrors.push(dbCheck.message);
    } else if (dbCheck.status === 'warning') {
      warnings.push(dbCheck.message);
    }

    // Check R2 backup service
    const r2Check = await this.checkR2BackupService();
    results.push(r2Check);
    if (r2Check.status === 'error') {
      criticalErrors.push(r2Check.message);
    } else if (r2Check.status === 'warning') {
      warnings.push(r2Check.message);
    }

    // Check payment service configuration
    const paymentCheck = await this.checkPaymentService();
    results.push(paymentCheck);
    if (paymentCheck.status === 'error') {
      criticalErrors.push(paymentCheck.message);
    } else if (paymentCheck.status === 'warning') {
      warnings.push(paymentCheck.message);
    }

    // Check authentication configuration
    const authCheck = await this.checkAuthConfiguration();
    results.push(authCheck);
    if (authCheck.status === 'error') {
      criticalErrors.push(authCheck.message);
    } else if (authCheck.status === 'warning') {
      warnings.push(authCheck.message);
    }

    // Check database migrations
    const migrationCheck = await this.checkDatabaseMigrations();
    results.push(migrationCheck);
    if (migrationCheck.status === 'error') {
      criticalErrors.push(migrationCheck.message);
    } else if (migrationCheck.status === 'warning') {
      warnings.push(migrationCheck.message);
    }

    // Determine overall status
    let overall: 'healthy' | 'warning' | 'error' = 'healthy';
    if (criticalErrors.length > 0) {
      overall = 'error';
    } else if (warnings.length > 0) {
      overall = 'warning';
    }

    const summary: HealthCheckSummary = {
      overall,
      timestamp: new Date().toISOString(),
      results,
      criticalErrors,
      warnings
    };

    this.logHealthCheckResults(summary);
    return summary;
  }

  /**
   * Check required and optional environment variables
   */
  private async checkEnvironmentVariables(): Promise<HealthCheckResult> {
    const requiredEnvs = [
      'NODE_ENV',
      'PORT',
      'JWT_SECRET',
      'FRONTEND_URL'
    ];

    const optionalEnvs = [
      'BACKUP_ENABLED',
      'R2_ACCOUNT_ID',
      'R2_ACCESS_KEY_ID',
      'R2_SECRET_ACCESS_KEY',
      'R2_BUCKET_NAME',
      'R2_ENDPOINT',
      'STRIPE_SECRET_KEY',
      'STRIPE_WEBHOOK_SECRET',
      'GOOGLE_CLIENT_ID',
      'GOOGLE_CLIENT_SECRET',
      'LINKEDIN_CLIENT_ID',
      'LINKEDIN_CLIENT_SECRET'
    ];

    const missingRequired = requiredEnvs.filter(env => !process.env[env]);
    const missingOptional = optionalEnvs.filter(env => !process.env[env]);

    if (missingRequired.length > 0) {
      return {
        component: 'Environment Variables',
        status: 'error',
        message: `Missing required environment variables: ${missingRequired.join(', ')}`,
        details: { missingRequired, missingOptional }
      };
    }

    if (missingOptional.length > 0) {
      return {
        component: 'Environment Variables',
        status: 'warning',
        message: `Missing optional environment variables: ${missingOptional.join(', ')}`,
        details: { missingOptional }
      };
    }

    return {
      component: 'Environment Variables',
      status: 'healthy',
      message: 'All environment variables are configured'
    };
  }

  /**
   * Check database connection and basic functionality
   */
  private async checkDatabaseConnection(): Promise<HealthCheckResult> {
    try {
      // Test basic query using the DuckDB connection
      const result = await duckdb.query('SELECT 1 as test');

      if (result && result.rows && result.rows.length > 0 && result.rows[0].test === 1) {
        return {
          component: 'Database Connection',
          status: 'healthy',
          message: 'Database connection is working properly'
        };
      } else {
        return {
          component: 'Database Connection',
          status: 'error',
          message: 'Database query returned unexpected results'
        };
      }
    } catch (error) {
      return {
        component: 'Database Connection',
        status: 'error',
        message: `Database connection failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        details: { error }
      };
    }
  }

  /**
   * Check R2 backup service connectivity and configuration
   */
  private async checkR2BackupService(): Promise<HealthCheckResult> {
    try {
      const isEnabled = process.env.BACKUP_ENABLED === 'true';
      
      if (!isEnabled) {
        return {
          component: 'R2 Backup Service',
          status: 'warning',
          message: 'R2 backup service is disabled (BACKUP_ENABLED=false)'
        };
      }

      const isConnected = await this.r2BackupService.testConnection();
      
      if (isConnected) {
        return {
          component: 'R2 Backup Service',
          status: 'healthy',
          message: 'R2 backup service is configured and accessible'
        };
      } else {
        return {
          component: 'R2 Backup Service',
          status: 'error',
          message: 'R2 backup service is enabled but connection failed'
        };
      }
    } catch (error) {
      return {
        component: 'R2 Backup Service',
        status: 'error',
        message: `R2 backup service error: ${error instanceof Error ? error.message : 'Unknown error'}`,
        details: { error }
      };
    }
  }

  /**
   * Check payment service configuration
   */
  private async checkPaymentService(): Promise<HealthCheckResult> {
    const stripeSecretKey = process.env.STRIPE_SECRET_KEY;
    const stripeWebhookSecret = process.env.STRIPE_WEBHOOK_SECRET;

    if (!stripeSecretKey || !stripeWebhookSecret) {
      return {
        component: 'Payment Service',
        status: 'warning',
        message: 'Stripe payment service is not fully configured',
        details: {
          hasSecretKey: !!stripeSecretKey,
          hasWebhookSecret: !!stripeWebhookSecret
        }
      };
    }

    // Validate key format
    if (!stripeSecretKey.startsWith('sk_')) {
      return {
        component: 'Payment Service',
        status: 'error',
        message: 'Invalid Stripe secret key format'
      };
    }

    return {
      component: 'Payment Service',
      status: 'healthy',
      message: 'Payment service is properly configured'
    };
  }

  /**
   * Check authentication service configuration
   */
  private async checkAuthConfiguration(): Promise<HealthCheckResult> {
    const jwtSecret = process.env.JWT_SECRET;
    const googleClientId = process.env.GOOGLE_CLIENT_ID;
    const googleClientSecret = process.env.GOOGLE_CLIENT_SECRET;
    const linkedinClientId = process.env.LINKEDIN_CLIENT_ID;
    const linkedinClientSecret = process.env.LINKEDIN_CLIENT_SECRET;

    if (!jwtSecret) {
      return {
        component: 'Authentication Service',
        status: 'error',
        message: 'JWT_SECRET is required for authentication'
      };
    }

    if (jwtSecret.length < 32) {
      return {
        component: 'Authentication Service',
        status: 'warning',
        message: 'JWT_SECRET should be at least 32 characters long for security'
      };
    }

    const hasGoogleOAuth = googleClientId && googleClientSecret;
    const hasLinkedInOAuth = linkedinClientId && linkedinClientSecret;

    if (!hasGoogleOAuth && !hasLinkedInOAuth) {
      return {
        component: 'Authentication Service',
        status: 'warning',
        message: 'No OAuth providers configured (Google or LinkedIn)',
        details: { hasGoogleOAuth, hasLinkedInOAuth }
      };
    }

    return {
      component: 'Authentication Service',
      status: 'healthy',
      message: 'Authentication service is properly configured',
      details: { hasGoogleOAuth, hasLinkedInOAuth }
    };
  }

  /**
   * Check database migrations status
   */
  private async checkDatabaseMigrations(): Promise<HealthCheckResult> {
    try {
      await this.migrationService.initialize();
      return await this.migrationService.createHealthCheckResult();
    } catch (error) {
      return {
        component: 'Database Migrations',
        status: 'error',
        message: `Migration service initialization failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        details: { error }
      };
    }
  }

  /**
   * Log health check results with visual indicators
   */
  private logHealthCheckResults(summary: HealthCheckSummary): void {
    logger.info('='.repeat(60));
    logger.info('🏥 STARTUP HEALTH CHECK RESULTS');
    logger.info('='.repeat(60));

    // Overall status
    const overallIcon = summary.overall === 'healthy' ? '✅' : 
                       summary.overall === 'warning' ? '⚠️' : '❌';
    logger.info(`Overall Status: ${overallIcon} ${summary.overall.toUpperCase()}`);
    logger.info('');

    // Individual component results
    summary.results.forEach(result => {
      const icon = result.status === 'healthy' ? '✅' : 
                  result.status === 'warning' ? '⚠️' : '❌';
      logger.info(`${icon} ${result.component}: ${result.message}`);
    });

    // Critical errors
    if (summary.criticalErrors.length > 0) {
      logger.info('');
      logger.error('❌ CRITICAL ERRORS:');
      summary.criticalErrors.forEach(error => {
        logger.error(`   • ${error}`);
      });
    }

    // Warnings
    if (summary.warnings.length > 0) {
      logger.info('');
      logger.warn('⚠️  WARNINGS:');
      summary.warnings.forEach(warning => {
        logger.warn(`   • ${warning}`);
      });
    }

    logger.info('='.repeat(60));
  }

  /**
   * Validate that system is ready for production
   */
  validateProductionReadiness(summary: HealthCheckSummary): boolean {
    if (summary.overall === 'error') {
      logger.error('❌ System is NOT ready for production due to critical errors');
      logger.error('Please fix the following issues before starting:');
      summary.criticalErrors.forEach(error => {
        logger.error(`   • ${error}`);
      });
      return false;
    }

    if (summary.overall === 'warning') {
      logger.warn('⚠️  System has warnings but can start');
      logger.warn('Consider addressing these issues:');
      summary.warnings.forEach(warning => {
        logger.warn(`   • ${warning}`);
      });
    }

    logger.info('✅ System is ready to start');
    return true;
  }
}