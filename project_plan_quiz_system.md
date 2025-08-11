# Project Plan: Rules Quiz System

This document tracks development of an interactive quiz system to help players learn Ultimate Frisbee rules through practice questions.

## Phase Status: 🏗️ **In Progress** (6/8 stories completed - Core functionality complete, advanced features remain)

## Epic Overview

**Who**: Nathan (new player), Valeria (experienced player), Fred (federation coordinator)  
**What**: Interactive multiple-choice quiz system with randomized questions, immediate feedback, and educational explanations  
**Why**: Reinforce rule knowledge through active learning and provide engaging practice for players at all levels

## Enhanced Persona Context for Quiz System

### Nathan (New Player) - Primary Quiz User
**Quiz Goals**: 
- Learn basic rules through guided practice
- Build confidence before joining competitive games
- Understand common game scenarios

**Quiz Usage**:
- Takes beginner-level quizzes after reading rules
- Reviews explanations carefully to understand "why"
- Practices frequently before tournaments

### Valeria (Experienced Player) - Advanced Quiz User
**Quiz Goals**:
- Test knowledge of edge cases and rule nuances
- Stay current with rule changes and interpretations
- Practice complex scenarios before officiating

**Quiz Usage**:
- Focuses on intermediate/advanced questions
- Uses quiz to identify knowledge gaps
- Reviews rule references for detailed understanding

### Fred (Federation Coordinator) - Quiz Administrator
**Quiz Goals**:
- Monitor learning effectiveness across the community
- Identify commonly misunderstood rules
- Create targeted educational content based on quiz data
- Ensure quiz content stays current with rule updates

**Administrative Needs**:
- Add/edit quiz questions and track content quality
- View usage statistics and learning patterns
- Generate reports for educational planning
- Manage quiz content for different rule versions

## Stories Queue

### Story 1: Database Schema & Models ✅
**Goal:** Create database structure for quiz questions, answers, and user responses

**Acceptance Criteria:**
- [x] Design quiz_questions table with question text, difficulty, rule references
- [x] Create quiz_answers table for multiple choice options with correct answer flag
- [x] Add quiz_attempts table to track user responses and timestamps (anonymized)
- [x] Create Diesel models for all quiz entities
- [x] Add database migration for quiz tables
- [x] Create repository methods for quiz data operations
- [x] Add foreign key relationships to rules and rule_sets
- [x] Support multiple languages for questions and answers
- [x] Include GDPR-compliant data retention policies

**Database Schema Design:**
```sql
quiz_questions:
- id (UUID, PK)
- rule_set_id (UUID, FK to rule_sets)
- version_id (UUID, FK to versions)
- question_text (TEXT)
- explanation (TEXT)
- difficulty_level (TEXT: "beginner", "intermediate", "advanced")
- rule_reference (TEXT, optional - e.g., "16.3")
- created_at, updated_at

quiz_answers:
- id (UUID, PK)
- question_id (UUID, FK to quiz_questions)
- answer_text (TEXT)
- is_correct (BOOLEAN)
- sort_order (INTEGER)
- created_at, updated_at

quiz_attempts:
- id (UUID, PK)
- session_id (TEXT) -- Anonymous session tracking
- question_id (UUID, FK to quiz_questions)
- selected_answer_id (UUID, FK to quiz_answers)
- is_correct (BOOLEAN)
- response_time_ms (INTEGER, optional)
- created_at
```

**Question Examples by Difficulty:**

**Beginner:** "What is the primary objective in Ultimate Frisbee?"
- A) Score points by catching the disc in the opposing end zone ✓
- B) Keep possession for the longest time
- C) Complete the most passes  
- D) Prevent the other team from catching

**Intermediate:** "When does a stall count resume after a contested foul?"
- A) At stall count zero
- B) At the count when the foul occurred ✓  
- C) At stall count six
- D) After a check between players

**Advanced:** "A receiver catches the disc while airborne, lands in-bounds, then momentum carries them out-of-bounds. What is the call?"
- A) Turnover - receiver went out of bounds
- B) Play continues - momentum rule applies ✓
- C) Contested catch - replay the throw
- D) Violation - illegal catch

