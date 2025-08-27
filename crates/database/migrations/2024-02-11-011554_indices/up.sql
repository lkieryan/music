-- Your SQL goes here
CREATE UNIQUE INDEX path_uq ON tracks(path);

CREATE UNIQUE INDEX sanitized_artist_name_uq ON artists(sanitized_artist_name);

CREATE UNIQUE INDEX album_name_uq ON albums(album_name);

CREATE UNIQUE INDEX genre_name_uq ON genres(genre_name);

CREATE UNIQUE INDEX artist_bridge_uq ON artist_bridge(track, artist);

CREATE UNIQUE INDEX album_bridge_uq ON album_bridge(track, album);

CREATE UNIQUE INDEX genre_bridge_uq ON genre_bridge(track, genre);