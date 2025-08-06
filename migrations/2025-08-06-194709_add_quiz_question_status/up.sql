-- Add status field to quiz_questions table for question management
-- Status values: 'draft', 'active', 'archived'
ALTER TABLE quiz_questions ADD COLUMN status TEXT NOT NULL DEFAULT 'active';