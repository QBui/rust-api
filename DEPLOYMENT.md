# Production Deployment Guide

## 🚀 Quick Start

### 1. Environment Setup
```bash
# Clone and navigate to the project
cd rust-api

# Copy environment configuration
cp .env.example .env
cp .env.production .env.prod

# Edit production variables
vim .env.prod  # Set secure passwords and secrets
```

### 2. Development Environment
```bash
# Start development stack
make setup
make dev

# Or manually:
docker-compose up -d postgres redis
cargo run --bin api
```

### 3. Production Deployment
```bash
# Build production image
make build

# Deploy with production configuration
make prod

# Or manually:
docker build -t scalable-rust-api:latest .
docker-compose -f docker-compose.prod.yml up -d
```

## 🔧 Configuration

### Required Environment Variables
```bash
# Database
DATABASE_URL=postgres://user:pass@host:5432/db
DB_PASSWORD=secure_database_password

# Authentication
JWT_SECRET=your-super-secure-jwt-secret-key

# Admin credentials
GRAFANA_PASSWORD=secure_grafana_password
```

### SSL Certificate Setup
```bash
# Generate self-signed certificates for development
mkdir -p ssl
openssl req -x509 -newkey rsa:4096 -nodes -keyout ssl/key.pem -out ssl/cert.pem -days 365

# For production, use Let's Encrypt or your certificate provider
```

## 📊 Monitoring URLs

Once deployed, access these endpoints:

- **API**: `https://your-domain/api/v1/`
- **Health Check**: `https://your-domain/health`
- **Grafana**: `http://your-domain:3000` (admin/your-grafana-password)
- **Prometheus**: `http://your-domain:9091`

## 🔒 Security Checklist

- [ ] Change all default passwords
- [ ] Set strong JWT secret (32+ characters)
- [ ] Configure SSL certificates
- [ ] Review rate limiting settings
- [ ] Set up firewall rules
- [ ] Enable audit logging
- [ ] Configure backup strategies

## 📈 Scaling Considerations

### Horizontal Scaling
```yaml
# In docker-compose.prod.yml
services:
  api:
    deploy:
      replicas: 3
      resources:
        limits:
          memory: 512M
          cpus: '0.5'
```

### Database Optimization
- Read replicas for scaling reads
- Connection pooling (configured: 50 max connections)
- Query optimization and indexing
- Regular VACUUM and ANALYZE

### Caching Strategy
- Redis for session storage
- Application-level caching for frequent queries
- CDN for static assets

## 🚨 Troubleshooting

### Common Issues
1. **Database Connection Failed**
   ```bash
   # Check database status
   docker-compose logs postgres
   # Verify connection string
   psql $DATABASE_URL
   ```

2. **High Memory Usage**
   ```bash
   # Monitor container resources
   docker stats
   # Adjust memory limits in docker-compose.prod.yml
   ```

3. **Authentication Errors**
   ```bash
   # Verify JWT secret consistency
   echo $JWT_SECRET
   # Check token expiration settings
   ```

## 📋 Maintenance Tasks

### Daily
- Monitor application logs
- Check system resource usage
- Verify backup completion

### Weekly
- Review security logs
- Update dependencies
- Performance monitoring review

### Monthly
- Security patch updates
- Database maintenance (VACUUM, ANALYZE)
- Capacity planning review
