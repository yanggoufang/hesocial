#!/usr/bin/env node

/**
 * R2 Integration Test
 * Tests backup functionality with Cloudflare R2
 */

import { S3Client, PutObjectCommand, ListObjectsV2Command, GetObjectCommand } from '@aws-sdk/client-s3';
import { Upload } from '@aws-sdk/lib-storage';
import { createReadStream, createWriteStream, existsSync, writeFileSync, statSync, unlinkSync } from 'fs';
import { join } from 'path';
import dotenv from 'dotenv';

// Load environment variables
dotenv.config();

// R2 Configuration
const R2_CONFIG = {
  accountId: process.env.R2_ACCOUNT_ID,
  accessKeyId: process.env.R2_ACCESS_KEY_ID,
  secretAccessKey: process.env.R2_SECRET_ACCESS_KEY,
  bucketName: process.env.R2_BUCKET_NAME,
  endpoint: process.env.R2_ENDPOINT,
  region: process.env.R2_REGION || 'auto'
};

console.log('🧪 R2 Integration Test Starting...\n');

// Validate configuration
function validateConfig() {
  const required = ['accountId', 'accessKeyId', 'secretAccessKey', 'bucketName', 'endpoint'];
  const missing = required.filter(key => !R2_CONFIG[key]);
  
  if (missing.length > 0) {
    console.error('❌ Missing required R2 configuration:', missing);
    process.exit(1);
  }
  
  console.log('✅ R2 Configuration validated');
  console.log(`   Bucket: ${R2_CONFIG.bucketName}`);
  console.log(`   Endpoint: ${R2_CONFIG.endpoint}\n`);
}

// Initialize S3 Client for R2
function createR2Client() {
  return new S3Client({
    region: R2_CONFIG.region,
    endpoint: R2_CONFIG.endpoint,
    credentials: {
      accessKeyId: R2_CONFIG.accessKeyId,
      secretAccessKey: R2_CONFIG.secretAccessKey,
    },
  });
}

// Test R2 connection
async function testConnection(client) {
  console.log('🔗 Testing R2 connection...');
  
  try {
    const command = new ListObjectsV2Command({
      Bucket: R2_CONFIG.bucketName,
      MaxKeys: 1
    });
    
    await client.send(command);
    console.log('✅ R2 connection successful\n');
    return true;
  } catch (error) {
    console.error('❌ R2 connection failed:', error.message);
    return false;
  }
}

// Create dummy DuckDB file if not exists
function ensureDuckDBFile() {
  const dbPath = join(process.cwd(), 'hesocial.duckdb');
  
  if (existsSync(dbPath)) {
    console.log('✅ Found existing DuckDB file');
    const stats = statSync(dbPath);
    console.log(`   Size: ${(stats.size / 1024).toFixed(2)} KB`);
    console.log(`   Path: ${dbPath}\n`);
    return dbPath;
  }
  
  console.log('📝 Creating dummy DuckDB file...');
  
  // Create a dummy database file with some content
  const dummyContent = Buffer.alloc(1024 * 50); // 50KB dummy file
  dummyContent.write('DUMMY_DUCKDB_FILE_FOR_TESTING', 0);
  dummyContent.write(new Date().toISOString(), 50);
  dummyContent.write('HeSocial Social Event Platform', 100);
  
  writeFileSync(dbPath, dummyContent);
  
  console.log('✅ Dummy DuckDB file created');
  console.log(`   Size: 50 KB`);
  console.log(`   Path: ${dbPath}\n`);
  
  return dbPath;
}

// Upload backup to R2
async function uploadBackup(client, dbPath) {
  console.log('⬆️  Uploading backup to R2...');
  
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, -5);
  const backupFileName = `hesocial-test-${timestamp}.duckdb`;
  const r2Key = `backups/${backupFileName}`;
  
  try {
    const fileStream = createReadStream(dbPath);
    const stats = statSync(dbPath);
    
    console.log(`   File: ${backupFileName}`);
    console.log(`   Size: ${(stats.size / 1024).toFixed(2)} KB`);
    console.log(`   Key: ${r2Key}`);
    
    const upload = new Upload({
      client: client,
      params: {
        Bucket: R2_CONFIG.bucketName,
        Key: r2Key,
        Body: fileStream,
        ContentType: 'application/octet-stream',
        Metadata: {
          type: 'test',
          timestamp: new Date().toISOString(),
          size: stats.size.toString(),
          environment: 'test'
        }
      }
    });
    
    const result = await upload.done();
    
    console.log('✅ Backup uploaded successfully');
    console.log(`   ETag: ${result.ETag}`);
    console.log(`   Location: ${result.Location || 'R2 Bucket'}\n`);
    
    return backupFileName;
  } catch (error) {
    console.error('❌ Upload failed:', error.message);
    throw error;
  }
}

