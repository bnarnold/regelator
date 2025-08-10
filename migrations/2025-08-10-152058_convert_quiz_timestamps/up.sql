-- Convert quiz table timestamp columns from TEXT to TIMESTAMP
-- SQLite doesn't have a native TIMESTAMP type, but Diesel maps this to TEXT with proper formatting

-- Quiz questions table - convert timestamps
ALTER TABLE quiz_questions RENAME TO quiz_questions_old;

CREATE TABLE quiz_questions (
    id TEXT NOT NULL PRIMARY KEY,
    rule_set_id TEXT NOT NULL,
    version_id TEXT NOT NULL,
    question_text TEXT NOT NULL,
    explanation TEXT NOT NULL,
    difficulty_level TEXT NOT NULL CHECK (difficulty_level IN ('beginner', 'intermediate', 'advanced')),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'draft')),
    FOREIGN KEY (rule_set_id) REFERENCES rule_sets (id),
    FOREIGN KEY (version_id) REFERENCES versions (id)
);

-- Copy data with timestamp conversion
INSERT INTO quiz_questions (id, rule_set_id, version_id, question_text, explanation, difficulty_level, created_at, updated_at, status)
SELECT id, rule_set_id, version_id, question_text, explanation, difficulty_level, 
       DATETIME(created_at) as created_at, 
       DATETIME(updated_at) as updated_at, 
       status
FROM quiz_questions_old;

DROP TABLE quiz_questions_old;

-- Quiz answers table - convert timestamps
ALTER TABLE quiz_answers RENAME TO quiz_answers_old;

CREATE TABLE quiz_answers (
    id TEXT NOT NULL PRIMARY KEY,
    question_id TEXT NOT NULL,
    answer_text TEXT NOT NULL,
    is_correct BOOLEAN NOT NULL DEFAULT FALSE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (question_id) REFERENCES quiz_questions (id) ON DELETE CASCADE
);

-- Copy data with timestamp conversion
INSERT INTO quiz_answers (id, question_id, answer_text, is_correct, sort_order, created_at, updated_at)
SELECT id, question_id, answer_text, is_correct, sort_order, 
       DATETIME(created_at) as created_at, 
       DATETIME(updated_at) as updated_at
FROM quiz_answers_old;

DROP TABLE quiz_answers_old;

-- Quiz attempts table - convert timestamp
ALTER TABLE quiz_attempts RENAME TO quiz_attempts_old;

CREATE TABLE quiz_attempts (
    id TEXT NOT NULL PRIMARY KEY,
    session_id TEXT NOT NULL,
    question_id TEXT NOT NULL,
    selected_answer_id TEXT,
    is_correct BOOLEAN,
    response_time_ms INTEGER,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (question_id) REFERENCES quiz_questions (id),
    FOREIGN KEY (selected_answer_id) REFERENCES quiz_answers (id)
);

-- Copy data with timestamp conversion
INSERT INTO quiz_attempts (id, session_id, question_id, selected_answer_id, is_correct, response_time_ms, created_at)
SELECT id, session_id, question_id, selected_answer_id, is_correct, response_time_ms, 
       DATETIME(created_at) as created_at
FROM quiz_attempts_old;

DROP TABLE quiz_attempts_old;

-- Quiz question rules table - convert timestamp
ALTER TABLE quiz_question_rules RENAME TO quiz_question_rules_old;

CREATE TABLE quiz_question_rules (
    id TEXT NOT NULL PRIMARY KEY,
    question_id TEXT NOT NULL,
    rule_id TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (question_id) REFERENCES quiz_questions (id) ON DELETE CASCADE,
    FOREIGN KEY (rule_id) REFERENCES rule_content (id),
    UNIQUE(question_id, rule_id)
);

-- Copy data with timestamp conversion
INSERT INTO quiz_question_rules (id, question_id, rule_id, created_at)
SELECT id, question_id, rule_id, 
       DATETIME(created_at) as created_at
FROM quiz_question_rules_old;

DROP TABLE quiz_question_rules_old;

-- Recreate triggers for updated_at timestamps
CREATE TRIGGER update_quiz_questions_updated_at
    AFTER UPDATE ON quiz_questions
    FOR EACH ROW
    BEGIN
        UPDATE quiz_questions SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;

CREATE TRIGGER update_quiz_answers_updated_at
    AFTER UPDATE ON quiz_answers
    FOR EACH ROW
    BEGIN
        UPDATE quiz_answers SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;

-- Recreate indexes for performance
CREATE INDEX idx_quiz_questions_rule_set ON quiz_questions(rule_set_id);
CREATE INDEX idx_quiz_questions_version ON quiz_questions(version_id);
CREATE INDEX idx_quiz_questions_difficulty ON quiz_questions(difficulty_level);
CREATE INDEX idx_quiz_answers_question ON quiz_answers(question_id);
CREATE INDEX idx_quiz_question_rules_question ON quiz_question_rules(question_id);
CREATE INDEX idx_quiz_question_rules_rule ON quiz_question_rules(rule_id);
CREATE INDEX idx_quiz_attempts_session ON quiz_attempts(session_id);
CREATE INDEX idx_quiz_attempts_question ON quiz_attempts(question_id);
CREATE INDEX idx_quiz_attempts_created_at ON quiz_attempts(created_at);