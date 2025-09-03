-- Add migration script here
ALTER TABLE users 
ADD COLUMN IF NOT EXISTS phone VARCHAR(20);