#!/usr/bin/env node

/**
 * Documentation Drift Prevention Monitor
 * Monitors for documentation drift and generates reports
 */

import { readFileSync, writeFileSync, existsSync } from 'fs';
import { execSync } from 'child_process';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const PROJECT_ROOT = dirname(__dirname);

class DocumentationDriftMonitor {
  constructor() {
    this.report = {
      timestamp: new Date().toISOString(),
      summary: {
        drift_detected: false,
        issues_found: 0,
        warnings: 0,
        last_check: null
      },
      issues: [],
      metrics: {},
      recommendations: []
    };
  }

  /**
   * Check for route documentation drift
   */
  checkRouteDrift() {
    try {
      // Get current routes from validation script
      const validationOutput = execSync('npm run validate:docs', {
        encoding: 'utf8',
        cwd: PROJECT_ROOT
      });

      const hasErrors = validationOutput.includes('❌') || validationOutput.includes('Error');
      const hasWarnings = validationOutput.includes('⚠️');

      if (hasErrors) {
        this.report.issues.push({
          type: 'documentation_accuracy',
          severity: 'error',
          message: 'Documentation validation errors detected',
          details: 'Run npm run validate:docs to see detailed errors'
        });
        this.report.summary.drift_detected = true;
      }

      if (hasWarnings) {
        this.report.issues.push({
          type: 'documentation_accuracy',
          severity: 'warning',
          message: 'Documentation validation warnings detected',
          details: 'Some routes may be undocumented or documentation may be outdated'
        });
        this.report.summary.warnings++;
      }

      // Count specific issues
      const undocumentedMatch = validationOutput.match(/(\d+) undocumented routes found/);
      if (undocumentedMatch) {
        this.report.issues.push({
          type: 'missing_documentation',
          severity: 'warning',
          message: `${undocumentedMatch[1]} routes not documented`,
          details: 'Consider adding these routes to API documentation'
        });
      }

      const missingTablesMatch = validationOutput.match(/Missing required database tables: ([^]+)/);
      if (missingTablesMatch) {
        this.report.issues.push({
          type: 'schema_documentation',
          severity: 'error',
          message: `Missing database tables: ${missingTablesMatch[1]}`,
          details: 'Update database schema to include missing tables'
        });
        this.report.summary.drift_detected = true;
      }

    } catch (error) {
      this.report.issues.push({
        type: 'validation_failure',
        severity: 'error',
        message: 'Documentation validation failed to run',
        details: error.message
      });
      this.report.summary.drift_detected = true;
    }
  }

  /**
   * Check for version consistency drift
   */
  checkVersionConsistency() {
    try {
      const filesToCheck = [
        { file: 'CLAUDE.md', pattern: /Phase (\d+)/, name: 'Main Project Info' },
        { file: 'docs/systems/DEVELOPMENT_STATUS.md', pattern: /Phase (\d+)/, name: 'Development Status' },
        { file: 'docs/systems/COMPLETED_SYSTEMS.md', pattern: /(\d+)\/(\d+) major system components/, name: 'Completed Systems' }
      ];

      const extractedInfo = [];

      filesToCheck.forEach(({ file, pattern, name }) => {
        try {
          const content = readFileSync(join(PROJECT_ROOT, file), 'utf8');
          const match = content.match(pattern);
          if (match) {
            extractedInfo.push({
              file,
              name,
              value: match[1],
              line: this.getLineNumber(content, match[0])
            });
          }
        } catch (error) {
          this.report.issues.push({
            type: 'file_access',
            severity: 'warning',
            message: `Could not read ${file}: ${error.message}`,
            details: 'File may be missing or inaccessible'
          });
        }
      });

      // Check for phase inconsistencies
      const phaseValues = extractedInfo.filter(info => info.name.includes('Phase'));
      const uniquePhases = [...new Set(phaseValues.map(info => info.value))];

      if (uniquePhases.length > 1) {
        this.report.issues.push({
          type: 'version_inconsistency',
          severity: 'warning',
          message: `Version inconsistency detected: ${uniquePhases.join(', ')}`,
          details: phaseValues.map(info => `${info.name}: Phase ${info.value}`).join('; ')
        });
      }

      // Check for system count inconsistencies
      const systemCountMatches = extractedInfo.filter(info => info.name.includes('Systems'));
      const uniqueSystemCounts = [...new Set(systemCountMatches.map(info => info.value))];

      if (uniqueSystemCounts.length > 1) {
        this.report.issues.push({
          type: 'version_inconsistency',
          severity: 'warning',
          message: `System count inconsistency: ${uniqueSystemCounts.join(', ')}`,
          details: systemCountMatches.map(info => `${info.name}: ${info.value}`).join('; ')
        });
      }

      this.report.metrics.versionConsistency = {
        filesChecked: extractedInfo.length,
        phaseValues: uniquePhases,
        systemCounts: uniqueSystemCounts,
        inconsistencies: uniquePhases.length > 1 || uniqueSystemCounts.length > 1
      };

    } catch (error) {
      this.report.issues.push({
        type: 'consistency_check_failed',
        severity: 'error',
        message: 'Version consistency check failed',
        details: error.message
      });
    }
  }