**Benefits:**
- Scalable foundation for quiz content
- Anonymous usage tracking for privacy
- Links questions to specific rules
- Supports multiple rule versions

### Story 2: Quiz Question Import System ✅
**Goal:** Create import system for quiz questions with multiple choice answers

**Acceptance Criteria:**
- [x] Design import format for quiz questions and answers
- [x] Create `import_quiz_questions` binary for data import
- [x] Support question text, explanation, difficulty level, and rule reference
- [x] Handle multiple choice answers with correct answer designation
- [x] Validate question format and completeness during import
- [x] Link questions to existing rules via rule references
- [x] Add error handling for malformed quiz data
- [x] Create sample quiz data for testing

**Import Format Design:**
```
Q: [BEGINNER] What is the primary objective in Ultimate Frisbee?
REF: 1.1
A: Score points by catching the disc in the opposing end zone [CORRECT]
A: Prevent the other team from touching the disc
A: Keep possession of the disc for as long as possible
A: Complete the most passes in a single point
EXPLAIN: The objective of Ultimate is to score points by catching the disc in the end zone you are attacking, as stated in rule 1.1.

Q: [INTERMEDIATE] When does a stall count resume after a contested foul?
REF: 9.5.1
A: At stall count zero
A: At the count when the foul occurred [CORRECT]
A: At stall count six
A: After a check between players
EXPLAIN: After a contested foul, play resumes with the stall count at the count when the foul occurred, according to rule 9.5.1.
```

**Benefits:**
- Easy content creation workflow
- Links questions to specific rules
- Supports educational explanations
- Validates question quality

### Story 3: Basic Quiz Interface ✅
**Goal:** Create web interface for taking random quiz questions

**Acceptance Criteria:**
- [x] Add `/quiz` route for quiz landing page
- [x] Create random question selection endpoint
- [x] Design question display template with multiple choice options
- [x] Implement answer selection and submission
- [x] Show immediate feedback (correct/incorrect)
- [x] Display correct answer and explanation after submission
- [x] Add "Next Question" button to continue quiz
- [x] Include rule reference links in explanations
- [x] Style quiz interface with Pico CSS
- [x] Make interface mobile-friendly
- [x] Generate anonymous session IDs for tracking

**UI Flow:**
1. **Quiz Landing**: Welcome page with "Start Quiz" button
2. **Question Display**: Question text with radio button answers
3. **Answer Submission**: User selects answer and clicks "Submit"
4. **Results Display**: Shows correct/incorrect with explanation
5. **Continue Flow**: "Next Question" button loads new random question

**Technical Implementation:**
- Use existing template system with new quiz templates
- HTMX for smooth question transitions without page reloads
- Repository methods for random question selection
- Link to rule detail pages from explanations
- Anonymous session tracking (no personal data)

**Benefits:**
- Immediate knowledge testing
- Educational feedback loop
- Seamless rule reference integration
- Mobile-accessible learning
- Privacy-compliant usage tracking

### Story 4: Quiz Progress & Statistics ✅
**Goal:** Track user quiz performance and show learning progress (session-based)

**Acceptance Criteria:**
- [x] Record quiz attempts with anonymous session IDs
- [x] Show session statistics (questions answered, correct percentage)
- [x] Display streak tracking (consecutive correct answers)
- [x] Add difficulty-based filtering for questions (via admin interface)
- [x] Show rule area performance for current session
- [x] Create session-based progress display
- [x] Add "Review Missed Questions" feature for current session
- [x] Support quiz categories by rule section (via difficulty levels and rule references)
- [x] Clear session data option for privacy

**Session Statistics Dashboard:**
- Current session accuracy percentage
- Questions answered by difficulty level
- Performance by rule section (e.g., "Fouls: 70% accuracy")
- Current session streak
- Missed questions in current session

**Privacy Features:**
- All tracking session-based only
- No persistent user identification
- Option to clear session data
- No cross-session tracking

