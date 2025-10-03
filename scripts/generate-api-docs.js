#!/usr/bin/env node

/**
 * Automated API Documentation Generator
 * Scans route files and generates comprehensive API documentation
 */

import { readFileSync, writeFileSync, readdirSync, existsSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const PROJECT_ROOT = dirname(__dirname);

class ApiDocumentationGenerator {
  constructor() {
    this.routes = [];
    this.endpoints = [];
    this.categories = {};
    this.stats = {
      totalEndpoints: 0,
      methods: {},
      files: {}
    };
  }

  /**
   * Extract JSDoc comments from route files
   */
  extractJSDoc(content, filePath) {
    const jsdocMatches = content.match(/\/\*\*[\s\S]*?\*\//g) || [];
    const comments = [];

    jsdocMatches.forEach(match => {
      // Clean up the JSDoc comment
      const cleaned = match
        .replace(/\/\*\*|\*\//g, '')
        .replace(/^\s*\*\s?/gm, '')
        .trim();

      // Parse JSDoc tags
      const tags = {};
      const tagMatches = cleaned.match(/@(\w+)\s+(.+?)(?=\s@\w+|$)/g) || [];

      tagMatches.forEach(tagMatch => {
        const tagParts = tagMatch.match(/@(\w+)\s+(.+)/);
        if (tagParts) {
          tags[tagParts[1]] = tagParts[2].trim();
        }
      });

      // Extract description (text before first tag)
      const description = cleaned.split(/\s*@\w+/)[0].trim();

      comments.push({
        description,
        tags
      });
    });

    return comments;
  }

  /**
   * Load and analyze route files
   */
  loadRoutes() {
    try {
      const routesDir = join(PROJECT_ROOT, 'backend', 'src', 'routes');
      const routeFiles = readdirSync(routesDir).filter(file => file.endsWith('.ts'));

      for (const file of routeFiles) {
        if (file === 'index.ts') continue; // Skip main router file

        const filePath = join(routesDir, file);
        const content = readFileSync(filePath, 'utf8');
        const comments = this.extractJSDoc(content, filePath);

        // Extract route definitions
        const routeMatches = content.match(/router\.(get|post|put|delete|patch)\s*\(\s*['"`]([^'"`]+)['"`]/g) || [];

        routeMatches.forEach(match => {
          const methodMatch = match.match(/(get|post|put|delete|patch)/);
          const pathMatch = match.match(/['"`]([^'"`]+)['"`]/);

          if (methodMatch && pathMatch) {
            const route = {
              file,
              filePath,
              method: methodMatch[1].toUpperCase(),
              path: pathMatch[1],
              fullPath: `/api/${pathMatch[1]}`,
              description: '',
              accessLevel: 'Public',
              parameters: [],
              responses: [],
              examples: [],
              tags: []
            };

            // Find preceding JSDoc comment
            const matchIndex = content.indexOf(match);
            const beforeMatch = content.substring(0, matchIndex);
            const lastJSDocIndex = beforeMatch.lastIndexOf('/**');

            if (lastJSDocIndex !== -1) {
              const jsd = content.substring(lastJSDocIndex, matchIndex);
              const jsdMatch = jsd.match(/\/\*\*[\s\S]*?\*\//);
              if (jsdMatch) {
                const comments = this.extractJSDoc(jsdMatch[0], filePath);
                if (comments.length > 0) {
                  const comment = comments[0];
                  route.description = comment.description;
                  route.accessLevel = comment.tags.access || 'Public';
                  route.tags = comment.tags.tags ? comment.tags.tags.split(',').map(t => t.trim()) : [];
                }
              }
            }

            this.routes.push(route);
          }
        });

        // Update statistics
        this.stats.files[file] = routeMatches.length;
      }

      this.log(`Loaded ${this.routes.length} routes from ${routeFiles.length - 1} files`);

    } catch (error) {
      this.log(`Failed to load routes: ${error.message}`, 'error');
    }
  }

  /**
   * Categorize routes based on file names and paths
   */
  categorizeRoutes() {
    this.routes.forEach(route => {
      let category = 'General';

      // Determine category based on file name
      if (route.file.includes('auth')) {
        category = 'Authentication';
      } else if (route.file.includes('event')) {
        category = 'Event Management';
      } else if (route.file.includes('registration')) {
        category = 'Registration System';
      } else if (route.file.includes('participant')) {
        category = 'Social Features';
      } else if (route.file.includes('sales')) {
        category = 'Sales Management';
      } else if (route.file.includes('analytics')) {
        category = 'Business Intelligence';
      } else if (route.file.includes('health') || route.file.includes('system')) {
        category = 'System Monitoring';
      } else if (route.file.includes('admin')) {
        category = 'Administration';
      } else if (route.file.includes('deployment')) {
        category = 'Deployment';
      } else if (route.file.includes('emergency')) {
        category = 'Emergency Operations';
      } else if (route.file.includes('media')) {
        category = 'Media Management';
      } else if (route.file.includes('debug')) {
        category = 'Development';
      }

      if (!this.categories[category]) {
        this.categories[category] = [];
      }

      this.categories[category].push(route);

      // Update method statistics
      this.stats.methods[route.method] = (this.stats.methods[route.method] || 0) + 1;
    });

    this.stats.totalEndpoints = this.routes.length;
  }

  /**
   * Generate API documentation in Markdown format
   */
  generateMarkdownDocs() {
    let markdown = `# API Documentation - HeSocial Platform

**Generated**: ${new Date().toISOString()}
**Total Endpoints**: ${this.stats.totalEndpoints}
**Route Files**: ${Object.keys(this.stats.files).length}

---

## 🔐 **Authentication**

All API endpoints require authentication unless specified as **Public**.

### **Authentication Methods**
- **JWT Token**: Include in \`Authorization: Bearer <token>\` header
- **Development Token**: Use \`dev-token-12345\` for development testing
- **Google OAuth 2.0**: Redirect-based authentication

### **Role-Based Access**
- **Public**: No authentication required
- **User**: Authenticated user access
- **Admin**: Admin role required
- **Super Admin**: Super admin role required

---

`;

    // Generate sections for each category
    Object.keys(this.categories).sort().forEach(category => {
      markdown += `## ${this.getCategoryEmoji(category)} **${category}**\n\n`;

      const categoryRoutes = this.categories[category];

      // Group by method within category
      const groupedRoutes = {};
      categoryRoutes.forEach(route => {
        if (!groupedRoutes[route.method]) {
          groupedRoutes[route.method] = [];
        }
        groupedRoutes[route.method].push(route);
      });

      // Sort methods: GET, POST, PUT, DELETE, PATCH
      const methodOrder = ['GET', 'POST', 'PUT', 'DELETE', 'PATCH'];

      methodOrder.forEach(method => {
        if (groupedRoutes[method]) {
          groupedRoutes[method].sort((a, b) => a.path.localeCompare(b.path));

          groupedRoutes[method].forEach(route => {
            markdown += this.generateEndpointMarkdown(route);
          });
        }
      });

      markdown += '\n---\n\n';
    });

    // Add statistics section
    markdown += this.generateStatisticsSection();

    // Add examples section
    markdown += this.generateExamplesSection();

    return markdown;
  }

  /**
   * Get emoji for category
   */
  getCategoryEmoji(category) {
    const emojis = {
      'Authentication': '🔐',
      'Event Management': '📅',
      'Registration System': '📝',
      'Social Features': '👥',
      'Sales Management': '💰',
      'Business Intelligence': '📊',
      'System Monitoring': '🏥',
      'Administration': '⚙️',
      'Deployment': '🚀',
      'Emergency Operations': '🚨',
      'Media Management': '📸',
      'Development': '🔧',
      'General': '📡'
    };
    return emojis[category] || '📡';
  }

  /**
   * Generate markdown for a single endpoint
   */
  generateEndpointMarkdown(route) {
    let markdown = `### **${route.method}** ${route.fullPath}\n`;

    if (route.accessLevel !== 'Public') {
      markdown += `**Access**: ${route.accessLevel}\n`;
    } else {
      markdown += `**Access**: Public\n`;
    }

    if (route.description) {
      markdown += `**Description**: ${route.description}\n`;
    }

    // Add example request/response if available
    if (route.method === 'GET') {
      markdown += `**Example Request**:\n\`\`\`bash\ncurl -X GET ${route.fullPath} \\\
  -H "Authorization: Bearer <token>" \\\
  -H "Content-Type: application/json"\n\`\`\n\n`;

      markdown += `**Example Response**:\n\`\`\`json\n{\n  "success": true,\n  "data": {}\n}\n\`\`\n\n`;
    } else if (route.method === 'POST') {
      markdown += `**Example Request**:\n\`\`\`bash\ncurl -X POST ${route.fullPath} \\\
  -H "Authorization: Bearer <token>" \\\
  -H "Content-Type: application/json" \\\
  -d '{"key": "value"}'\n\`\`\n\n`;

      markdown += `**Example Response**:\n\`\`\`json\n{\n  "success": true,\n  "message": "Operation successful",\n  "data": {}\n}\n\`\`\n\n`;
    }

    markdown += '\n';
    return markdown;
  }

  /**
   * Generate statistics section
   */
  generateStatisticsSection() {
    let stats = `## 📊 **API Statistics**

### **Endpoint Summary**
- **Total Endpoints**: ${this.stats.totalEndpoints}
- **Route Files**: ${Object.keys(this.stats.files).length}

### **Methods Distribution**
`;

    Object.keys(this.stats.methods).sort().forEach(method => {
      stats += `- **${method}**: ${this.stats.methods[method]} endpoints\n`;
    });

    stats += `\n### **Files Distribution**
`;

    Object.keys(this.stats.files).sort().forEach(file => {
      stats += `- **${file}**: ${this.stats.files[file]} endpoints\n`;
    });

    stats += `\n### **Categories Distribution**
`;

    Object.keys(this.categories).sort().forEach(category => {
      stats += `- **${category}**: ${this.categories[category].length} endpoints\n`;
    });

    return stats;
  }

  /**
   * Generate examples section
   */
  generateExamplesSection() {
    return `## 📝 **Usage Examples**

### **Authentication Example**
\`\`\`bash
# Login and get token
curl -X POST http://localhost:5000/api/auth/login \\
  -H "Content-Type: application/json" \\
  -d '{"email": "user@example.com", "password": "password"}'

# Use token for authenticated requests
curl -X GET http://localhost:5000/api/events \\
  -H "Authorization: Bearer <your-jwt-token>"
\`\`\`

### **Error Handling**
All API endpoints return consistent error responses:

\`\`\`json
{
  "success": false,
  "error": "Error description",
  "details": {}
}
\`\`\`

### **HTTP Status Codes**
- \`200\` - Success
- \`201\` - Created
- \`400\` - Bad Request
- \`401\` - Unauthorized
- \`403\` - Forbidden
- \`404\` - Not Found
- \`500\` - Internal Server Error

---

**Generated**: ${new Date().toISOString()}
**Auto-Update**: This documentation is generated automatically from route files`;
  }

  /**
   * Save documentation to file
   */
  saveDocumentation() {
    const markdown = this.generateMarkdownDocs();
    const outputPath = join(PROJECT_ROOT, 'docs', 'api', 'API_AUTOGENERATED.md');

    writeFileSync(outputPath, markdown, 'utf8');
    this.log(`API documentation saved to: ${outputPath}`);

    // Also save JSON version for programmatic access
    const jsonData = {
      generated: new Date().toISOString(),
      stats: this.stats,
      categories: Object.keys(this.categories).reduce((acc, cat) => {
        acc[cat] = this.categories[cat].map(route => ({
          method: route.method,
          path: route.path,
          fullPath: route.fullPath,
          description: route.description,
          accessLevel: route.accessLevel,
          file: route.file
        }));
        return acc;
      }, {}),
      routes: this.routes
    };

    const jsonPath = join(PROJECT_ROOT, 'docs', 'api', 'api-data.json');
    writeFileSync(jsonPath, JSON.stringify(jsonData, null, 2), 'utf8');
    this.log(`API data saved to: ${jsonPath}`);

    return { markdownPath: outputPath, jsonPath };
  }

  /**
   * Log message
   */
  log(message, type = 'info') {
    const timestamp = new Date().toISOString();
    switch (type) {
      case 'error':
        console.error(`❌ ${message}`);
        break;
      case 'warning':
        console.warn(`⚠️  ${message}`);
        break;
      case 'success':
        console.log(`✅ ${message}`);
        break;
      default:
        console.log(`ℹ️  ${message}`);
    }
  }

  /**
   * Run the generation process
   */
  async run() {
    console.log('🔧 Generating API Documentation...\n');

    this.loadRoutes();
    this.categorizeRoutes();

    const { markdownPath, jsonPath } = this.saveDocumentation();

    console.log('\n📊 Generation Summary:');
    console.log(`  Total Endpoints: ${this.stats.totalEndpoints}`);
    console.log(`  Categories: ${Object.keys(this.categories).length}`);
    console.log(`  Route Files: ${Object.keys(this.stats.files).length}`);
    console.log(`\n📄 Files Generated:`);
    console.log(`  ${markdownPath}`);
    console.log(`  ${jsonPath}`);
    console.log('\n✅ API documentation generated successfully!');
  }
}

// Run generator if this script is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  const generator = new ApiDocumentationGenerator();
  generator.run().catch(error => {
    console.error('Generation failed:', error);
    process.exit(1);
  });
}

export default ApiDocumentationGenerator;