  /**
   * Check for file structure drift
   */
  checkFileStructure() {
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

    const missingFiles = [];
    const optionalFiles = [];

    requiredFiles.forEach(file => {
      const filePath = join(PROJECT_ROOT, file);
      if (!existsSync(filePath)) {
        missingFiles.push(file);
      }
    });

    // Check for optional but expected files
    const expectedFiles = [
      'docs/api/API_AUTOGENERATED.md',
      'docs/api/api-data.json',
      'docs/DOCUMENTATION_GOVERNANCE.md',
      'scripts/validate-docs.js',
      'scripts/generate-api-docs.js',
      '.husky/pre-commit'
    ];

    expectedFiles.forEach(file => {
      const filePath = join(PROJECT_ROOT, file);
      if (!existsSync(filePath)) {
        optionalFiles.push(file);
      }
    });

    if (missingFiles.length > 0) {
      this.report.issues.push({
        type: 'missing_files',
        severity: 'error',
        message: `${missingFiles.length} required files missing`,
        details: `Missing: ${missingFiles.join(', ')}`
      });
      this.report.summary.drift_detected = true;
    }

    if (optionalFiles.length > 0) {
      this.report.issues.push({
        type: 'missing_optional_files',
        severity: 'info',
        message: `${optionalFiles.length} optional files missing`,
        details: `Missing: ${optionalFiles.join(', ')}`
      });
    }

    this.report.metrics.fileStructure = {
      requiredFiles: requiredFiles.length,
      optionalFiles: expectedFiles.length,
      missingRequired: missingFiles.length,
      missingOptional: optionalFiles.length
    };
  }

  /**
   * Get line number for a match in content
   */
  getLineNumber(content, searchString) {
    const lines = content.split('\n');
    for (let i = 0; i < lines.length; i++) {
      if (lines[i].includes(searchString)) {
        return i + 1;
      }
    }
    return -1;
  }

  /**
   * Generate recommendations based on findings
   */
  generateRecommendations() {
    // Add recommendations based on issues found
    if (this.report.summary.drift_detected) {
      this.report.recommendations.push({
        priority: 'high',
        action: 'Fix documentation accuracy issues',
        description: 'Run npm run validate:docs to identify and fix issues',
        command: 'npm run validate:docs'
      });
    }

    if (this.report.summary.warnings > 0) {
      this.report.recommendations.push({
        priority: 'medium',
        action: 'Address documentation warnings',
        description: 'Review and fix documentation warnings to improve quality',
        command: 'npm run validate:docs'
      });
    }

    // Check for tooling recommendations
    if (!existsSync(join(PROJECT_ROOT, '.husky/pre-commit'))) {
      this.report.recommendations.push({
        priority: 'low',
        action: 'Set up pre-commit hooks',
        description: 'Install pre-commit hooks to prevent documentation drift',
        command: 'npm install husky && npx husky install'
      });
    }

    if (!existsSync(join(PROJECT_ROOT, 'docs/api/API_AUTOGENERATED.md'))) {
      this.report.recommendations.push({
        priority: 'low',
        action: 'Generate API documentation',
        description: 'Generate current API documentation from code',
        command: 'npm run generate:api-docs'
      });
    }

    // Add routine maintenance recommendations
    this.report.recommendations.push({
      priority: 'low',
      action: 'Run regular validation',
      description: 'Run documentation validation regularly to prevent drift',
      command: 'npm run validate:docs',
      frequency: 'Weekly'
    });

    this.report.recommendations.push({
      priority: 'low',
      action: 'Update API documentation',
      description: 'Regenerate API documentation after route changes',
      command: 'npm run generate:api-docs',
      frequency: 'After route changes'
    });
  }

  /**
   * Generate monitoring report
   */
  generateReport() {
    // Update summary
    this.report.summary.issues_found = this.report.issues.length;
    this.report.summary.last_check = new Date().toISOString();

    // Add metrics
    this.report.metrics.total_issues = this.report.issues.length;
    this.report.metrics.issues_by_type = {};
    this.report.issues.forEach(issue => {
      if (!this.report.metrics.issues_by_type[issue.type]) {
        this.report.metrics.issues_by_type[issue.type] = 0;
      }
      this.report.metrics.issues_by_type[issue.type]++;
    });

    this.report.metrics.issues_by_severity = {};
    this.report.issues.forEach(issue => {
      if (!this.report.metrics.issues_by_severity[issue.severity]) {
        this.report.metrics.issues_by_severity[issue.severity] = 0;
      }
      this.report.metrics.issues_by_severity[issue.severity]++;
    });

    // Save report
    const reportPath = join(PROJECT_ROOT, 'docs', 'DRIFT_MONITORING_REPORT.json');
    writeFileSync(reportPath, JSON.stringify(this.report, null, 2), 'utf8');

    // Generate human-readable summary
    const summaryPath = join(PROJECT_ROOT, 'docs', 'DRIFT_SUMMARY.md');
    const summary = this.generateHumanReadableSummary();
    writeFileSync(summaryPath, summary, 'utf8');

    return { reportPath, summaryPath };
  }

