# Project Plan: Rules Quiz System

This document tracks development of an interactive quiz system to help players learn Ultimate Frisbee rules through practice questions.

## Phase Status: 📋 PLANNED

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

### Story 4: Quiz Progress & Statistics 🎯
**Goal:** Track user quiz performance and show learning progress (session-based)

**Acceptance Criteria:**
- [ ] Record quiz attempts with anonymous session IDs
- [ ] Show session statistics (questions answered, correct percentage)
- [ ] Display streak tracking (consecutive correct answers)
- [ ] Add difficulty-based filtering for questions
- [ ] Show rule area performance for current session
- [ ] Create session-based progress display
- [ ] Add "Review Missed Questions" feature for current session
- [ ] Support quiz categories by rule section
- [ ] Clear session data option for privacy

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

**Benefits:**
- Immediate learning insights
- Motivation through progress tracking
- Identifies knowledge gaps
- Maintains user privacy

### Story 5: Quiz Administration Interface (Fred) 🎯
**Goal:** Create admin interface for Fred to manage quiz content and monitor usage

**Acceptance Criteria:**
- [ ] Create admin login system for Fred
- [ ] Build quiz question management interface
- [ ] Add question creation/editing forms
- [ ] Implement question review and approval workflow
- [ ] Create bulk question import interface
- [ ] Add question quality metrics (answer distribution, difficulty calibration)
- [ ] Enable question activation/deactivation
- [ ] Support question categorization by rule section
- [ ] Add rule reference validation
- [ ] Create question preview functionality

**Admin Interface Features:**
- Question library management
- Difficulty level assignment
- Rule reference linking
- Question performance metrics
- Content quality control
- Bulk operations for question management

**Benefits:**
- Centralized content management
- Quality control for educational content
- Efficient question maintenance
- Rule alignment verification

### Story 6: Usage Statistics Collection (GDPR Compliant) 🎯
**Goal:** Collect anonymous usage data for educational insights while maintaining privacy

**Acceptance Criteria:**
- [ ] Implement privacy-first analytics system
- [ ] Collect anonymous quiz usage patterns
- [ ] Track question performance metrics (difficulty, success rates)
- [ ] Record rule section coverage and engagement
- [ ] Monitor quiz completion rates
- [ ] Gather timing data for question difficulty calibration
- [ ] Implement data retention policies (auto-deletion)
- [ ] Add explicit consent mechanisms
- [ ] Create privacy policy and data handling documentation
- [ ] Ensure no personally identifiable information (PII) collection

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

### Story 7: Usage Statistics Analysis & Reporting (Fred) 🎯
**Goal:** Provide Fred with insights for educational planning and content improvement

**Acceptance Criteria:**
- [ ] Create analytics dashboard for aggregated quiz data
- [ ] Generate reports on question performance and difficulty
- [ ] Show rule section engagement patterns
- [ ] Identify content gaps and improvement opportunities
- [ ] Track learning effectiveness metrics
- [ ] Create exportable reports for educational planning
- [ ] Add trend analysis over time
- [ ] Implement question recommendation system based on data
- [ ] Generate content quality scores
- [ ] Provide insights for rule clarification needs

**Analytics Dashboard Features:**
- **Question Performance**: Success rates, difficulty calibration, response times
- **Rule Coverage**: Engagement by rule section, knowledge gap identification
- **Learning Patterns**: Progression through difficulty levels, common misconceptions
- **Content Quality**: Question clarity metrics, answer distribution analysis
- **Educational Impact**: Learning effectiveness indicators, rule comprehension improvement

**Reporting Capabilities:**
- Monthly educational impact reports
- Question performance summaries
- Rule section analysis
- Content recommendations
- Export to PDF/CSV for external use

**Benefits:**
- Data-driven educational content decisions
- Improved question quality through performance feedback
- Targeted educational material development
- Enhanced rule learning effectiveness

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