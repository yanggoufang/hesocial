import { BaseMigration } from './Migration.js';

export default class AddSalesManagementSystem extends BaseMigration {
  id = '003_add_sales_management_system';
  version = 3;
  name = 'Add Sales Management System';
  description = 'Add comprehensive CRM and sales pipeline tables for luxury membership sales';
  category = 'schema' as const;

  async up(): Promise<void> {
    // Sales leads table - tracks prospects and their journey
    await this.executeSQL(`
      CREATE TABLE IF NOT EXISTS sales_leads (
        id INTEGER PRIMARY KEY,
        first_name VARCHAR(100) NOT NULL,
        last_name VARCHAR(100) NOT NULL,
        email VARCHAR(255) UNIQUE NOT NULL,
        phone VARCHAR(20),
        company VARCHAR(200),
        job_title VARCHAR(200),
        annual_income BIGINT,
        net_worth BIGINT,
        source VARCHAR(50) NOT NULL, -- 'website', 'referral', 'event', 'cold_call', 'linkedin', 'advertisement'
        referral_code VARCHAR(50),
        lead_score INTEGER DEFAULT 0 CHECK (lead_score >= 0 AND lead_score <= 100),
        status VARCHAR(20) DEFAULT 'new' CHECK (status IN ('new', 'qualified', 'contacted', 'nurturing', 'proposal', 'negotiation', 'closed_won', 'closed_lost')),
        interested_membership_tier VARCHAR(20) CHECK (interested_membership_tier IN ('Platinum', 'Diamond', 'Black Card')),
        budget_range VARCHAR(50),
        timeline VARCHAR(50),
        pain_points TEXT,
        interests VARCHAR[],
        notes TEXT,
        last_contact_date TIMESTAMP,
        next_follow_up_date TIMESTAMP,
        assigned_to INTEGER, -- references users.id (sales rep)
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
      );
    `);

    // Sales opportunities table - formal sales processes
    await this.executeSQL(`
      CREATE TABLE IF NOT EXISTS sales_opportunities (
        id INTEGER PRIMARY KEY,
        lead_id INTEGER NOT NULL,
        name VARCHAR(255) NOT NULL, -- opportunity name
        description TEXT,
        stage VARCHAR(30) NOT NULL DEFAULT 'qualification' CHECK (stage IN ('qualification', 'needs_analysis', 'proposal', 'negotiation', 'closed_won', 'closed_lost')),
        probability INTEGER DEFAULT 25 CHECK (probability >= 0 AND probability <= 100),
        value DECIMAL(12,2) NOT NULL, -- opportunity value in TWD
        expected_close_date DATE,
        actual_close_date DATE,
        membership_tier VARCHAR(20) NOT NULL CHECK (membership_tier IN ('Platinum', 'Diamond', 'Black Card')),
        payment_terms VARCHAR(50),
        close_reason TEXT,
        assigned_to INTEGER NOT NULL, -- references users.id (sales rep)
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
      );
    `);

    // Sales activities table - track all interactions
    await this.executeSQL(`
      CREATE TABLE IF NOT EXISTS sales_activities (
        id INTEGER PRIMARY KEY,
        lead_id INTEGER,
        opportunity_id INTEGER,
        activity_type VARCHAR(30) NOT NULL CHECK (activity_type IN ('call', 'email', 'meeting', 'demo', 'proposal', 'follow_up', 'note')),
        subject VARCHAR(255) NOT NULL,
        description TEXT,
        outcome VARCHAR(50),
        duration_minutes INTEGER,
        scheduled_at TIMESTAMP,
        completed_at TIMESTAMP,
        created_by INTEGER NOT NULL, -- references users.id
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
      );
    `);

    // Sales pipeline stages configuration
    await this.executeSQL(`
      CREATE TABLE IF NOT EXISTS sales_pipeline_stages (
        id INTEGER PRIMARY KEY,
        name VARCHAR(50) NOT NULL UNIQUE,
        display_order INTEGER NOT NULL,
        default_probability INTEGER DEFAULT 0 CHECK (default_probability >= 0 AND default_probability <= 100),
        is_active BOOLEAN DEFAULT TRUE,
        color_code VARCHAR(7), -- hex color code
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
      );
    `);

    // Sales targets and quotas
    await this.executeSQL(`
      CREATE TABLE IF NOT EXISTS sales_targets (
        id INTEGER PRIMARY KEY,
        sales_rep_id INTEGER NOT NULL, -- references users.id
        period_type VARCHAR(20) NOT NULL CHECK (period_type IN ('monthly', 'quarterly', 'yearly')),
        period_start DATE NOT NULL,
        period_end DATE NOT NULL,
        target_revenue DECIMAL(12,2) NOT NULL,
        target_leads INTEGER,
        target_conversions INTEGER,
        actual_revenue DECIMAL(12,2) DEFAULT 0,
        actual_leads INTEGER DEFAULT 0,
        actual_conversions INTEGER DEFAULT 0,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
      );
    `);

    // Sales team structure
    await this.executeSQL(`
      CREATE TABLE IF NOT EXISTS sales_team_members (
        id INTEGER PRIMARY KEY,
        user_id INTEGER NOT NULL, -- references users.id
        role VARCHAR(30) NOT NULL CHECK (role IN ('sales_rep', 'senior_sales_rep', 'sales_manager', 'sales_director')),
        territory VARCHAR(100),
        commission_rate DECIMAL(5,2) DEFAULT 0.00,
        is_active BOOLEAN DEFAULT TRUE,
        hire_date DATE,
        manager_id INTEGER, -- references users.id
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
      );
    `);

    // Commission tracking
    await this.executeSQL(`
      CREATE TABLE IF NOT EXISTS sales_commissions (
        id INTEGER PRIMARY KEY,
        sales_rep_id INTEGER NOT NULL, -- references users.id
        opportunity_id INTEGER NOT NULL,
        commission_type VARCHAR(20) NOT NULL CHECK (commission_type IN ('new_member', 'renewal', 'upsell', 'referral')),
        base_amount DECIMAL(12,2) NOT NULL,
        commission_rate DECIMAL(5,2) NOT NULL,
        commission_amount DECIMAL(12,2) NOT NULL,
        payment_status VARCHAR(20) DEFAULT 'pending' CHECK (payment_status IN ('pending', 'approved', 'paid', 'disputed')),
        period_month INTEGER NOT NULL,
        period_year INTEGER NOT NULL,
        paid_at TIMESTAMP,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
      );
    `);

    // Add indexes for performance
    await this.executeSQL(`
      CREATE INDEX IF NOT EXISTS idx_sales_leads_email ON sales_leads(email);
      CREATE INDEX IF NOT EXISTS idx_sales_leads_assigned_to ON sales_leads(assigned_to);
      CREATE INDEX IF NOT EXISTS idx_sales_leads_status ON sales_leads(status);
      CREATE INDEX IF NOT EXISTS idx_sales_opportunities_lead_id ON sales_opportunities(lead_id);
      CREATE INDEX IF NOT EXISTS idx_sales_opportunities_assigned_to ON sales_opportunities(assigned_to);
      CREATE INDEX IF NOT EXISTS idx_sales_opportunities_stage ON sales_opportunities(stage);
      CREATE INDEX IF NOT EXISTS idx_sales_activities_lead_id ON sales_activities(lead_id);
      CREATE INDEX IF NOT EXISTS idx_sales_activities_opportunity_id ON sales_activities(opportunity_id);
      CREATE INDEX IF NOT EXISTS idx_sales_activities_created_by ON sales_activities(created_by);
      CREATE INDEX IF NOT EXISTS idx_sales_targets_sales_rep_id ON sales_targets(sales_rep_id);
      CREATE INDEX IF NOT EXISTS idx_sales_team_members_user_id ON sales_team_members(user_id);
      CREATE INDEX IF NOT EXISTS idx_sales_commissions_sales_rep_id ON sales_commissions(sales_rep_id);
    `);

    // Insert default pipeline stages
    await this.executeSQL(`
      INSERT INTO sales_pipeline_stages (name, display_order, default_probability, color_code) VALUES
      ('Qualification', 1, 10, '#FF6B6B'),
      ('Needs Analysis', 2, 25, '#4ECDC4'),
      ('Proposal', 3, 50, '#45B7D1'),
      ('Negotiation', 4, 75, '#F7B731'),
      ('Closed Won', 5, 100, '#26DE81'),
      ('Closed Lost', 6, 0, '#95A5A6')
      ON CONFLICT (name) DO NOTHING;
    `);
  }

  async down(): Promise<void> {
    // Drop all sales management tables in reverse order
    await this.executeSQL(`DROP TABLE IF EXISTS sales_commissions;`);
    await this.executeSQL(`DROP TABLE IF EXISTS sales_team_members;`);
    await this.executeSQL(`DROP TABLE IF EXISTS sales_targets;`);
    await this.executeSQL(`DROP TABLE IF EXISTS sales_pipeline_stages;`);
    await this.executeSQL(`DROP TABLE IF EXISTS sales_activities;`);
    await this.executeSQL(`DROP TABLE IF EXISTS sales_opportunities;`);
    await this.executeSQL(`DROP TABLE IF EXISTS sales_leads;`);
  }
}