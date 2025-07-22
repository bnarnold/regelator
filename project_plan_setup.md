# Project Plan - Setup Phase

## Project Status: 🏗️ **Foundation**

**Current Phase:** Initial setup and basic infrastructure
**Started:** 2025-07-22
**Next Review:** After Story 1.3 completion

## Stories - Setup Phase

### Story 1.1: Basic Web Server ✅ **Completed**
**Acceptance Criteria:**
- [x] Axum server running on port 8000
- [x] Health check endpoint `/health` returns "OK"
- [x] Server starts without errors

**Testing:**
- Manual: `curl http://localhost:8000/health` returns "OK"

---

### Story 1.2: Basic HTML Templating ✅ **Completed**
**Acceptance Criteria:**
- [x] Add `minijinja` for HTML templating
- [x] Create basic HTML page template with proper structure
- [x] Add route for root `/` that returns rendered HTML
- [x] Include basic CSS framework (pico.css) for styling

**Testing:**
- Manual: Visit `http://localhost:8000/` and see properly formatted HTML page
- Manual: Verify page has basic styling and responsive design

---

### Story 1.3: Static File Serving ⏳ **Pending**
**Acceptance Criteria:**
- [ ] Configure Axum to serve static files (CSS, JS, images)
- [ ] Create `static/` directory structure
- [ ] Add pico.css as static asset
- [ ] Verify static assets load correctly in browser

**Testing:**
- Manual: CSS styles apply correctly to HTML pages
- Manual: Static assets accessible via direct URLs

---

### Story 1.4: Database Setup ⏳ **Pending**
**Acceptance Criteria:**
- [ ] Add `diesel` and SQLite dependencies
- [ ] Set up database connection pool
- [ ] Create initial migration for basic schema
- [ ] Add database health check to existing `/health` endpoint

**Testing:**
- Manual: Application starts with database connection
- Manual: `/health` endpoint confirms database connectivity
- Unit: Database connection pool works correctly

---

## Next Phase: Core Functionality

### Story 2.1: Rules Data Model ⏳ **Pending**
**Description:** Define data structures for Ultimate Frisbee rules
**Priority:** High
**Estimate:** Medium

### Story 2.2: Basic Rules API ⏳ **Pending**
**Description:** REST endpoints for fetching rules
**Priority:** High
**Estimate:** Medium

### Story 2.3: Simple Rules Browser ⏳ **Pending**
**Description:** HTML interface to browse rules
**Priority:** High
**Estimate:** Large

---

## Progress Tracking

**Stories Completed:** 2/4 setup stories (50%)
**Current Velocity:** 1 story/session (baseline)
**Blockers:** None

## Notes

- Starting with minimal viable setup to establish development workflow
- Each story builds incrementally on previous work
- Focus on getting basic infrastructure solid before adding features
- All stories should result in a working, testable application state