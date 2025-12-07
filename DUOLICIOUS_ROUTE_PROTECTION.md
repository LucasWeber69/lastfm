# Duolicious Route Protection Analysis

## Overview

Comprehensive analysis of Duolicious backend route protection patterns and how they've been adapted for our Rust/Axum implementation.

## Duolicious Route Protection Patterns

### 1. Multi-Layer Security Architecture

**Decorators Used:**
```python
# Public routes (no auth)
@post('/request-otp')
@get('/health')

# Authenticated routes with status checks
@apost('/check-otp', expected_onboarding_status=None, expected_sign_in_status=False)
@aget('/me')  # Requires onboarded=True, signed_in=True (defaults)
@apatch('/onboardee-info', expected_onboarding_status=False)  # Only for non-onboarded users
```

### 2. Session-Based Authentication

**Key Features:**
- Database-backed sessions (not just JWT)
- Session tokens stored as SHA512 hashes
- Sessions have expiration timestamps
- Session validation query on every authenticated request:

```sql
SELECT
    duo_session.person_id,
    person.uuid::TEXT AS person_uuid,
    duo_session.email,
    duo_session.signed_in,
    duo_session.pending_club_name
FROM
    duo_session
LEFT JOIN
    person
ON
    duo_session.person_id = person.id
WHERE
    session_token_hash = %(session_token_hash)s
AND
    session_expiry > NOW()
```

### 3. Multi-Level Rate Limiting

**IP-Based Limits:**
- Default: `60 per minute; 12 per second`
- OTP endpoints: `3 per minute` (shared across all OTP routes)
- Specific operations: `40 per day` for OTP request/check
- Exempted for private IPs (development/testing)

**Account-Based Limits:**
- Uses normalized email as key (`limiter_account()`)
- Same limits applied per account
- Prevents abuse from single user across multiple IPs

**Dual Enforcement:**
```python
with (
    limiter.limit(limit, scope=scope, exempt_when=disable_ip_rate_limit),
    limiter.limit(limit, scope=scope, key_func=limiter_account, exempt_when=disable_account_rate_limit)
):
    return person.post_request_otp(req)
```

### 4. Authorization Header Validation

**Required Format:**
```
Authorization: Bearer <session_token>
```

**Validation Steps:**
1. Check header exists
2. Split into `bearer` + `token`
3. Verify `bearer` keyword (case-insensitive)
4. Hash token with SHA512
5. Query database for session
6. Check expiration
7. Verify onboarding/sign-in status
8. Apply account-based rate limiting

**Error Responses:**
- Missing/malformed header: `400 Bad Request`
- Invalid token: `401 Unauthorized`
- Wrong onboarding/sign-in status: `401 Unauthorized`

### 5. Onboarding State Management

**User States:**
- **Not onboarded, not signed in**: Just created account, verifying OTP
- **Not onboarded, signed in**: OTP verified, needs to complete profile
- **Onboarded, signed in**: Full access (normal user)

**Route Restrictions:**
```python
# Only for users who haven't completed onboarding
@apatch('/onboardee-info', expected_onboarding_status=False)

# Only for users who haven't signed in yet
@apost('/check-otp', expected_sign_in_status=False)

# For all authenticated users (defaults: onboarded=True, signed_in=True)
@aget('/me')

# Allow any onboarding status
@apost('/sign-out', expected_onboarding_status=None)
```

### 6. Request Validation with Pydantic

**Pattern:**
```python
@validate(t.PostRequestOtp)
def post_request_otp(req: t.PostRequestOtp):
    # req is already validated and typed
    return person.post_request_otp(req)
```

**Benefits:**
- Type safety
- Automatic validation errors (returns 400 with JSON error details)
- Input sanitization
- File upload support

### 7. IP Address Handling

**Features:**
- Proxy-aware (`ProxyFix` middleware for `X-Forwarded-For`)
- Private IP detection for rate limit exemption
- Mocking support for testing
- Default to `127.0.0.1` if no IP found

**Implementation:**
```python
def _get_remote_address() -> str:
    return mock_ip_address() or request.remote_addr or "127.0.0.1"
```

### 8. CORS Configuration

