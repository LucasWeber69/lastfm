# Comprehensive Alovoa Analysis

## Deep Dive Analysis - Security, Matching & Performance

Based on complete code review of https://github.com/Alovoa/alovoa

**Tech Stack**: Java Spring Boot + PostgreSQL + Thymeleaf templates

---

## 1. Security Architecture

### Authentication & Authorization

**Key Features:**
1. **Spring Security** with comprehensive filter chain
2. **CAPTCHA Required** for login (image-based, IP-hashed)
3. **Remember-Me Tokens** (BCrypt hashed, 2-week expiry)
4. **Session Management**: Max 10 concurrent sessions per user
5. **OAuth2 Support** for external logins
6. **CSRF Protection** disabled (REST API)

```java
// Security Configuration Highlights
- ROLE_USER and ROLE_ADMIN separation
- Public endpoints: /register, /login, /captcha, /password, /media/*
- Protected: Everything else requires authentication
- Session fixation protection
- Logout deletes both JSESSIONID and remember-me cookies
```

### CAPTCHA System (Anti-Bot)

```java
// IP-based CAPTCHA
- Generate unique captcha per IP (MD5 hashed with salt)
- Store in database with expiration
- Validate on login: correct text + matching IP hash
- Auto-delete old captcha on new generation (1 per IP)
- Visual: distortion + noise lines for bot resistance
```

**Key Insight**: CAPTCHA tied to IP prevents automated attacks even if someone steals captcha image.

### Password Security

```java
// BCryptPasswordEncoder (Spring Security default)
- Strength: 10 rounds (configurable)
- Salted automatically
- One-way hashing (cannot decrypt)
```

---

## 2. Search & Matching Algorithm

### Multi-Stage Fallback Search

Alovoa uses **progressive relaxation** when no matches found:

#### Stage 1: Normal Search (Strict Criteria)
```java
// Filters applied:
- Location: Within specified radius (Haversine distance)
- Age: Min/max age preferences (mutual respect)
- Gender: User's preferred genders
- Intentions: Relationship goals match
- Miscellaneous: Body type, kids, etc.
- Interests: Common interests filter
- Blocks: Exclude blocked/hidden users
- Legal age separation: Under 18 cannot see 18+
```

#### Stage 2: Compatible Search (Relaxed)
If Stage 1 returns 0 results:
```java
// Remove filters:
- Intentions set to ALL
- Misc info filters removed
- Keep: location, age, gender, blocks
```

#### Stage 3: Global Search (Last Resort)
If Stage 2 returns 0 results:
```java
// Remove: Location filter
// Keep only: age, gender, blocks
// Search worldwide
```

### Sorting Options

```java
SORT_DEFAULT (0): Latest donation date DESC, then creation date DESC
SORT_ACTIVE_DATE (1): Last active date DESC
SORT_DONATION_TOTAL (2): Total donations DESC (supports development)
SORT_NEWEST_USER (3): Account creation date DESC
SORT_DISTANCE (4): Distance to current user ASC (client-side)
SORT_INTEREST (5): Common interests count DESC, then distance DESC
```

### Location Handling

```java
// Latitude/Longitude bounding box calculation
deltaLat = distance_km / LATITUDE_CONSTANT (110.574 km/degree)
deltaLong = distance_km / (LONGITUDE_CONSTANT * cos(latitude))

// SQL: WHERE lat BETWEEN minLat AND maxLat 
//      AND long BETWEEN minLong AND maxLong
```

**Performance**: Bounding box filter is much faster than Haversine distance calculation for all rows.

---

## 3. Performance Optimizations

### Database Query Strategy

1. **Conditional Queries** based on filter presence:
   ```java
   if (miscInfos.empty && interests.empty):
       usersSearchNoExtras()  // Fastest
   elif (miscInfos present, interests empty):
       usersSearchMisc()
   elif (miscInfos empty, interests present):
       usersSearchInterest()
   else:
       usersSearchInterestMisc()  // Slowest
   ```

2. **Pagination**: `PageRequest.of(0, SEARCH_MAX, sort)` limits results

3. **Lazy Loading**: User entities load profile data only when needed

### Active User Tracking