// List backups in R2
async function listBackups(client) {
  console.log('📋 Listing backups in R2...');
  
  try {
    const command = new ListObjectsV2Command({
      Bucket: R2_CONFIG.bucketName,
      Prefix: 'backups/',
      MaxKeys: 10
    });
    
    const response = await client.send(command);
    
    if (!response.Contents || response.Contents.length === 0) {
      console.log('   No backups found\n');
      return [];
    }
    
    console.log(`   Found ${response.Contents.length} backup(s):`);
    
    const backups = response.Contents
      .filter(obj => obj.Key?.endsWith('.duckdb'))
      .map(obj => ({
        key: obj.Key,
        size: obj.Size ? `${(obj.Size / 1024).toFixed(2)} KB` : 'Unknown',
        lastModified: obj.LastModified?.toISOString() || 'Unknown'
      }));
    
    backups.forEach((backup, index) => {
      console.log(`   ${index + 1}. ${backup.key}`);
      console.log(`      Size: ${backup.size}`);
      console.log(`      Modified: ${backup.lastModified}`);
    });
    
    console.log('');
    return backups;
  } catch (error) {
    console.error('❌ Failed to list backups:', error.message);
    throw error;
  }
}

// Test download backup
async function testDownload(client, backupFileName) {
  console.log('⬇️  Testing backup download...');
  
  const r2Key = `backups/${backupFileName}`;
  const downloadPath = join(process.cwd(), `downloaded-${backupFileName}`);
  
  try {
    const command = new GetObjectCommand({
      Bucket: R2_CONFIG.bucketName,
      Key: r2Key
    });
    
    const response = await client.send(command);
    
    if (!response.Body) {
      throw new Error('No data received from R2');
    }
    
    // Write to local file
    const writeStream = createWriteStream(downloadPath);
    
    await new Promise((resolve, reject) => {
      if (response.Body instanceof ReadableStream) {
        const nodeStream = response.Body;
        nodeStream.pipe(writeStream);
        nodeStream.on('end', resolve);
        nodeStream.on('error', reject);
      } else {
        reject(new Error('Unexpected response body type'));
      }
    });
    
    const stats = statSync(downloadPath);
    
    console.log('✅ Download successful');
    console.log(`   Downloaded: ${downloadPath}`);
    console.log(`   Size: ${(stats.size / 1024).toFixed(2)} KB\n`);
    
    // Clean up downloaded file
    unlinkSync(downloadPath);
    console.log('🧹 Cleaned up downloaded test file\n');
    
    return true;
  } catch (error) {
    console.error('❌ Download failed:', error.message);
    throw error;
  }
}

// Main test function
async function runIntegrationTest() {
  let client;
  
  try {
    // Step 1: Validate configuration
    validateConfig();
    
    // Step 2: Create R2 client
    client = createR2Client();
    
    // Step 3: Test connection
    const connected = await testConnection(client);
    if (!connected) {
      throw new Error('Failed to connect to R2');
    }
    
    // Step 4: Ensure DuckDB file exists
    const dbPath = ensureDuckDBFile();
    
    // Step 5: Upload backup
    const backupFileName = await uploadBackup(client, dbPath);
    
    // Step 6: List backups
    await listBackups(client);
    
    // Step 7: Test download
    await testDownload(client, backupFileName);
    
    console.log('🎉 R2 Integration Test PASSED!');
    console.log('✅ All backup operations working correctly');
    console.log('🚀 Ready for production use\n');
    
  } catch (error) {
    console.error('\n❌ R2 Integration Test FAILED!');
    console.error('Error:', error.message);
    
    if (error.code) {
      console.error('Error Code:', error.code);
    }
    
    console.error('\n🔧 Check your R2 configuration and try again\n');
    process.exit(1);
  }
}

// Run the test
if (import.meta.url === `file://${process.argv[1]}`) {
  runIntegrationTest();
}