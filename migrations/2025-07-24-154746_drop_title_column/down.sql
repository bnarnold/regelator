-- Add back the title column (for rollback if needed)
ALTER TABLE rule_content ADD COLUMN title TEXT;