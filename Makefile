
# Test database connection and health check
database_health_check:
	@echo "Running database health check..."
	@cargo test -p lib-core test_database_connection -- --ignored