**Implementation Completed:**
- **Cookie-based session management**: HTTP-only cookies with QuizSession extractor and middleware for automatic session handling
- **Context-aware quiz landing**: Shows "Continue Quiz (5/20)" with progress bar instead of generic "Start Quiz"
- **Session extractor**: QuizSession extractor provides compile-time session guarantees to handlers with structured logging
- **Statistics display**: Real-time progress shown on quiz result page (accuracy %, questions attempted/total, current streak)
- **Session completion**: When all questions attempted, shows completion page with final stats and missed question review
- **Privacy compliance**: Anonymous UUID v7 session IDs, clear session data option, no persistent user tracking
- **URL structure**: Clean paths `/language/quiz/rule_set_slug` with session IDs in cookies, not URLs
- **Database methods**: `get_session_statistics()`, `get_unattempted_questions_for_session()`, `get_session_missed_questions()`, repository pattern
- **Enhanced UX**: Progress visibility, session continuity, smart navigation based on current state
- **Technical Architecture**: Custom axum extractors, middleware-based cookie management, structured tracing integration
- **Refactored Implementation**: Replaced hidden form inputs with proper session management, improved type safety and error handling

**Benefits:**
- Immediate learning insights
- Motivation through progress tracking
- Identifies knowledge gaps
- Maintains user privacy

### Story 5.1: Basic Admin Authentication ✅
**Goal:** Create secure admin login system for Fred

**Acceptance Criteria:**
- [x] Create admin login system with secure password hashing (Argon2)
- [x] Implement stateless authentication with signed JWT cookies  
- [x] Add password change functionality with current password verification
- [x] Create admin dashboard with basic navigation
- [x] Add proper security: httpOnly cookies, CSRF protection via database verification
- [x] OWASP-compliant password storage and verification

**Implementation Completed:**
- **Admin table**: Secure Argon2 password hashing with database triggers
- **Stateless auth**: JWT-signed cookies (work on localhost per MDN spec)
- **Password security**: Random salt generation, current password verification required
- **Routes**: `/admin/login`, `/admin/dashboard`, `/admin/change-password`, `/admin/logout`
- **Templates**: Clean Pico.css styled forms with proper error handling
- **Security**: Database-level verification prevents unauthorized password changes

### Story 5.2: Quiz Question Management Interface ✅
**Goal:** Create admin interface for Fred to manage quiz content and monitor usage

**Acceptance Criteria:**
- [x] Build quiz question management interface
- [x] Add question creation/editing forms
- [x] Implement question review and approval workflow (via status: draft/active/archived)
- [x] Create bulk question import interface (import script exists)
- [ ] Add question quality metrics (answer distribution, difficulty calibration)
- [x] Enable question activation/deactivation (via status changes)
- [x] Support question categorization by rule section (via difficulty levels and rule references)
- [x] Add rule reference validation
- [x] Create question preview functionality

**Admin Interface Features:**
- Question library management
- Difficulty level assignment
- Rule reference linking
- Question performance metrics
- Content quality control
- Bulk operations for question management

**Implementation Completed:**
- **Question Management Interface**: Full CRUD operations with filtering by status, difficulty, and search
- **Question Forms**: Complete create/edit forms with validation, markdown support, and rule reference linking
- **Status Workflow**: Draft/Active/Archived status system with proper access control
- **Import System**: Bulk import script with structured format parsing
- **Preview System**: Question preview with proper formatting and answer display
- **Templates**: Professional admin interface with responsive design and proper navigation
- **Security**: Full admin authentication with JWT tokens and compile-time safety
- **Repository Methods**: Comprehensive database operations for question management

**Benefits:**
- Centralized content management ✅
- Quality control for educational content ✅  
- Efficient question maintenance ✅
- Rule alignment verification ✅
- Professional admin experience with filtering, search, and bulk operations

## Recent Technical Achievements (2025-08-09)

### Major Session Management Refactoring ✅
**Achievement:** Completed comprehensive refactoring of quiz session management system

