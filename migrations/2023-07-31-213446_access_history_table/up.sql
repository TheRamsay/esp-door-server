CREATE TABLE access_history (
    id SERIAL PRIMARY KEY,
    door_id INTEGER NOT NULL REFERENCES door(id),
    user_profile_id INTEGER NOT NULL REFERENCES user_profile(id),
    access_timestamp timestamptz NOT NULL
)