  /**
   * Generate human-readable summary
   */
  generateHumanReadableSummary() {
    const timestamp = new Date().toISOString().split('T')[0];
    let summary = `# Documentation Drift Monitoring Summary

**Date**: ${timestamp}
**Status**: ${this.report.summary.drift_detected ? '🚨 Issues Detected' : '✅ No Drift Detected'}

## Summary
- **Issues Found**: ${this.report.summary.issues_found}
- **Warnings**: ${this.report.summary.warnings}
- **Last Check**: ${this.report.summary.last_check}

`;

    if (this.report.issues.length > 0) {
      summary += `\n## Issues Found\n\n`;

      // Group issues by severity
      const errors = this.report.issues.filter(i => i.severity === 'error');
      const warnings = this.report.issues.filter(i => i.severity === 'warning');
      const info = this.report.issues.filter(i => i.severity === 'info');

      if (errors.length > 0) {
        summary += `### 🚨 Errors (${errors.length})\n\n`;
        errors.forEach((issue, index) => {
          summary += `${index + 1}. **${issue.type}**: ${issue.message}\n`;
          if (issue.details) {
            summary += `   - ${issue.details}\n`;
          }
        });
        summary += '\n';
      }

      if (warnings.length > 0) {
        summary += `### ⚠️ Warnings (${warnings.length})\n\n`;
        warnings.forEach((issue, index) => {
          summary += `${index + 1}. **${issue.type}**: ${issue.message}\n`;
          if (issue.details) {
            summary += `   - ${issue.details}\n`;
          }
        });
        summary += '\n';
      }

      if (info.length > 0) {
        summary += `### ℹ️ Information (${info.length})\n\n`;
        info.forEach((issue, index) => {
          summary += `${index + 1}. **${issue.type}**: ${issue.message}\n`;
          if (issue.details) {
            summary += `   - ${issue.details}\n`;
          }
        });
        summary += '\n';
      }
    }

    if (this.report.recommendations.length > 0) {
      summary += `## Recommendations\n\n`;

      this.report.recommendations.forEach((rec, index) => {
        const emoji = rec.priority === 'high' ? '🔴' : rec.priority === 'medium' ? '🟡' : '🟢';
        summary += `${index + 1}. ${emoji} **${rec.action}** (${rec.priority} priority)\n`;
        summary += `   ${rec.description}\n`;
        summary += `   **Command**: \`${rec.command}\`\n`;
        if (rec.frequency) {
          summary += `   **Frequency**: ${rec.frequency}\n`;
        }
        summary += '\n';
      });
    }

    summary += `## Quick Fix Commands\n\n`;
    summary += `\`\`\`bash\n# Validate documentation accuracy\nnpm run validate:docs\n\n# Generate API documentation\nnpm run generate:api-docs\n\n# Run all validations\nnpm run validate:all\n\`\`\n\n`;
    summary += `---\n`;
    summary += `*This report was automatically generated by the documentation drift prevention system*\n`;
    summary += `*Last updated: ${new Date().toISOString()}\n`;

    return summary;
  }

  /**
   * Run monitoring process
   */
  async run() {
    console.log('🔍 Starting Documentation Drift Monitoring...\n');

    console.log('📋 Checking route documentation accuracy...');
    this.checkRouteDrift();

    console.log('🔢 Checking version consistency...');
    this.checkVersionConsistency();

    console.log('📁 Checking file structure...');
    this.checkFileStructure();

    console.log('💡 Generating recommendations...');
    this.generateRecommendations();

    const { reportPath, summaryPath } = this.generateReport();

    console.log('\n📊 Monitoring Summary:');
    console.log(`  Status: ${this.report.summary.drift_detected ? '🚨 Issues Detected' : '✅ No Drift Detected'}`);
    console.log(`  Issues Found: ${this.report.summary.issues_found}`);
    console.log(`  Warnings: ${this.report.summary.warnings}`);
    console.log(`\n📄 Reports Generated:`);
    console.log(`  ${reportPath}`);
    console.log(`  ${summaryPath}`);

    if (this.report.summary.drift_detected) {
      console.log('\n❌ Documentation drift detected!');
      console.log('   Please review the recommendations above to fix issues');
      process.exit(1);
    } else if (this.report.summary.warnings > 0) {
      console.log('\n⚠️  Documentation warnings detected');
      console.log('   Consider addressing warnings to improve quality');
      process.exit(2);
    } else {
      console.log('\n✅ No documentation drift detected');
      console.log('   Documentation is in good shape');
      process.exit(0);
    }
  }
}

// Run monitor if this script is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  const monitor = new DocumentationDriftMonitor();
  monitor.run().catch(error => {
    console.error('Monitoring failed:', error);
    process.exit(1);
  });
}

export default DocumentationDriftMonitor;