**Production Pattern:**
```python
CORS_ORIGINS = os.environ.get('DUO_CORS_ORIGINS', '*')
CORS(app, origins=CORS_ORIGINS.split(','))
```

**Benefits:**
- Environment-based configuration
- Multiple origins support (comma-separated)
- Credentials support (cookies/auth headers)

## Our Implementation (Rust/Axum)

### Current State

✅ **Implemented:**
1. JWT Bearer token authentication
2. Protected vs public route separation
3. Authorization header validation (`Bearer <token>`)
4. CORS with configurable origins
5. CAPTCHA on auth endpoints
6. Email normalization (duplicate prevention)

⚠️ **Missing (Compared to Duolicious):**
1. Database-backed session management
2. Onboarding state tracking
3. Account-based rate limiting (only have infrastructure)
4. IP address logging
5. Dual (IP + account) rate limit enforcement
6. Session expiration in database
7. Multiple user states (onboarded/signed-in)

### Enhanced Implementation

Below are the improvements implemented based on Duolicious patterns:

#### 1. Request Context Extension

**Added to our middleware:**
```rust
pub struct RequestContext {
    pub user_id: String,
    pub email: String,
    pub ip_address: String,
}
```

This allows routes to access:
- User identity
- Email (for account-based rate limiting)
- IP address (for logging and IP-based limits)

#### 2. IP Address Extraction

**ConnectInfo extractor:**
```rust
use axum::extract::ConnectInfo;
use std::net::SocketAddr;

async fn handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> String {
    format!("IP: {}", addr.ip())
}
```

#### 3. Rate Limiting Integration Points

**Structure prepared for:**
- IP-based limits (via middleware)
- Account-based limits (via user_id in context)
- Per-endpoint limits (decorator pattern in Axum)

#### 4. Session Management Schema

**Database table ready (migration 002):**
```sql
CREATE TABLE sessions (
    id CHAR(36) PRIMARY KEY,
    user_id CHAR(36) NOT NULL,
    session_token_hash VARCHAR(128) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP NOT NULL,
    last_used_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    ip_address VARCHAR(45),
    user_agent TEXT,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_token_hash (session_token_hash),
    INDEX idx_expires (expires_at)
);
```

## Key Takeaways

### Security Best Practices from Duolicious

1. **Defense in Depth**: Multiple layers (auth + rate limiting + validation + CORS)
2. **Database Session Validation**: Not just JWT verification
3. **Dual Rate Limiting**: Both IP and account to prevent abuse
4. **Fine-Grained Authorization**: Different routes for different user states
5. **IP Tracking**: Log all auth events for security auditing
6. **Token Hashing**: Never store raw tokens (use SHA512)
7. **Graceful Degradation**: Private IP exemption for development

### What Makes Our Implementation Unique

1. **Music-Based Matching**: Last.fm integration (Duolicious uses personality)
2. **CAPTCHA Protection**: Math challenges on auth (Duolicious uses OTP)
3. **Vector Similarity**: Cosine similarity for music preferences
4. **Rust Performance**: Axum is faster than Flask
5. **Type Safety**: Compile-time guarantees vs runtime validation

## Recommendations

### High Priority
1. ✅ **Implement IP address logging** (auth events)
2. ✅ **Add request context middleware** (user + IP in every request)
3. ⏳ Redis rate limiting integration (infrastructure ready)
4. ⏳ Ban system endpoints (schema ready)

### Medium Priority
1. Database session management (optional - JWT works well for mobile)
2. Account-based rate limiting (complement IP-based)
3. User state tracking (onboarded/verified flags)
4. Session cleanup job (remove expired sessions)

### Low Priority
1. OTP authentication (we use email/password + CAPTCHA)
2. Multiple device sessions (one JWT per device is simpler)
3. Club/group features (not in our MVP)

## Conclusion

Duolicious's route protection is **production-tested with 100K+ users**. Their patterns ensure:

- **Robust security**: Multi-layer defense
- **Abuse prevention**: IP + account rate limiting
- **Flexibility**: Different rules for different user states
- **Auditability**: Full IP and session logging

Our implementation adopts their core security principles while maintaining our unique features (music matching, CAPTCHA, Rust performance). The database schema is ready for advanced features when needed.