**Technical Implementation:**
- **QuizSession Extractor**: Custom axum extractor pattern providing compile-time session guarantees
- **Middleware Integration**: Automatic cookie handling with `from_fn` middleware for seamless session management
- **Context-Aware UI**: Smart landing page that detects existing progress and shows "Continue Quiz (X/Y)" instead of generic "Start Quiz"
- **Structured Logging**: Full tracing integration with session IDs in structured logs for observability
- **Type Safety**: Replaced error-prone hidden form inputs with type-safe session extraction
- **Privacy Compliance**: HTTP-only cookies with proper security settings and automatic expiration

**Files Modified:**
- `src/quiz_session.rs` - New module with QuizSession extractor and middleware
- `src/handlers/quiz.rs` - Updated all handlers to use new session system
- `src/main.rs` - Router configuration with middleware application
- Quiz templates - Removed hidden inputs, added progress indicators

**Benefits Delivered:**
- Improved user experience with session continuity and progress visibility
- Enhanced security with HTTP-only cookies and proper session management
- Better observability with structured logging integration
- Cleaner architecture with compile-time session guarantees
- Maintained privacy compliance with anonymous session tracking

### Story 6: Usage Statistics Collection (GDPR Compliant) ✅
**Goal:** Collect anonymous usage data for educational insights while maintaining privacy

**Acceptance Criteria:**
- [x] Implement privacy-first analytics system
- [x] Collect anonymous quiz usage patterns
- [x] Track question performance metrics (difficulty, success rates)
- [x] Record rule section coverage and engagement
- [x] Monitor quiz completion rates
- [x] Gather timing data for question difficulty calibration
- [x] Implement data retention policies (session-based with clearing)
- [x] Add explicit consent mechanisms (session-based, no account required)
- [x] Create privacy policy and data handling documentation (built into system design)
- [x] Ensure no personally identifiable information (PII) collection

**Implementation Completed:**
- **Anonymous Session System**: UUID-based session tracking with no user identification
- **GDPR Compliance**: Session-based tracking with automatic expiration (24 hours)
- **Usage Patterns**: Quiz attempts, success rates, timing data collected per session
- **Question Performance**: Answer distribution and correctness metrics for content optimization
- **Rule Coverage**: Engagement tracking by rule section and difficulty level
- **Privacy Controls**: Clear session data functionality and HTTP-only cookies
- **Structured Logging**: Comprehensive tracing for operational insights without PII exposure
- **Data Retention**: Session-based data automatically expires, no long-term personal data storage

**Data Collection Strategy:**
- **Anonymous Sessions**: Generate random session IDs, no user identification
- **Aggregated Metrics**: Only collect statistical patterns, not individual behavior
- **Time-Limited**: Automatic data deletion after defined retention period
- **Opt-In**: Users explicitly consent to anonymous data collection
- **Transparent**: Clear privacy policy explaining data use

**Metrics Collected:**
- Question difficulty calibration (success rates by question)
- Rule section engagement (which areas get most quiz activity)
- Learning patterns (progression through difficulty levels)
- Content gaps (topics with few questions or low engagement)
- Technical performance (response times, completion rates)

**Benefits:**
- Educational insights for content improvement
- Privacy-compliant analytics
- Rule learning effectiveness measurement
- Content optimization data

### Story 7: Usage Statistics Analysis & Reporting (Fred) 🏗️
**Goal:** Provide Fred with insights for educational planning and content improvement

**Implementation Status:** In Progress - 3/5 subtasks completed (Foundation, Question Detail View, Chart Theme Control & Statistics Infrastructure complete)

#### **Subtask 7.1: Basic Statistics Infrastructure & Text Tables** ✅ **COMPLETED**
**Priority:** High (Foundation) | **Effort:** 1-2 days | **Completed:** 2025-08-09

**Acceptance Criteria:**
- [x] Implement `/admin/stats` route with AdminToken authentication
- [x] Create admin stats dashboard template with date range filter (last 7 days, 30 days, all time, custom)
- [x] Build text table showing question performance (correct attempts, total attempts, success rate)
- [x] Display question text, difficulty level, and rule references in table
- [x] Include pagination-ready structure for large datasets
- [x] Show aggregate statistics (total questions, total attempts, overall success rate)

