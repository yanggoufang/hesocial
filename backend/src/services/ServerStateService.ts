import { duckdb } from '../database/duckdb-connection.js';
import logger from '../utils/logger.js';

export interface ServerStats {
  startCount: number;
  firstStartTime: Date | null;
  lastStartTime: Date | null;
  lastStopTime: Date | null;
  lastSessionDuration: number; // in seconds
  totalLifetime: number; // in seconds
  currentUptime: number; // in seconds
  averageSessionDuration: number; // in seconds
}

export class ServerStateService {
  private db: any;
  private serverStartTime: Date;
  private isInitialized: boolean = false;

  constructor() {
    this.db = duckdb;
    this.serverStartTime = new Date();
  }

  /**
   * Initialize server state tracking on startup
   */
  async initialize(): Promise<void> {
    if (this.isInitialized) {
      return;
    }

    try {
      await this.ensureServerStateTable();
      await this.recordServerStart();
      this.isInitialized = true;
      logger.info('Server state tracking initialized');
    } catch (error) {
      logger.error('Failed to initialize server state tracking', { 
        error: error instanceof Error ? error.message : error 
      });
      throw error;
    }
  }

  /**
   * Ensure server_state table exists (it should from schema, but double-check)
   */
  private async ensureServerStateTable(): Promise<void> {
    const sql = `
      CREATE TABLE IF NOT EXISTS server_state (
        id INTEGER PRIMARY KEY,
        start_count INTEGER NOT NULL DEFAULT 0,
        first_start_time TIMESTAMP,
        last_start_time TIMESTAMP,
        last_stop_time TIMESTAMP,
        last_session_duration INTEGER DEFAULT 0,
        total_lifetime INTEGER DEFAULT 0
      );
    `;

    await this.db.query(sql);
  }

  /**
   * Record server startup
   */
  private async recordServerStart(): Promise<void> {
    try {
      // Get current server state
      const currentState = await this.getServerState();
      
      if (currentState) {
        // Update existing record
        const newStartCount = currentState.startCount + 1;
        const lastSessionDuration = currentState.lastStopTime 
          ? Math.floor((currentState.lastStopTime.getTime() - currentState.lastStartTime!.getTime()) / 1000)
          : 0;
        
        const sql = `
          UPDATE server_state 
          SET start_count = ?, 
              last_start_time = ?, 
              last_session_duration = ?,
              total_lifetime = total_lifetime + ?
          WHERE id = 1
        `;
        
        await this.runQuery(sql, [
          newStartCount,
          this.serverStartTime.toISOString(),
          lastSessionDuration,
          lastSessionDuration
        ]);

        logger.info('Server startup recorded', { 
          startCount: newStartCount,
          lastSessionDuration: `${lastSessionDuration}s`
        });
      } else {
        // Create initial record
        const sql = `
          INSERT INTO server_state (id, start_count, first_start_time, last_start_time)
          VALUES (1, 1, ?, ?)
        `;
        
        await this.runQuery(sql, [
          this.serverStartTime.toISOString(),
          this.serverStartTime.toISOString()
        ]);

        logger.info('Server state initialized', { 
          firstStart: this.serverStartTime.toISOString()
        });
      }
    } catch (error) {
      logger.error('Failed to record server start', { 
        error: error instanceof Error ? error.message : error 
      });
      throw error;
    }
  }

  /**
   * Record server shutdown
   */
  async recordServerStop(): Promise<void> {
    if (!this.isInitialized) {
      return;
    }

    try {
      const stopTime = new Date();
      const sessionDuration = Math.floor((stopTime.getTime() - this.serverStartTime.getTime()) / 1000);

      const sql = `
        UPDATE server_state 
        SET last_stop_time = ?, 
            last_session_duration = ?,
            total_lifetime = total_lifetime + ?
        WHERE id = 1
      `;

      await this.runQuery(sql, [
        stopTime.toISOString(),
        sessionDuration,
        sessionDuration
      ]);

      logger.info('Server shutdown recorded', { 
        sessionDuration: `${sessionDuration}s`,
        stopTime: stopTime.toISOString()
      });
    } catch (error) {
      logger.error('Failed to record server stop', { 
        error: error instanceof Error ? error.message : error 
      });
    }
  }

