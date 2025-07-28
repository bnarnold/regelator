-- Quiz questions table
CREATE TABLE quiz_questions (
    id TEXT NOT NULL PRIMARY KEY,
    rule_set_id TEXT NOT NULL,
    version_id TEXT NOT NULL,
    question_text TEXT NOT NULL,
    explanation TEXT NOT NULL,
    difficulty_level TEXT NOT NULL CHECK (difficulty_level IN ('beginner', 'intermediate', 'advanced')),
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (rule_set_id) REFERENCES rule_sets (id),
    FOREIGN KEY (version_id) REFERENCES versions (id)
);

-- Quiz answer options table
CREATE TABLE quiz_answers (
    id TEXT NOT NULL PRIMARY KEY,
    question_id TEXT NOT NULL,
    answer_text TEXT NOT NULL,
    is_correct BOOLEAN NOT NULL DEFAULT FALSE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (question_id) REFERENCES quiz_questions (id) ON DELETE CASCADE
);

-- Many-to-many relationship between questions and rules
CREATE TABLE quiz_question_rules (
    id TEXT NOT NULL PRIMARY KEY,
    question_id TEXT NOT NULL,
    rule_id TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (question_id) REFERENCES quiz_questions (id) ON DELETE CASCADE,
    FOREIGN KEY (rule_id) REFERENCES rule_content (id),
    UNIQUE(question_id, rule_id)
);

-- Quiz attempts table (anonymous tracking)
CREATE TABLE quiz_attempts (
    id TEXT NOT NULL PRIMARY KEY,
    session_id TEXT NOT NULL,
    question_id TEXT NOT NULL,
    selected_answer_id TEXT,
    is_correct BOOLEAN,
    response_time_ms INTEGER,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (question_id) REFERENCES quiz_questions (id),
    FOREIGN KEY (selected_answer_id) REFERENCES quiz_answers (id)
);

-- Triggers for updating updated_at timestamps
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

-- Indexes for performance
CREATE INDEX idx_quiz_questions_rule_set ON quiz_questions(rule_set_id);
CREATE INDEX idx_quiz_questions_version ON quiz_questions(version_id);
CREATE INDEX idx_quiz_questions_difficulty ON quiz_questions(difficulty_level);
CREATE INDEX idx_quiz_answers_question ON quiz_answers(question_id);
CREATE INDEX idx_quiz_question_rules_question ON quiz_question_rules(question_id);
CREATE INDEX idx_quiz_question_rules_rule ON quiz_question_rules(rule_id);
CREATE INDEX idx_quiz_attempts_session ON quiz_attempts(session_id);
CREATE INDEX idx_quiz_attempts_question ON quiz_attempts(question_id);
CREATE INDEX idx_quiz_attempts_created_at ON quiz_attempts(created_at);