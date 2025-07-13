# Dubai Healthcare Emergency Response System

A production-ready Rust web application for coordinating emergency medical services across Dubai's healthcare network.

## Features

- 🏥 **Patient Management** - Real-time patient tracking and triage
- 👨‍⚕️ **Staff Coordination** - Medical staff availability and assignments  
- 🚑 **Ambulance Dispatch** - GPS tracking and route optimization
- 🔐 **Secure Authentication** - JWT-based auth with role-based access control
- 📊 **Hospital Analytics** - Real-time capacity and performance metrics

## Quick Start

### Prerequisites

- Rust 1.70+ 
- Docker & Docker Compose
- PostgreSQL (via Docker)
- Redis (via Docker)

### Setup

1. **Clone and setup the project:**
   ```bash
   git clone <your-repo>
   cd dubai-healthcare-emergency
   cp .env.example .env