  /**
   * Get current server statistics
   */
  async getServerStats(): Promise<ServerStats> {
    try {
      const state = await this.getServerState();
      const currentUptime = Math.floor((Date.now() - this.serverStartTime.getTime()) / 1000);
      
      if (!state) {
        return {
          startCount: 0,
          firstStartTime: null,
          lastStartTime: null,
          lastStopTime: null,
          lastSessionDuration: 0,
          totalLifetime: 0,
          currentUptime,
          averageSessionDuration: 0
        };
      }

      const averageSessionDuration = state.startCount > 0 
        ? Math.floor(state.totalLifetime / state.startCount)
        : 0;

      return {
        startCount: state.startCount,
        firstStartTime: state.firstStartTime,
        lastStartTime: state.lastStartTime,
        lastStopTime: state.lastStopTime,
        lastSessionDuration: state.lastSessionDuration,
        totalLifetime: state.totalLifetime,
        currentUptime,
        averageSessionDuration
      };
    } catch (error) {
      logger.error('Failed to get server stats', { 
        error: error instanceof Error ? error.message : error 
      });
      throw error;
    }
  }

  /**
   * Get raw server state from database
   */
  private async getServerState(): Promise<{
    startCount: number;
    firstStartTime: Date | null;
    lastStartTime: Date | null;
    lastStopTime: Date | null;
    lastSessionDuration: number;
    totalLifetime: number;
  } | null> {
    const sql = `
      SELECT 
        start_count as startCount,
        first_start_time as firstStartTime,
        last_start_time as lastStartTime,
        last_stop_time as lastStopTime,
        last_session_duration as lastSessionDuration,
        total_lifetime as totalLifetime
      FROM server_state 
      WHERE id = 1
    `;

    const result = await this.db.query(sql);
    
    if (!result.rows || result.rows.length === 0) {
      return null;
    }
    
    const row = result.rows[0];
    return {
      startCount: row.startCount || 0,
      firstStartTime: row.firstStartTime ? new Date(row.firstStartTime) : null,
      lastStartTime: row.lastStartTime ? new Date(row.lastStartTime) : null,
      lastStopTime: row.lastStopTime ? new Date(row.lastStopTime) : null,
      lastSessionDuration: row.lastSessionDuration || 0,
      totalLifetime: row.totalLifetime || 0
    };
  }

  /**
   * Helper method to run database queries
   */
  private async runQuery(sql: string, params: any[] = []): Promise<void> {
    await this.db.query(sql, params);
  }

  /**
   * Get formatted server uptime info
   */
  getFormattedUptime(): string {
    const currentUptime = Math.floor((Date.now() - this.serverStartTime.getTime()) / 1000);
    return this.formatDuration(currentUptime);
  }

  /**
   * Format duration in seconds to human-readable format
   */
  private formatDuration(seconds: number): string {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;

    const parts = [];
    if (days > 0) parts.push(`${days}d`);
    if (hours > 0) parts.push(`${hours}h`);
    if (minutes > 0) parts.push(`${minutes}m`);
    if (secs > 0 || parts.length === 0) parts.push(`${secs}s`);

    return parts.join(' ');
  }

  /**
   * Log server statistics (useful for debugging and monitoring)
   */
  async logServerStats(): Promise<void> {
    try {
      const stats = await this.getServerStats();
      
      logger.info('='.repeat(50));
      logger.info('📊 SERVER STATISTICS');
      logger.info('='.repeat(50));
      logger.info(`Start Count: ${stats.startCount}`);
      logger.info(`First Start: ${stats.firstStartTime?.toISOString() || 'Never'}`);
      logger.info(`Last Start: ${stats.lastStartTime?.toISOString() || 'Never'}`);
      logger.info(`Last Stop: ${stats.lastStopTime?.toISOString() || 'Never'}`);
      logger.info(`Current Uptime: ${this.formatDuration(stats.currentUptime)}`);
      logger.info(`Last Session: ${this.formatDuration(stats.lastSessionDuration)}`);
      logger.info(`Average Session: ${this.formatDuration(stats.averageSessionDuration)}`);
      logger.info(`Total Lifetime: ${this.formatDuration(stats.totalLifetime)}`);
      logger.info('='.repeat(50));
    } catch (error) {
      logger.error('Failed to log server stats', { 
        error: error instanceof Error ? error.message : error 
      });
    }
  }
}