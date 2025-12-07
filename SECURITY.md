# Security Documentation

## Overview
This document outlines the security measures implemented in the Last.fm Dating App to protect user data and prevent unauthorized access.

## Authentication & Authorization

### JWT Bearer Token Authentication
- **Implementation**: All protected endpoints require a valid JWT Bearer token in the `Authorization` header
- **Format**: `Authorization: Bearer <token>`
- **Token Lifetime**: 30 days (configurable)
- **Token Validation**: Tokens are verified on every protected route request
- **Secret Key**: Stored in environment variable `JWT_SECRET` (must be strong in production)

### Password Security
- **Hashing Algorithm**: Argon2 (industry standard, resistant to brute-force attacks)
- **Password Storage**: Only hashed passwords are stored in the database
- **Password Transmission**: Passwords are never exposed in API responses
- **Serialization Protection**: `#[serde(skip_serializing)]` prevents password hash exposure

## Route Protection

### Public Routes (No Authentication Required)
- `POST /auth/register` - User registration
- `POST /auth/login` - User login
- `POST /auth/logout` - User logout

### Protected Routes (Authentication Required)
All routes below require valid Bearer token:
- `GET /users/me` - Get current user profile
- `PUT /users/me` - Update current user profile
- `GET /users/:id` - Get user profile by ID
- `POST /likes` - Create a like
- `GET /matches` - Get user matches
- `DELETE /matches/:id` - Delete a match
- `POST /photos` - Upload a photo
- `GET /photos/:user_id` - Get user photos
- `DELETE /photos/:id` - Delete a photo
- `POST /lastfm/connect` - Connect Last.fm account
- `POST /lastfm/sync` - Sync Last.fm scrobbles
- `GET /discover` - Get discover profiles

## SQL Injection Prevention

### Parameterized Queries
All database queries use SQLx parameterized queries to prevent SQL injection:
```rust
sqlx::query("SELECT * FROM users WHERE id = ?")
    .bind(&user_id)
    .fetch_optional(pool)
```

### Safe Update Operations
User update operations use individual parameterized queries instead of dynamic SQL building to prevent injection attacks.

## CORS (Cross-Origin Resource Sharing)

### Restricted Origins
- **Default**: `http://localhost:3000` for development
- **Production**: Configure via `ALLOWED_ORIGINS` environment variable
- **Multiple Origins**: Comma-separated list supported
- **Credentials**: Enabled for authenticated requests
- **Methods**: GET, POST, PUT, DELETE only

### Configuration
```env
ALLOWED_ORIGINS=https://yourdomain.com,https://www.yourdomain.com
```

## Data Protection

### Sensitive Data Handling
1. **Password Hashes**: Never exposed in API responses
2. **JWT Secrets**: Stored in environment variables, never in code
3. **API Keys**: Stored in environment variables
4. **Database Credentials**: Stored in environment variables

### User Data Access Control
- Users can only update their own profile
- Users can only delete their own photos
- Users can only access their own matches and likes

## Environment Variables Security

### Required Secure Configuration
```env
# CRITICAL: Change these in production
JWT_SECRET=<strong-random-secret-minimum-32-characters>
DATABASE_URL=mysql://<user>:<password>@<host>:<port>/<database>
LASTFM_API_KEY=<your-lastfm-api-key>
LASTFM_API_SECRET=<your-lastfm-api-secret>

# Optional but recommended
S3_ACCESS_KEY=<your-s3-access-key>
S3_SECRET_KEY=<your-s3-secret-key>

# CORS configuration
ALLOWED_ORIGINS=https://yourdomain.com
```

## Best Practices Implemented

1. ✅ **JWT Bearer Token Authentication**: Industry-standard authentication
2. ✅ **Argon2 Password Hashing**: Secure password storage
3. ✅ **Parameterized SQL Queries**: SQL injection prevention
4. ✅ **Route-level Authorization**: Proper separation of public and protected routes
5. ✅ **CORS Restrictions**: Only allowed origins can access the API
6. ✅ **Password Hash Protection**: Never exposed in responses
7. ✅ **Environment Variable Secrets**: No hardcoded credentials
8. ✅ **Error Message Security**: Generic error messages to prevent information leakage

## Security Checklist for Production

- [ ] Set strong `JWT_SECRET` (minimum 32 random characters)
- [ ] Configure `ALLOWED_ORIGINS` with production domain(s)
- [ ] Use HTTPS only (configure reverse proxy/load balancer)
- [ ] Set up rate limiting (recommend using a reverse proxy like nginx)
- [ ] Enable database connection encryption
- [ ] Set up monitoring and logging for security events
- [ ] Regular security audits and dependency updates
- [ ] Implement additional security headers (via reverse proxy):
  - `X-Content-Type-Options: nosniff`
  - `X-Frame-Options: DENY`
  - `X-XSS-Protection: 1; mode=block`
  - `Strict-Transport-Security: max-age=31536000; includeSubDomains`
  - `Content-Security-Policy: default-src 'self'`

## Vulnerability Disclosure

If you discover a security vulnerability, please report it responsibly:
1. Do not publicly disclose the vulnerability
2. Email security details to the maintainers
3. Allow time for the issue to be fixed before disclosure

## Testing Security

### Manual Testing
1. Try accessing protected routes without Bearer token (should return 401)
2. Try accessing protected routes with invalid token (should return 401)
3. Try accessing other users' data (should return appropriate errors)
4. Verify CORS headers are present and correct

### Automated Testing
- Run `cargo clippy` for security lints
- Use tools like `cargo audit` to check for vulnerable dependencies
- Consider using OWASP ZAP or similar tools for penetration testing

## Updates and Maintenance

- Regularly update dependencies: `cargo update`
- Check for security advisories: `cargo audit`
- Review and rotate JWT secrets periodically
- Monitor logs for suspicious activity

---

**Last Updated**: 2025-12-06
**Version**: 1.0.0
