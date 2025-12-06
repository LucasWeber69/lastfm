# Performance and Security Improvements

This document outlines performance and security improvements inspired by the Duolicious backend architecture.

## Improvements Implemented

### 1. Email Normalization (Anti-Abuse)

**Inspired by**: Duolicious's `antiabuse/antispam/signupemail` module

**Implementation**: `services/email_normalization.rs`

Prevents users from creating multiple accounts using email variations:
- Removes dots from Gmail addresses (`john.doe@gmail.com` → `johndoe@gmail.com`)
- Removes plus addressing (`user+tag@example.com` → `user@example.com`)
- Treats `googlemail.com` and `gmail.com` as the same domain
- Case-insensitive email matching

**Example**:
```rust
let normalized = normalize_email("John.Doe+spam@Gmail.com");
// Result: "johndoe@gmail.com"
```

**Benefits**:
- Prevents email abuse and duplicate accounts
- Blocks users from bypassing bans with simple email variations
- Reduces spam account creation

### 2. Rate Limiting Infrastructure

**Inspired by**: Duolicious's Flask-Limiter with Redis

**Implementation**: `middleware/rate_limit.rs`

Provides foundation for rate limiting:
- IP-based rate limiting
- Configurable limits and time windows
- In-memory implementation (TODO: Redis for production)
- Cleanup of old entries

**Production Recommendations**:
1. Use Redis for distributed rate limiting
2. Implement per-account rate limiting (not just IP-based)
3. Add different limits for different endpoints
4. Implement graduated throttling (slow down before blocking)

**Example Configuration** (TODO):
```rust
// In main.rs
let rate_limiter = RateLimiter::new(60, Duration::from_secs(60)); // 60 req/min
app.layer(middleware::from_fn(rate_limit_middleware))
```

### 3. Security Best Practices from Duolicious

#### Database Security
- ✅ **Parameterized Queries**: All queries use bind parameters (SQLx)
- ✅ **Transactions**: User updates wrapped in transactions for atomicity
- 📝 **TODO**: Add database constraints similar to Duolicious (CHECK constraints, UNIQUE constraints)

#### Authentication Security
- ✅ **Strong Password Hashing**: Argon2 (same as Duolicious uses bcrypt/argon2)
- ✅ **JWT Tokens**: With expiration (30 days, configurable)
- ✅ **Bearer Token Authentication**: Standard HTTP authentication
- 📝 **TODO**: Session management with Redis (like Duolicious's duo_session table)

#### Input Validation
- ✅ **Type-safe Models**: Rust's type system provides compile-time validation
- 📝 **TODO**: Add runtime validation similar to Pydantic in Duolicious
- 📝 **TODO**: Add content moderation for user-generated content

### 4. Performance Optimizations

#### Inspired by Duolicious Patterns:

1. **Database Indexing** (TODO):
   ```sql
   CREATE INDEX idx_users_email_normalized ON users(LOWER(email));
   CREATE INDEX idx_users_lastfm ON users(lastfm_username);
   CREATE INDEX idx_scrobbles_user_artist ON scrobbles_cache(user_id, artist_name);
   ```

2. **Caching** (TODO):
   - Cache compatibility scores (Redis)
   - Cache top artists per user
   - Cache discover profiles

3. **Background Jobs** (TODO):
   - Async scrobble syncing
   - Periodic compatibility recalculation
   - Photo processing

### 5. Anti-Abuse Measures (TODO)

From Duolicious's extensive anti-abuse system:

1. **Bad Email Domain Blocking**:
   - Maintain list of disposable email providers
   - Block known spam domains
   - Whitelist known good domains

2. **Content Moderation**:
   - Filter inappropriate usernames
   - Filter bio content for spam/abuse
   - Report system for user-generated content

3. **Age Verification**:
   - Proper age gate (18+)
   - Birth date validation
   - Prevent underage users

4. **IP-based Protection**:
   - Block VPN/proxy IP ranges (optional)
   - Detect and prevent automated signups
   - Implement CAPTCHA for suspicious activity

## Migration Path

### Phase 1: Immediate (Implemented ✅)
- [x] Email normalization for duplicate prevention
- [x] Rate limiting infrastructure
- [x] Enhanced documentation

### Phase 2: Short-term (Recommended)
- [ ] Redis integration for rate limiting
- [ ] Database indexes for performance
- [ ] Session management with Redis
- [ ] Add bad email domain list

### Phase 3: Medium-term
- [ ] Background job system for scrobble syncing
- [ ] Caching layer for compatibility scores
- [ ] Content moderation system
- [ ] Report and block functionality

### Phase 4: Long-term
- [ ] Advanced anti-abuse measures
- [ ] Age verification system
- [ ] Admin dashboard for moderation
- [ ] Analytics and monitoring

## Testing Rate Limiting

To test rate limiting in development:

```bash
# Make rapid requests to test rate limit
for i in {1..100}; do
  curl http://localhost:8000/discover \
    -H "Authorization: Bearer YOUR_TOKEN"
  echo "Request $i"
done
```

Expected: After hitting the limit, you should receive 429 (Too Many Requests) responses.

## Configuration

Add to `.env` for production:

```env
# Rate Limiting
REDIS_URL=redis://localhost:6379
RATE_LIMIT_REQUESTS_PER_MINUTE=60
RATE_LIMIT_REQUESTS_PER_SECOND=12

# Email Validation
BLOCK_DISPOSABLE_EMAILS=true
REQUIRE_EMAIL_VERIFICATION=true
```

## Comparison with Duolicious

| Feature | Duolicious | Our Implementation | Status |
|---------|-----------|-------------------|---------|
| Language | Python (Flask) | Rust (Axum) | ✅ Different but equivalent |
| Database | PostgreSQL | MySQL | ✅ Both SQL |
| Rate Limiting | Flask-Limiter + Redis | In-memory (TODO: Redis) | 🔄 In Progress |
| Email Normalization | Custom Python | Custom Rust | ✅ Implemented |
| Session Management | PostgreSQL table | JWT only | 📝 TODO |
| Anti-abuse | Extensive | Basic | 🔄 In Progress |
| Content Moderation | Yes | No | 📝 TODO |
| Background Jobs | Cron + Python | No | 📝 TODO |

## References

- Duolicious Backend: https://github.com/duolicious/duolicious-backend
- Flask-Limiter: https://flask-limiter.readthedocs.io/
- Argon2: https://en.wikipedia.org/wiki/Argon2
- JWT Best Practices: https://tools.ietf.org/html/rfc8725

---

**Last Updated**: 2025-12-06
**Based on**: Duolicious Backend Analysis
