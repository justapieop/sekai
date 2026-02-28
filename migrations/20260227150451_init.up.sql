CREATE TABLE IF NOT EXISTS file_metadata (
    id NUMERIC(39, 0) PRIMARY KEY,
    file_name VARCHAR(256) NOT NULL
);

CREATE UNIQUE INDEX file_metadata_idx ON file_metadata(id);

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    bio TEXT NOT NULL DEFAULT '',
    is_admin BOOLEAN NOT NULL DEFAULT false,
    points BIGINT NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX user_idx ON users(id);

CREATE TABLE IF NOT EXISTS posts (
    id NUMERIC(39, 0) PRIMARY KEY,
    author UUID NOT NULL REFERENCES users(id) ON UPDATE CASCADE ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    content TEXT NOT NULL DEFAULT '',
    likes BIGINT NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX post_idx ON posts(id);

CREATE TABLE post_attachments (
    post_id NUMERIC(39, 0) REFERENCES posts(id) ON UPDATE CASCADE ON DELETE CASCADE,
    attachment_id NUMERIC(39, 0) REFERENCES file_metadata(id) ON UPDATE CASCADE ON DELETE CASCADE,
    PRIMARY KEY (post_id, attachment_id)
);

CREATE UNIQUE INDEX post_attachment_idx ON post_attachments(post_id, attachment_id);

CREATE TABLE IF NOT EXISTS pin_types (
    id NUMERIC(39, 0) PRIMARY KEY,
    name VARCHAR(256) UNIQUE NOT NULL,
    icon BYTEA NOT NULL
);

CREATE UNIQUE INDEX pin_type_idx ON pin_types(id);

CREATE TABLE IF NOT EXISTS pins (
    id NUMERIC(39, 0) PRIMARY KEY,
    name VARCHAR(256) NOT NULL, 
    type_id NUMERIC(39, 0) NOT NULL REFERENCES pin_types(id) ON UPDATE CASCADE ON DELETE CASCADE,
    lat real NOT NULL,
    long real NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    address TEXT NOT NULL,
    is_sponsored BOOLEAN NOT NULL DEFAULT false,
    terms TEXT NOT NULL DEFAULT '',
    opening INT[] NOT NULL DEFAULT ARRAY[0, 0],
    closing INT[] NOT NULL DEFAULT ARRAY[0, 0]
);

CREATE UNIQUE INDEX pin_idx ON pins(id);

CREATE TABLE IF NOT EXISTS challenges (
    id NUMERIC(39, 0) PRIMARY KEY,
    title VARCHAR(256) NOT NULL,
    description TEXT NOT NULL,
    instruction TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ends_at TIMESTAMPTZ,
    points INT NOT NULL CHECK(points > 0)
);

CREATE UNIQUE INDEX challenge_idx ON challenges (id);

CREATE TABLE IF NOT EXISTS user_challenges (
    user_id UUID REFERENCES users(id) ON UPDATE CASCADE ON DELETE CASCADE,
    challenge_id NUMERIC(39, 0) REFERENCES challenges(id) ON UPDATE CASCADE ON DELETE CASCADE,
    PRIMARY KEY(user_id, challenge_id)
);

CREATE UNIQUE INDEX user_challenge_idx ON user_challenges(user_id, challenge_id);