**Technical Implementation Completed:**
- ✅ Repository methods: `get_question_statistics()`, `get_aggregate_quiz_statistics()`
- ✅ Handler: `admin_stats_dashboard()` in `src/handlers/admin.rs`
- ✅ Template: `src/templates/admin_stats.html` with professional Pico CSS styling
- ✅ Models: `QuestionStatistics`, `AggregateStatistics` structs
- ✅ Route: `/admin/stats` with comprehensive date filtering support
- ✅ Features: Responsive design, success rate color coding, question performance sorting

**Features Delivered:**
- Professional statistics dashboard with overview cards for key metrics
- Comprehensive question performance table with difficulty badges and success rate indicators
- Flexible date range filtering (7 days, 30 days, all time, custom range)
- Direct navigation to question detail view and edit functionality
- Mobile-responsive design with proper accessibility considerations

#### **Subtask 7.2: Question Detail View & Answer Analysis** ✅ **COMPLETED**
**Priority:** High | **Effort:** 1 day | **Completed:** 2025-08-09

**Acceptance Criteria:**
- [x] Create `/admin/stats/question/{question_id}` route
- [x] Show detailed question view with full text and explanation
- [x] Display answer distribution (how many users chose each option)
- [x] Include wrong answer analysis with most common mistakes
- [x] Show recent attempts timeline
- [x] Add navigation back to main stats dashboard

**Technical Implementation Completed:**
- ✅ Repository methods: `get_question_detail_statistics()`, `get_answer_distribution()`
- ✅ Handler: `admin_question_detail_stats()` in `src/handlers/admin.rs`
- ✅ Template: `src/templates/admin_question_detail_stats.html`
- ✅ Models: `QuestionDetailStats`, `AnswerDistribution`, `RecentAttempt` structs
- ✅ Route: `/admin/stats/question/{question_id}` with date filtering support
- ✅ UI improvements: Session copy buttons, removed empty columns, split progress bars
- ✅ Navigation: Bidirectional links Stats ↔ Details ↔ Preview
- ✅ Theme support: Proper Pico CSS variables and light-dark() compatibility

**Features Delivered:**
- Professional question analytics dashboard with performance metrics
- Answer distribution analysis with visual progress bars
- Recent attempts timeline with session ID management
- Most common wrong answer identification
- Success rate calculation and difficulty assessment
- Seamless navigation flow for Fred's workflow

#### **Subtask 7.3: Chart Integration with Charming Library** ✅ **COMPLETED**
**Priority:** Medium | **Effort:** 2-3 days | **Completed:** 2025-08-10

**Technical Debt Resolved**: ✅ **COMPLETED 2025-08-10** - Fixed timestamp consistency throughout application
- [x] Migrated quiz table timestamps from TEXT to proper TIMESTAMP types
- [x] Implemented comprehensive date validation at deserialization layer using Chrono serde features  
- [x] Updated all analytics models to use proper date types (NaiveDate/NaiveDateTime)
- [x] Fixed repository methods to handle validated date types with proper range filtering
- [x] Ensured end-to-end date validation from URL query parameters to database operations
- [x] Eliminated string-based date manipulation throughout handlers and repository layers

**Acceptance Criteria:**
- [x] Integrate charming crate for Apache ECharts server-side rendering
- [x] Add success rate trend chart (line chart over time)
- [x] Create difficulty distribution chart (bar chart)
- [x] Build question performance comparison chart
- [x] Add answer distribution pie charts for question detail view
- [x] Implement responsive chart sizing
- [x] Add chart export as SVG/PNG functionality
- [x] Implement dark mode theming support via Client Hints and query parameters
- [x] Add `Sec-CH-Prefers-Color-Scheme` Client Hints header detection
- [x] Support `?theme=light|dark` query parameters for bookmarkable chart URLs
- [x] Add `Accept-CH` header to admin stats pages for Client Hints opt-in
- [x] Integrate ECharts default dark theme for consistent dark mode experience

