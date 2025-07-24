-- This migration cannot be easily reversed since we're fixing a schema design issue
-- The original schema with nullable primary keys was incorrect
-- If rollback is absolutely necessary, would need to recreate tables with nullable PKs
-- But this should not be needed in practice

-- Placeholder for required down.sql file
SELECT 'Cannot rollback primary key nullability fix - this was a schema correction' as message;