-- Dubai Healthcare Emergency Response System
-- Database initialization script

-- Create extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "postgis";

-- Set timezone to Gulf Standard Time
SET timezone = 'Asia/Dubai';

-- Create database user for application (if not exists)
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_user WHERE usename = 'healthcare_app') THEN
        CREATE USER healthcare_app WITH PASSWORD 'app_password';
    END IF;
END
$$;

-- Grant permissions
GRANT CONNECT ON DATABASE healthcare_emergency TO healthcare_app;
GRANT USAGE ON SCHEMA public TO healthcare_app;
GRANT CREATE ON SCHEMA public TO healthcare_app;

-- Create initial tables will be handled by migration service
