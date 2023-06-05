CREATE TABLE user_profile (
  id SERIAL PRIMARY KEY,
  discord_id INT NOT NULL,
  username VARCHAR NOT NULL,
  avatar_url VARCHAR
);

CREATE TABLE door (
  id SERIAL PRIMARY KEY,
  about VARCHAR
);

CREATE TABLE door_permission (
    door_id INTEGER REFERENCES door(id),
    user_profile_id INTEGER REFERENCES user_profile(id),
    edit_permission BOOLEAN NOT NULL,
    open_permission BOOLEAN NOT NULL,
    PRIMARY KEY(door_id, user_profile_id)
);