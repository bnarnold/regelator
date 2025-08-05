-- Create admins table for secure authentication
CREATE TABLE admins (
    id TEXT PRIMARY KEY NOT NULL,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_login TIMESTAMP
);

-- Create index on username for login performance
CREATE INDEX idx_admins_username ON admins(username);

-- Create trigger to automatically update updated_at column
CREATE TRIGGER update_admins_updated_at
    AFTER UPDATE ON admins
    FOR EACH ROW
BEGIN
    UPDATE admins SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

-- Insert default admin user with password "rules123" 
-- (hashed with Argon2 - should be changed on first login in production)
INSERT INTO admins (id, username, password_hash, is_active) VALUES (
    '01234567-89ab-cdef-0123-456789abcdef',
    'admin',
    '$argon2id$v=19$m=65536,t=3,p=4$IWC85K7QpngzswEzXfjPig$PCYSP10aBPuJnbWXlXX36IR6HL57v2zwGx81thp7bG8',
    true
);