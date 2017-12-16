CREATE TABLE person (
    id SERIAL PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    password CHAR(64) NOT NULL
);

INSERT INTO person (name, password) VALUES ('system', '6ee4a469cd4e91053847f5d3fcb61dbcc91e8f0ef10be7748da4c4a1ba382d17');

CREATE TABLE article (
    id SERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    body TEXT,
    author INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE INDEX article_updated_at ON article (updated_at);