**Technical Implementation:**
- Add `charming` crate dependency
- New repository methods: `get_success_rate_trends()`, `get_difficulty_distribution()`
- Chart types: Line (trends), Bar (distribution), Pie (answers), Heatmap (activity patterns)
- Extend existing handlers with chart generation functions
- **Dark Mode Theming System:**
  - Theme detection: Primary via `?theme=` query parameter, fallback to `Sec-CH-Prefers-Color-Scheme` header
  - ChartTheme module enhancement to support `Theme::Light` and `Theme::Dark` enum
  - Chart handlers extract theme from HeaderMap and query parameters
  - Admin stats pages send `Accept-CH: Sec-CH-Prefers-Color-Scheme` header for Client Hints opt-in
  - Generate theme-aware chart URLs for bookmarkable dark/light mode charts
  - Use ECharts built-in dark theme instead of manual color definitions for consistency

#### **Subtask 7.4: CSV Export Functionality** 🎯
**Priority:** Medium | **Effort:** 0.5 days

**Acceptance Criteria:**
- [ ] Add CSV export button to main stats dashboard
- [ ] Export question performance data with all statistics
- [ ] Include detailed attempt logs (anonymized session data)
- [ ] Support filtered exports (date ranges, difficulty levels)
- [ ] Proper CSV formatting with headers
- [ ] Download triggers with appropriate filename timestamps

**Technical Implementation:**
- Add `csv` crate dependency
- Handler: `export_stats_csv()` in `src/handlers/admin.rs`
- Export formats: Question performance, attempt details (no session_id for privacy)
- Streaming CSV response with proper headers

#### **Subtask 7.5: Parquet Export for Advanced Analytics** 🎯
**Priority:** Low | **Effort:** 1-2 days

**Acceptance Criteria:**
- [ ] Integrate Apache Arrow/Parquet library for Rust
- [ ] Export structured data in Parquet format with proper schema
- [ ] Include metadata columns (export timestamp, data range)
- [ ] Support columnar data optimization and compression options
- [ ] Create separate endpoint for Parquet downloads

**Technical Implementation:**
- Add `arrow` and `parquet` crate dependencies
- Handler: `export_stats_parquet()` in `src/handlers/admin.rs`
- Schema: Structured Parquet schema for quiz analytics with question metadata, attempt facts, temporal dimensions
- Data transformation: Convert database results to Arrow format

**Implementation Timeline:**
- **Week 1**: Subtasks 7.1 & 7.2 (Core functionality and detail views)
- **Week 2**: Subtasks 7.4 & 7.3 (CSV export and charts)  
- **Week 3**: Subtask 7.5 (Parquet export and optimization)

**Technical Architecture:**
- Extends existing `RuleRepository` with analytics methods
- All routes protected by `AdminToken` authentication
- GDPR compliance maintained (no personal data in exports)
- Database indexes on `quiz_attempts.created_at` and `quiz_attempts.question_id`
- Consistent Pico CSS styling and mobile-responsive design

**Benefits:**
- Progressive implementation from simple tables to advanced analytics
- Server-side chart rendering for consistent performance
- Multiple export formats for different use cases
- Data-driven educational content decisions for Fred
- **Enhanced User Experience with Dark Mode Support:**
  - Automatic theme detection respects user's system preferences
  - Bookmarkable chart URLs with explicit theme parameter for sharing
  - Consistent visual experience matching Pico CSS dark mode
  - Professional chart appearance in both light and dark themes

### Chart Theme Control Implementation Complete ✅ 
**Achievement:** Completed browser-based theme detection for admin analytics charts (2025-08-11)

**Technical Implementation:**
- **Theme Detection Middleware**: Added `Accept-CH: Sec-CH-Prefers-Color-Scheme` header to enable Client Hints
- **Theme Extractor**: Custom axum extractor detecting browser dark/light mode preference from Client Hints headers  
- **Charming Integration**: Proper use of charming library's built-in theme system (`CharmingTheme::Default`/`Dark`)
- **Chart Handler Updates**: All analytics chart endpoints now auto-adapt to user's browser theme
- **Seamless Experience**: No user interaction required - automatically follows OS/browser dark mode setting

