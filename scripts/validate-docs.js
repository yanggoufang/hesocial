#!/usr/bin/env node

/**
 * Documentation Validation Script
 * Validates that all documentation accurately reflects the current implementation
 */

import { readFileSync, readdirSync, existsSync, writeFileSync } from 'fs';
import { execSync } from 'child_process';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const PROJECT_ROOT = dirname(__dirname);

class DocumentationValidator {
  constructor() {
    this.errors = [];
    this.warnings = [];
    this.info = [];
    this.packageJson = null;
    this.routes = [];
    this.endpoints = [];
  }

  log(message, type = 'info') {
    const timestamp = new Date().toISOString();
    const logEntry = `[${timestamp}] ${type.toUpperCase()}: ${message}`;

    switch (type) {
      case 'error':
        this.errors.push(logEntry);
        console.error(`❌ ${message}`);
        break;
      case 'warning':
        this.warnings.push(logEntry);
        console.warn(`⚠️  ${message}`);
        break;
      case 'info':
        this.info.push(logEntry);
        console.log(`ℹ️  ${message}`);
        break;
      default:
        console.log(`✅ ${message}`);
    }
  }

  /**
   * Load and parse package.json files
   */
  loadPackageJson() {
    try {
      this.packageJson = {
        root: JSON.parse(readFileSync(join(PROJECT_ROOT, 'package.json'), 'utf8')),
        backend: JSON.parse(readFileSync(join(PROJECT_ROOT, 'backend', 'package.json'), 'utf8')),
        frontend: JSON.parse(readFileSync(join(PROJECT_ROOT, 'frontend', 'package.json'), 'utf8'))
      };
      this.log('Loaded package.json files');
    } catch (error) {
      this.log(`Failed to load package.json: ${error.message}`, 'error');
    }
  }

  /**
   * Validate that all documented npm scripts actually exist
   */
  validateNpmScripts() {
    if (!this.packageJson) return;

    const rootScripts = this.packageJson.root.scripts || {};
    const documentedCommands = this.getDocumentedCommands();

    Object.keys(documentedCommands).forEach(command => {
      if (!rootScripts[command]) {
        this.log(`Documented command '${command}' not found in root package.json`, 'error');
      }
    });

    // Check for extra scripts that might need documentation
    Object.keys(rootScripts).forEach(script => {
      if (!documentedCommands[script] && !script.includes(':')) {
        this.log(`Script '${script}' exists but might not be documented`, 'warning');
      }
    });

    this.log('Validated npm scripts');
  }

  /**
   * Extract documented commands from command documentation
   */
  getDocumentedCommands() {
    const documentedCommands = {};

    try {
      const commandDocs = readFileSync(
        join(PROJECT_ROOT, 'docs', 'commands', 'DEVELOPMENT_COMMANDS_UPDATED.md'),
        'utf8'
      );

      // Extract npm run commands using regex
      const commandMatches = commandDocs.match(/npm run (\w+)/g) || [];

      commandMatches.forEach(match => {
        const command = match.replace('npm run ', '');
        documentedCommands[command] = true;
      });

    } catch (error) {
      this.log(`Could not read command documentation: ${error.message}`, 'warning');
    }

    return documentedCommands;
  }

