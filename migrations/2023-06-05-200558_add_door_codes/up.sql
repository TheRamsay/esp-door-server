CREATE TABLE door_code (
    code varchar(36) PRIMARY KEY,
    door_id int not null REFERENCES door(id),
    created_at timestamptz NOT NULL,
    expires_at timestamptz,
    creator_id int not null REFERENCES user_profile(id)
)