DROP VIEW IF EXISTS author_stats, album_stats, track_stats CASCADE;
DROP PROCEDURE IF EXISTS add_album(track_input[], album_input, author_input, INOUT INT);
DROP PROCEDURE IF EXISTS add_track(INT, TEXT, INT, TEXT);
DROP PROCEDURE IF EXISTS record_listen(INT);
DROP TABLE IF EXISTS listenings, users, track_albums, tracks, albums, authors CASCADE;
DROP TYPE IF EXISTS track_input, album_input, author_input CASCADE;


-- =================================================================
-- 1. SCHEMAS 
-- =================================================================

CREATE TABLE authors (
  id        INT     PRIMARY KEY,
  title     TEXT    NOT NULL,
  img       TEXT
);

CREATE TABLE albums (
  id          INT     PRIMARY KEY,
  title       TEXT    NOT NULL,
  img         TEXT,
  author_id   INT     NOT NULL REFERENCES authors(id) ON DELETE CASCADE
);

CREATE TABLE tracks (
  id        INT     PRIMARY KEY,
  title     TEXT    NOT NULL,
  duration  INT     NOT NULL, -- Duration in ms
  img       TEXT,
  author_id INT     NOT NULL REFERENCES authors(id) ON DELETE CASCADE
);

CREATE TABLE track_albums (
  track_id    INT NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
  album_id    INT NOT NULL REFERENCES albums(id) ON DELETE CASCADE,
  PRIMARY KEY (track_id, album_id)
);

CREATE TABLE listenings (
  id          BIGSERIAL PRIMARY KEY,
  track_id    INT NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
  listened_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX ON listenings (track_id);
CREATE INDEX ON track_albums (album_id);


-- =================================================================
-- 2. TYPES (Unchanged)
-- =================================================================

CREATE TYPE author_input AS (
  id        INT,
  title     TEXT,
  img       TEXT
);

CREATE TYPE album_input AS (
  id        INT,
  title     TEXT,
  img       TEXT,
  author_id INT
);

CREATE TYPE track_input AS (
  id        INT,
  title     TEXT,
  duration  INT
);


-- =================================================================
-- 3. PROCEDURES
-- =================================================================


CREATE OR REPLACE PROCEDURE add_album(
  _tracks       track_input[],
  _album        album_input,
  _author       author_input
)
LANGUAGE plpgsql
AS $$
DECLARE
  t track_input;
BEGIN
  INSERT INTO authors (id, title, img)
    VALUES (_author.id, _author.title, _author.img)
    ON CONFLICT (id) DO NOTHING;

  INSERT INTO albums (id, title, img, author_id)
    VALUES (_album.id, _album.title, _album.img, _author.id)
    ON CONFLICT (id) DO NOTHING;

  FOREACH t IN ARRAY _tracks LOOP
    INSERT INTO tracks(id, title, duration, author_id)
      VALUES (t.id, t.title, t.duration, _author.id)
      ON CONFLICT (id) DO UPDATE SET img = NULL;

    INSERT INTO track_albums(track_id, album_id)
      VALUES (t.id, _album.id)
      ON CONFLICT (track_id, album_id) DO NOTHING;
  END LOOP;
END;
$$;


CREATE OR REPLACE PROCEDURE add_track(
  _track      track_input,
  _track_img  TEXT,
  _author     author_input
)
LANGUAGE plpgsql
AS $$
BEGIN
INSERT INTO authors (id, title, img)
VALUES (_author.id, _author.title, _author.img)
    ON CONFLICT (id) DO NOTHING;

INSERT INTO tracks (id, title, duration, img, author_id)
VALUES (_track.id, _track.title, _track.duration, _track_img, _author.id)
    ON CONFLICT (id) DO NOTHING;
END;
$$;


CREATE OR REPLACE PROCEDURE record_listen(
  _track_id INT
)
LANGUAGE plpgsql
AS $$
BEGIN
  INSERT INTO listenings (track_id)
  VALUES (_track_id);
END;
$$;


-- =================================================================
-- 4. LISTENING STATS - VIEWS
-- =================================================================

CREATE OR REPLACE VIEW track_stats AS
SELECT
  t.id AS track_id,
  t.title,
  t.duration,
  COUNT(l.id) AS listen_count
FROM tracks t
LEFT JOIN listenings l ON t.id = l.track_id
GROUP BY t.id;

CREATE OR REPLACE VIEW album_stats AS
SELECT
  a.id AS album_id,
  a.title,
  au.id as author_id,
  au.title AS author_title,
  COUNT(ta.track_id) AS track_count,
  SUM(ts.listen_count)::bigint AS total_listens
FROM albums a
JOIN authors au ON a.author_id = au.id
LEFT JOIN track_albums ta ON a.id = ta.album_id
LEFT JOIN track_stats ts ON ta.track_id = ts.track_id
GROUP BY a.id, au.id;

CREATE OR REPLACE VIEW author_stats AS
SELECT
  au.id AS author_id,
  au.title,
  COUNT(DISTINCT als.album_id) AS album_count,
  SUM(als.total_listens)::bigint AS total_listens
FROM authors au
LEFT JOIN album_stats als ON au.id = als.author_id
GROUP BY au.id;