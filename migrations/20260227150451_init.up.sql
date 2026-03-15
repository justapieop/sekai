CREATE TABLE IF NOT EXISTS users
(
    id         UUID PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    bio        TEXT        NOT NULL DEFAULT '',
    is_admin   BOOLEAN     NOT NULL DEFAULT false,
    points     BIGINT      NOT NULL DEFAULT 0,
    email      TEXT        NOT NULL,
    name       TEXT        NOT NULL,
    avatar_url TEXT        NOT NULL
);

CREATE UNIQUE INDEX user_idx ON users (id);

CREATE TABLE IF NOT EXISTS file_metadata
(
    id          NUMERIC(39, 0) PRIMARY KEY,
    uploaded_by UUID        NOT NULL REFERENCES users (id) ON UPDATE CASCADE ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX file_metadata_idx ON file_metadata (id);

CREATE TABLE IF NOT EXISTS posts
(
    id         NUMERIC(39, 0) PRIMARY KEY,
    author     UUID        NOT NULL REFERENCES users (id) ON UPDATE CASCADE ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    content    TEXT        NOT NULL DEFAULT '',
    likes      BIGINT      NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX post_idx ON posts (id DESC, created_at DESC);

CREATE TABLE post_attachments
(
    post_id       NUMERIC(39, 0) REFERENCES posts (id) ON UPDATE CASCADE ON DELETE CASCADE,
    attachment_id NUMERIC(39, 0) REFERENCES file_metadata (id) ON UPDATE CASCADE ON DELETE CASCADE,
    PRIMARY KEY (post_id, attachment_id)
);

CREATE UNIQUE INDEX post_attachment_idx ON post_attachments (post_id, attachment_id);

CREATE TABLE IF NOT EXISTS pin_types
(
    id   NUMERIC(39, 0) PRIMARY KEY,
    name VARCHAR(256) UNIQUE NOT NULL,
    icon VARCHAR(1)          NOT NULL
);

CREATE UNIQUE INDEX pin_type_idx ON pin_types (id);

CREATE TABLE IF NOT EXISTS pins
(
    id           NUMERIC(39, 0) PRIMARY KEY,
    name         VARCHAR(256)   NOT NULL DEFAULT '',
    type_id      NUMERIC(39, 0) NOT NULL REFERENCES pin_types (id) ON UPDATE CASCADE ON DELETE CASCADE,
    lat          real           NOT NULL DEFAULT 0,
    long         real           NOT NULL DEFAULT 0,
    created_at   TIMESTAMPTZ    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at   TIMESTAMPTZ    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    address      TEXT           NOT NULL DEFAULT '',
    is_sponsored BOOLEAN        NOT NULL DEFAULT false,
    terms        TEXT           NOT NULL DEFAULT '',
    instruction  TEXT           NOT NULL DEFAULT '',
    opening      INT[]          NOT NULL DEFAULT ARRAY [0, 0],
    closing      INT[]          NOT NULL DEFAULT ARRAY [0, 0]
);

CREATE UNIQUE INDEX pin_idx ON pins (id);

CREATE TABLE IF NOT EXISTS challenges
(
    id          NUMERIC(39, 0) PRIMARY KEY,
    title       VARCHAR(256)   NOT NULL,
    description TEXT           NOT NULL,
    instruction TEXT           NOT NULL,
    created_at  TIMESTAMPTZ    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMPTZ    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    starts_at   TIMESTAMPTZ    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ends_at     TIMESTAMPTZ    NOT NULL,
    points      INT            NOT NULL CHECK (points > 0),
    duration    INT            NOT NULL,
    cover_image NUMERIC(39, 0) NOT NULL REFERENCES file_metadata (id),
    created_by  UUID           NOT NULL REFERENCES users (id) ON UPDATE CASCADE ON DELETE CASCADE
);

CREATE UNIQUE INDEX challenge_idx ON challenges (id);

CREATE TABLE IF NOT EXISTS user_challenges
(
    user_id      UUID REFERENCES users (id) ON UPDATE CASCADE ON DELETE CASCADE,
    challenge_id NUMERIC(39, 0) REFERENCES challenges (id) ON UPDATE CASCADE ON DELETE CASCADE,
    joined_at    TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    finished     BOOLEAN     NOT NULL DEFAULT false,
    finished_at  TIMESTAMPTZ,
    PRIMARY KEY (user_id, challenge_id)
);

CREATE UNIQUE INDEX user_challenge_idx ON user_challenges (user_id, challenge_id);

CREATE TABLE IF NOT EXISTS user_challenge_uploads
(
    user_id       UUID REFERENCES users (id) ON UPDATE CASCADE ON DELETE CASCADE,
    challenge_id  NUMERIC(39, 0) REFERENCES challenges (id) ON UPDATE CASCADE ON DELETE CASCADE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    attachment_id NUMERIC(39, 0) REFERENCES file_metadata (id) ON UPDATE CASCADE ON DELETE CASCADE,
    PRIMARY KEY (user_id, challenge_id, attachment_id)
);

CREATE UNIQUE INDEX user_challenge_upload_idx ON user_challenge_uploads (user_id, challenge_id, attachment_id);

CREATE TABLE IF NOT EXISTS post_comments
(
    id            NUMERIC(39, 0) PRIMARY KEY,
    user_id       UUID           NOT NULL REFERENCES users (id) ON UPDATE CASCADE ON DELETE CASCADE,
    post_id       NUMERIC(39, 0) NOT NULL REFERENCES posts (id) ON UPDATE CASCADE ON DELETE CASCADE,
    reply_to      NUMERIC(39, 0) NULL,
    content       TEXT           NOT NULL,
    attachment_id NUMERIC(39, 0) NULL,
    created_at    TIMESTAMPTZ    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at    TIMESTAMPTZ    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_reply_to FOREIGN KEY (reply_to) REFERENCES post_comments (id) ON UPDATE CASCADE ON DELETE CASCADE,
    CONSTRAINT fk_attachment_id FOREIGN KEY (attachment_id) REFERENCES file_metadata (id) ON UPDATE CASCADE ON DELETE SET NULL
);

CREATE UNIQUE INDEX post_comment_idx ON post_comments (id);