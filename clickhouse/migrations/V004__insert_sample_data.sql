-- V004__insert_sample_data.sql

-- +migrate Up
-- 插入示例用户数据
INSERT INTO users (username, email, password_hash, status) VALUES
('john_doe', 'john@example.com', 'hashed_password_1', 'active'),
('jane_smith', 'jane@example.com', 'hashed_password_2', 'active'),
('bob_wilson', 'bob@example.com', 'hashed_password_3', 'inactive');

-- 插入示例用户资料数据
INSERT INTO user_profiles (user_id, first_name, last_name, bio) VALUES
(
    (SELECT id FROM users WHERE username = 'john_doe' LIMIT 1),
    'John',
    'Doe',
    'Software developer with 5 years of experience'
),
(
    (SELECT id FROM users WHERE username = 'jane_smith' LIMIT 1),
    'Jane',
    'Smith',
    'Product manager passionate about user experience'
),
(
    (SELECT id FROM users WHERE username = 'bob_wilson' LIMIT 1),
    'Bob',
    'Wilson',
    'DevOps engineer specializing in cloud infrastructure'
);

-- +migrate Down
-- 删除插入的示例数据
DELETE FROM user_profiles WHERE user_id IN (
    SELECT id FROM users WHERE username IN ('john_doe', 'jane_smith', 'bob_wilson')
);
DELETE FROM users WHERE username IN ('john_doe', 'jane_smith', 'bob_wilson');