  /**
   * Load and analyze route files
   */
  loadRoutes() {
    try {
      const routesDir = join(PROJECT_ROOT, 'backend', 'src', 'routes');
      const routeFiles = readdirSync(routesDir).filter(file => file.endsWith('.ts'));

      for (const file of routeFiles) {
        const filePath = join(routesDir, file);
        const content = readFileSync(filePath, 'utf8');

        // Extract route definitions
        const routeMatches = content.match(/router\.(get|post|put|delete|patch)\s*\(\s*['"`]([^'"`]+)['"`]/g) || [];

        routeMatches.forEach(match => {
          const methodMatch = match.match(/(get|post|put|delete|patch)/);
          const pathMatch = match.match(/['"`]([^'"`]+)['"`]/);

          if (methodMatch && pathMatch) {
            this.routes.push({
              file,
              method: methodMatch[1].toUpperCase(),
              path: pathMatch[1],
              content: match
            });
          }
        });
      }

      this.log(`Loaded ${this.routes.length} routes from ${routeFiles.length} files`);
    } catch (error) {
      this.log(`Failed to load routes: ${error.message}`, 'error');
    }
  }

  /**
   * Load API endpoints from documentation
   */
  loadApiDocumentation() {
    try {
      const apiDocs = readFileSync(
        join(PROJECT_ROOT, 'docs', 'api', 'API_REFERENCE_UPDATED.md'),
        'utf8'
      );

      // Extract endpoint definitions
      const endpointMatches = apiDocs.match(/### \*\*(GET|POST|PUT|DELETE|PATCH)\*\*\s+\/api\/([^\s\n]+)/g) || [];

      endpointMatches.forEach(match => {
        const methodMatch = match.match(/(GET|POST|PUT|DELETE|PATCH)/);
        const pathMatch = match.match(/\/api\/([^\s\n]+)/);

        if (methodMatch && pathMatch) {
          this.endpoints.push({
            method: methodMatch[1],
            path: pathMatch[1],
            documented: true
          });
        }
      });

      this.log(`Loaded ${this.endpoints.length} documented endpoints`);
    } catch (error) {
      this.log(`Failed to load API documentation: ${error.message}`, 'error');
    }
  }

  /**
   * Validate route documentation coverage
   */
  validateRouteDocumentation() {
    const undocumentedRoutes = [];
    const overdocumentedEndpoints = [];

    // Check for routes not in documentation
    this.routes.forEach(route => {
      const documented = this.endpoints.some(endpoint =>
        endpoint.method === route.method &&
        endpoint.path === route.path
      );

      if (!documented) {
        // Skip certain internal/debug routes
        if (!route.path.includes('debug') && !route.path.includes('legacy')) {
          undocumentedRoutes.push(route);
        }
      }
    });

    // Check for documented endpoints not in routes
    this.endpoints.forEach(endpoint => {
      const exists = this.routes.some(route =>
        route.method === endpoint.method &&
        route.path === endpoint.path
      );

      if (!exists) {
        overdocumentedEndpoints.push(endpoint);
      }
    });

    if (undocumentedRoutes.length > 0) {
      this.log(`${undocumentedRoutes.length} undocumented routes found:`, 'warning');
      undocumentedRoutes.forEach(route => {
        this.log(`  ${route.method} /api/${route.path} (${route.file})`, 'warning');
      });
    }

    if (overdocumentedEndpoints.length > 0) {
      this.log(`${overdocumentedEndpoints.length} documented endpoints not implemented:`, 'warning');
      overdocumentedEndpoints.forEach(endpoint => {
        this.log(`  ${endpoint.method} /api/${endpoint.path}`, 'warning');
      });
    }

    if (undocumentedRoutes.length === 0 && overdocumentedEndpoints.length === 0) {
      this.log('All routes are properly documented');
    }
  }

  /**
   * Validate database schema consistency
   */
  validateDatabaseSchema() {
    try {
      const schemaFile = join(PROJECT_ROOT, 'database', 'duckdb-schema.sql');
      const schemaContent = readFileSync(schemaFile, 'utf8');

      // Extract table names from schema
      const tableMatches = schemaContent.match(/CREATE TABLE\s+IF\s+NOT\s+EXISTS\s+(\w+)/g) || [];
      const schemaTables = tableMatches.map(match =>
        match.match(/CREATE TABLE.*\s+(\w+)/)[1]
      );

      // Check for participant privacy tables specifically mentioned in routes
      const requiredTables = [
        'users',
        'events',
        'event_registrations',
        'event_privacy_overrides',
        'event_participant_access',
        'participant_view_logs',
        'sales_leads',
        'sales_opportunities'
      ];

      const missingTables = requiredTables.filter(table =>
        !schemaTables.includes(table)
      );

      if (missingTables.length > 0) {
        this.log(`Missing required database tables: ${missingTables.join(', ')}`, 'error');
      } else {
        this.log('All required database tables are present in schema');
      }

    } catch (error) {
      this.log(`Failed to validate database schema: ${error.message}`, 'error');
    }
  }

  /**
   * Validate file structure consistency
   */
  validateFileStructure() {
    const requiredFiles = [
      'package.json',
      'backend/package.json',
      'frontend/package.json',
      'backend/src/server.ts',
      'database/duckdb-schema.sql',
      'docs/api/API_REFERENCE_UPDATED.md',
      'docs/commands/DEVELOPMENT_COMMANDS_UPDATED.md',
      'docs/FEATURE_STATUS_MATRIX.md',
      'CLAUDE.md'
    ];

    const missingFiles = requiredFiles.filter(file =>
      !existsSync(join(PROJECT_ROOT, file))
    );

    if (missingFiles.length > 0) {
      this.log(`Missing required files: ${missingFiles.join(', ')}`, 'error');
    } else {
      this.log('All required files are present');
    }

    // Check for orphaned documentation files
    const docsDir = join(PROJECT_ROOT, 'docs');
    if (existsSync(docsDir)) {
      const docFiles = readdirSync(docsDir, { recursive: true })
        .filter(file => file.endsWith('.md'))
        .map(file => join('docs', file));

      // Check for documentation that references non-existent files
      docFiles.forEach(docFile => {
        try {
          const content = readFileSync(join(PROJECT_ROOT, docFile), 'utf8');

          // Look for file references that might not exist
          const fileRefs = content.match(/\[([^\]]+)\]\(([^)]+)\)/g) || [];

          fileRefs.forEach(ref => {
            const match = ref.match(/\[([^\]]+)\]\(([^)]+)\)/);
            if (match && match[2].startsWith('./')) {
              const refPath = join(dirname(docFile), match[2]);
              if (!existsSync(join(PROJECT_ROOT, refPath))) {
                this.log(`Broken reference in ${docFile}: ${refPath}`, 'warning');
              }
            }
          });
        } catch (error) {
          // Skip files that can't be read
        }
      });
    }
  }

  /**
   * Validate version consistency across documentation
   */
  validateVersionConsistency() {
    const versionFiles = [
      { file: 'CLAUDE.md', pattern: /Phase (\d+)/ },
      { file: 'docs/systems/DEVELOPMENT_STATUS.md', pattern: /Phase (\d+)/ }
    ];

    const phases = [];

    versionFiles.forEach(({ file, pattern }) => {
      try {
        const content = readFileSync(join(PROJECT_ROOT, file), 'utf8');
        const match = content.match(pattern);
        if (match) {
          phases.push({ file, phase: parseInt(match[1]) });
        }
      } catch (error) {
        this.log(`Could not read ${file}: ${error.message}`, 'warning');
      }
    });

    if (phases.length > 1) {
      const uniquePhases = [...new Set(phases.map(p => p.phase))];
      if (uniquePhases.length > 1) {
        this.log('Version inconsistency detected:', 'warning');
        phases.forEach(({ file, phase }) => {
          this.log(`  ${file}: Phase ${phase}`, 'warning');
        });
      } else {
        this.log(`Version consistency: All files reference Phase ${uniquePhases[0]}`);
      }
    }
  }

  /**
   * Generate validation report
   */
  generateReport() {
    const report = {
      timestamp: new Date().toISOString(),
      summary: {
        errors: this.errors.length,
        warnings: this.warnings.length,
        info: this.info.length
      },
      details: {
        errors: this.errors,
        warnings: this.warnings,
        info: this.info
      },
      metrics: {
        routesFound: this.routes.length,
        endpointsDocumented: this.endpoints.length,
        packageJsonLoaded: !!this.packageJson
      }
    };

    const reportPath = join(PROJECT_ROOT, 'docs', 'validation-report.json');
    writeFileSync(reportPath, JSON.stringify(report, null, 2));

    return report;
  }

  /**
   * Run all validation checks
   */
  async run() {
    console.log('🔍 Starting Documentation Validation...\n');

    this.loadPackageJson();
    this.loadRoutes();
    this.loadApiDocumentation();

    this.validateNpmScripts();
    this.validateRouteDocumentation();
    this.validateDatabaseSchema();
    this.validateFileStructure();
    this.validateVersionConsistency();

    const report = this.generateReport();

    console.log('\n📊 Validation Summary:');
    console.log(`  Errors: ${report.summary.errors}`);
    console.log(`  Warnings: ${report.summary.warnings}`);
    console.log(`  Info: ${report.summary.info}`);

    if (report.summary.errors > 0) {
      console.log('\n❌ Validation failed with errors');
      process.exit(1);
    } else if (report.summary.warnings > 0) {
      console.log('\n⚠️  Validation passed with warnings');
      process.exit(2);
    } else {
      console.log('\n✅ All validations passed');
      process.exit(0);
    }
  }
}

// Run validation if this script is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  const validator = new DocumentationValidator();
  validator.run().catch(error => {
    console.error('Validation failed:', error);
    process.exit(1);
  });
}

export default DocumentationValidator;