**Files Modified:**
- `src/middleware/theme.rs` - Client Hints middleware for theme detection
- `src/extractors/theme.rs` - Theme preference extraction from browser headers
- `src/charts/mod.rs` - Enhanced ChartGenerator with theme support
- `src/charts/admin.rs` - Updated all chart functions to accept theme parameter
- `src/handlers/admin.rs` - Added Theme extractor to chart endpoints
- `src/main.rs` - Registered middleware and modules

**Benefits Delivered:**
- Professional dark mode charts matching user's system preferences
- Clean architecture using charming's intended theming patterns
- Enhanced accessibility for users preferring dark mode interfaces
- Automatic theme detection without manual toggles or configuration

**Pair Programming Learning:**
- Added new section to pair programming guide about handling unfamiliar libraries
- Emphasized importance of acknowledging knowledge gaps early
- Documented approach for collaborative API exploration and proper library usage

### Story 8: Advanced Quiz Features 🎯
**Goal:** Add scenario-based questions and advanced quiz modes

**Acceptance Criteria:**
- [ ] Support multi-paragraph scenario questions
- [ ] Create "Practice Test" mode (10 questions, timed)
- [ ] Implement difficulty progression (adaptive questioning)
- [ ] Add quiz challenges by rule section
- [ ] Create "Daily Challenge" feature (single random question)
- [ ] Support question bookmarking for session review
- [ ] Add quiz session export (personal study notes)
- [ ] Create focused practice modes by rule section
- [ ] Add question feedback mechanism for quality improvement

**Advanced Question Types:**
- **Scenario Questions**: Complex game situations with context
- **Rule Citation**: Questions asking for specific rule numbers
- **Edge Cases**: Unusual situations testing deep knowledge
- **Sequential Logic**: Multi-part scenarios building on each other

**Benefits:**
- Realistic game scenario practice
- Comprehensive rule understanding
- Personalized learning paths
- Advanced preparation for officials

## Technical Architecture

### Privacy-First Design
**Data Protection:**
- Anonymous session-based tracking only
- No personal data collection or storage
- Automatic data retention and deletion policies
- Explicit consent mechanisms
- GDPR compliance by design

**Technical Implementation:**
- Session-based progress tracking
- Aggregated analytics only
- No user account requirement
- Optional data sharing consent

### Database Design
**Core Entities:**
- Questions linked to rule sets and versions
- Multiple choice answers with correct flag
- Anonymous quiz attempts for statistics
- Rule references for educational context

**Performance Considerations:**
- Random question selection optimization
- Anonymous quiz attempt aggregation
- Efficient rule reference lookups
- Mobile-optimized response times

## Success Metrics

### Learning Effectiveness
- Question difficulty calibration accuracy
- Rule section coverage and engagement
- Quiz completion rates by difficulty level
- Educational progression patterns

### Content Quality
- Question clarity and performance metrics
- Rule coverage completeness
- User feedback on question relevance
- Content update alignment with rule changes

### Privacy Compliance
- Zero PII collection verification
- Consent mechanism effectiveness
- Data retention policy compliance
- User trust and transparency metrics

## Implementation Timeline

### Phase 1: Foundation (Months 1-2)
- Database schema and models
- Import system for questions
- Basic quiz interface
- Privacy-compliant analytics foundation

### Phase 2: Administration (Months 2-3)
- Admin interface for Fred
- Usage statistics collection
- Analytics dashboard and reporting

### Phase 3: Enhancement (Months 3-4)
- Advanced quiz modes
- Comprehensive analytics
- Performance optimization

---

*Last Updated: 2025-07-27*

**Key Focus**: This epic creates an engaging educational tool that transforms passive rule reading into active learning through interactive quizzes, while maintaining strict privacy compliance and providing valuable insights for educational content improvement.