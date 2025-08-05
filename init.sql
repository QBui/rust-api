-- Initialize database with required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_stat_statements";

-- Create application user with limited privileges
CREATE USER IF NOT EXISTS api_user WITH PASSWORD 'secure_password';
GRANT CONNECT ON DATABASE scalable_api TO api_user;
GRANT USAGE ON SCHEMA public TO api_user;
GRANT CREATE ON SCHEMA public TO api_user;
