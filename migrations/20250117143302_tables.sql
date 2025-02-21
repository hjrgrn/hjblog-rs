CREATE TABLE cities (
	id uuid NOT NULL,
    PRIMARY KEY (id),
	name VARCHAR(169) UNIQUE NOT NULL,
	latitude NUMERIC UNIQUE NOT NULL,
	longitude NUMERIC UNIQUE NOT NULL,
    timezone VARCHAR(200) NOT NULL
);

CREATE TABLE users (
    id uuid NOT NULL,
    PRIMARY KEY (id),
    username VARCHAR(60) UNIQUE NOT NULL,
    email VARCHAR(200) UNIQUE NOT NULL,
    city_id uuid,
    hash_pass TEXT NOT NULL,
    subscribed timestamptz NOT NULL DEFAULT (CURRENT_TIMESTAMP),
    is_admin BOOL NOT NULL DEFAULT(FALSE),
    is_two_factor_authentication_enabled BOOL NOT NULL DEFAULT(FALSE),
    secret_token VARCHAR(300),
    profile_pic VARCHAR(100) UNIQUE,
    FOREIGN KEY (city_id) REFERENCES cities (id)
);

CREATE TABLE posts (
    id uuid NOT NULL,
    PRIMARY KEY (id),
    title VARCHAR(60) NOT NULL,
    content VARCHAR(2000) NOT NULL,
    path_to_file VARCHAR(500),
    posted timestamptz NOT NULL DEFAULT (CURRENT_TIMESTAMP),
    author_id uuid NOT NULL,
    FOREIGN KEY (author_id) REFERENCES users (id)
);

CREATE TABLE comments (
    id uuid NOT NULL,
    PRIMARY KEY (id),
    post_id uuid NOT NULL,
    content VARCHAR(400) NOT NULL,
    author_id uuid NOT NULL,
    written timestamptz NOT NULL DEFAULT (CURRENT_TIMESTAMP),
    FOREIGN KEY (author_id) REFERENCES users (id),
    FOREIGN KEY (post_id) REFERENCES posts (id)
);

CREATE INDEX post_time_index ON posts (posted DESC);
