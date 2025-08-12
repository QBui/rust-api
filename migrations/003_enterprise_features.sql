-- Create audit logs table for enterprise audit trail
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id UUID,
    ip_address INET NOT NULL,
    user_agent TEXT,
    details JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);

-- Create feature flags table for A/B testing and feature rollouts
CREATE TABLE IF NOT EXISTS feature_flags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    enabled BOOLEAN NOT NULL DEFAULT false,
    rollout_percentage REAL NOT NULL DEFAULT 0.0 CHECK (rollout_percentage >= 0.0 AND rollout_percentage <= 100.0),
    conditions JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create performance metrics table for detailed analytics
CREATE TABLE IF NOT EXISTS performance_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    endpoint VARCHAR(255) NOT NULL,
    method VARCHAR(10) NOT NULL,
    duration_ms REAL NOT NULL,
    status_code INTEGER NOT NULL,
    memory_usage_mb REAL,
    db_query_time_ms REAL,
    cache_hit BOOLEAN DEFAULT false,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for audit logs
CREATE INDEX IF NOT EXISTS idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_resource ON audit_logs(resource_type, resource_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_action ON audit_logs(action);
CREATE INDEX IF NOT EXISTS idx_audit_logs_created_at ON audit_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_audit_logs_ip_address ON audit_logs(ip_address);

-- Create indexes for feature flags
CREATE INDEX IF NOT EXISTS idx_feature_flags_name ON feature_flags(name);
CREATE INDEX IF NOT EXISTS idx_feature_flags_enabled ON feature_flags(enabled);

-- Create indexes for performance metrics
CREATE INDEX IF NOT EXISTS idx_performance_metrics_endpoint ON performance_metrics(endpoint);
CREATE INDEX IF NOT EXISTS idx_performance_metrics_created_at ON performance_metrics(created_at);
CREATE INDEX IF NOT EXISTS idx_performance_metrics_duration ON performance_metrics(duration_ms);
CREATE INDEX IF NOT EXISTS idx_performance_metrics_user_id ON performance_metrics(user_id);

-- Add updated_at trigger for feature flags
CREATE OR REPLACE TRIGGER update_feature_flags_updated_at BEFORE UPDATE ON feature_flags
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Create partitioning for audit logs (monthly partitions for performance)
-- This helps with large-scale audit log storage
CREATE TABLE IF NOT EXISTS audit_logs_template (LIKE audit_logs INCLUDING ALL);

-- Create a function to automatically create partitions
CREATE OR REPLACE FUNCTION create_monthly_partition(table_name text, start_date TIMESTAMPTZ)
RETURNS void AS $$
DECLARE
    partition_name text;
    end_date date;
BEGIN
    partition_name := table_name || '_' || to_char(start_date, 'YYYY_MM');
    end_date := start_date + interval '1 month';

    EXECUTE format('CREATE TABLE IF NOT EXISTS %I PARTITION OF %I
                    FOR VALUES FROM (%L) TO (%L)',
                   partition_name, table_name, start_date, end_date);
END;
$$ LANGUAGE plpgsql;

-- Create initial partition for current month
SELECT create_monthly_partition('audit_logs', date_trunc('month', CURRENT_DATE));

-- Insert default feature flags
INSERT INTO feature_flags (name, enabled, rollout_percentage, conditions) VALUES
    ('user_registration', true, 100.0, NULL),
    ('beta_features', false, 10.0, '{"user_tier": ["premium", "enterprise"]}'),
    ('advanced_analytics', true, 50.0, NULL),
    ('rate_limit_bypass', false, 0.0, '{"role": ["admin", "system"]}'),
    ('new_dashboard', false, 25.0, NULL) ON CONFLICT (name) DO NOTHING;
