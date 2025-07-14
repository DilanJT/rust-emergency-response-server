use lib_core::config::{DatabaseConfig, DatabaseHealth};
use std::env;

#[tokio::test]
#[ignore] // Ignore by default since it requires a running database
async fn test_database_connection() {
    // Only run if DATABASE_URL is set
    if env::var("DATABASE_URL").is_err() {
        println!("Skipping database test - DATABASE_URL not set");
        return;
    }

    let config = DatabaseConfig::from_env().expect("Failed to load database config");
    
    // Test connection
    config.test_connection().await.expect("Database connection test failed");
    
    // Test pool creation
    let pool = config.create_pool().await.expect("Failed to create connection pool");
    
    // Test health check
    let health = DatabaseHealth::check(&pool, &config).await;
    assert!(health.is_healthy() || health.is_available());
    
    println!("Database integration test passed!");
    println!("Health status: {:?}", health.status);
    println!("Response time: {}ms", health.response_time_ms);
    println!("Active connections: {}", health.active_connections);
}