```java
// Update active date on every search
user.getDates().setActiveDate(new Date());

// Used for:
- "Last active" display
- Prioritize active users in search results
- Clean up abandoned accounts
```

---

## 4. Anti-Abuse Measures

### Block System

```java
// Three types:
1. Blocks: User actively blocked someone
2. BlockedBy: Users who blocked current user
3. Hidden: User swiped left/hidden someone

// All excluded from search results
```

### Legal Age Separation

```java
// Hard separation to comply with laws
if (user.age >= 18 && minAge < 18):
    minAge = 18  // Adult cannot see minors

if (user.age < 18 && maxAge >= 18):
    maxAge = 17  // Minor cannot see adults
```

### Location Validation

```java
// Geocoder validates lat/long
// Updates user's country based on coordinates
// Prevents fake locations (to some extent)
```

---

## 5. Key Differences from Duolicious

| Feature | Duolicious | Alovoa | Recommendation |
|---------|-----------|--------|----------------|
| **Language** | Python | Java Spring Boot | ✅ Rust (our choice) |
| **Matching** | Personality vectors | Location + preferences | ✅ Music vectors |
| **Captcha** | ❌ None | ✅ IP-based image captcha | ⚠️ **Add captcha** |
| **Session** | DB + OTP | Spring session + remember-me | ✅ JWT (simpler) |
| **Fallback Search** | ❌ Single pass | ✅ 3-stage progressive | ⚠️ **Implement** |
| **Legal Age Split** | ❌ Manual | ✅ Automatic | ⚠️ **Add if needed** |
| **Location Filter** | ❌ Vector distance | ✅ Bounding box first | ⚠️ **Optimize queries** |

---

## 6. Security Best Practices from Alovoa

### 1. **CAPTCHA on Login**
```java
// Prevents:
- Brute force attacks
- Automated account creation
- Bot scraping

// Implementation:
- Generate unique per IP
- Image with distortion
- One-time use
- Database stored with expiration
```

### 2. **IP-Based Rate Limiting** (Implicit)
```java
// CAPTCHA per IP = natural rate limit
// Only 1 valid captcha per IP at a time
// Forces delay between login attempts
```

### 3. **Session Security**
```java
// Features:
- Session fixation protection (new session ID after login)
- Max 10 concurrent sessions per user
- Expired session redirect to /logout
- Remember-me with separate cookie
```

### 4. **Comprehensive Route Protection**
```java
// Public routes explicitly listed
// Default: authenticated required
// Admin routes separate authority check
// Static assets (CSS/JS/images) public
```

---

## 7. Performance Patterns from Alovoa

### 1. **Conditional Query Selection**
Instead of one complex query with many optional filters, use separate optimized queries:

```java
// Fast path: No filters
SELECT * FROM users WHERE location_match AND age_match

// Medium path: Interest filter
SELECT * FROM users u 
JOIN user_interests ui ON u.id = ui.user_id
WHERE location_match AND interest IN (...)

// Slow path: Interest + misc
SELECT * FROM users u
JOIN user_interests ui ON u.id = ui.user_id  
JOIN user_misc_info mi ON u.id = mi.user_id
WHERE location_match AND interest IN (...) AND misc IN (...)
```

**Benefit**: Database can optimize each query path separately.

### 2. **Bounding Box for Geospatial**
```java
// FAST: Filter by rectangle first
WHERE lat BETWEEN minLat AND maxLat 
  AND long BETWEEN minLong AND maxLong

// Then calculate exact distance if needed (client-side or limited set)
distance = haversine(user.lat, user.long, candidate.lat, candidate.long)
```

**Benefit**: Reduces rows before expensive distance calculation.

### 3. **Progressive Search Fallback**
```java
// Try strict search first
results = search(all_filters)

if (results.empty):
    // Relax some filters
    results = search(fewer_filters)
    
if (results.empty):
    // Go global
    results = search(minimal_filters)
```

**Benefit**: Better UX (always show some users) without slow query by default.

---

## 8. Recommendations for Our App

### Priority 1: Security Enhancements

1. **Add CAPTCHA** (from Alovoa)
   ```rust
   // Use captcha crate: https://crates.io/crates/captcha
   // Store in Redis with IP hash as key
   // Require on login + register
   ```

2. **Implement Duolicious's Session Tracking**
   ```sql
   -- Already in migration 002!
   CREATE TABLE sessions (...)
   ```

3. **Add Request Validation**
   ```rust
   // Validate all inputs
   // Reject suspicious patterns
   // Log failed attempts
   ```

### Priority 2: Search Optimization

1. **Progressive Fallback Search** (from Alovoa)
   ```rust
   // Stage 1: Strict filters (compatibility, location, prefs)
   // Stage 2: Relaxed (remove compatibility requirement)
   // Stage 3: Global (remove location)
   ```

2. **Bounding Box Location Filter** (from Alovoa)
   ```rust
   // Use lat/long bounds BEFORE distance calculation
   // Much faster for large datasets
   ```

3. **Conditional Query Paths** (from Alovoa)
   ```rust
   // Different queries based on active filters
   // Let database optimizer do its job
   ```

### Priority 3: User Experience

1. **Active User Prioritization** (from Alovoa)
   ```sql
   ALTER TABLE users ADD COLUMN last_active_at TIMESTAMP;
   -- Update on every API call
   -- Sort search by last_active_at DESC
   ```

2. **Multiple Sort Options** (from Alovoa)
   ```rust
   enum SortBy {
       Compatibility,  // Default
       LastActive,
       Newest,
       Distance,
       CommonArtists,
   }
   ```

---

## 9. Combined Best Practices (Duolicious + Alovoa)

### Security Stack
```
✅ Argon2 password hashing (Duolicious)
✅ JWT Bearer tokens (our current)
⚠️ + CAPTCHA on login (Alovoa) - TO ADD
⚠️ + IP tracking (Duolicious) - Schema ready
⚠️ + Session management (both) - Schema ready
✅ Email normalization (Duolicious) - Done
⚠️ + Ban system (Duolicious) - Schema ready
✅ Rate limiting infrastructure (Duolicious) - Done
```

### Matching Algorithm
```
✅ Vector-based compatibility (Duolicious) - Done
⚠️ + Progressive fallback search (Alovoa) - TO ADD
⚠️ + Bounding box location filter (Alovoa) - TO ADD
✅ Multiple weighting factors (our enhancement)
```

### Performance
```
✅ Database indexes (Duolicious migration) - Done
⚠️ + Conditional query paths (Alovoa) - TO ADD
⚠️ + Search result caching (Duolicious) - Schema ready
⚠️ + Active user tracking (Alovoa) - TO ADD
```

---

## 10. Implementation Checklist

### Phase 1: Critical Security (From Both)
- [ ] CAPTCHA system (Alovoa pattern)
- [ ] IP address logging (Duolicious schema)
- [ ] Session management (both - schema ready)
- [ ] Request validation middleware

### Phase 2: Search Optimization (Alovoa)
- [ ] Progressive fallback search (3 stages)
- [ ] Bounding box location filter
- [ ] Conditional query selection
- [ ] Multiple sort options

### Phase 3: User Experience
- [ ] Active user tracking
- [ ] Last active timestamp display
- [ ] Common artists display on cards
- [ ] Distance display

### Phase 4: Anti-Abuse (Duolicious)
- [ ] Ban system integration
- [ ] Report system endpoints
- [ ] Bot detection scoring
- [ ] Redis rate limiting

---

## Summary

**Alovoa Strengths**:
- CAPTCHA anti-bot protection
- Progressive fallback search
- Location optimization (bounding box)
- Comprehensive Spring Security setup

**Duolicious Strengths**:
- Advanced vector matching
- Redis-based distributed rate limiting
- IP tracking and banning
- 4-pass search with caching

**Our Unique Value**:
- Music-based matching (Last.fm integration)
- Rust performance and safety
- Vector similarity for music preferences
- Type-safe compile-time guarantees

**Best Strategy**: Merge security from both (CAPTCHA from Alovoa + IP tracking from Duolicious), use Alovoa's search fallback pattern, and keep our music-based vector matching as the differentiator.

---

**Analysis Date**: 2025-12-06
**Codebases Analyzed**: 
- Duolicious: ~250 files (Python)
- Alovoa: ~171 Java files
**Key Insight**: Both use different matching strategies but share security patterns we should adopt.
