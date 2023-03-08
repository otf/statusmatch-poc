CREATE TABLE IF NOT EXISTS user_statuses (
    user_pubkey BYTEA NOT NULL,
    program_id INT NOT NULL,
    level INT NOT NULL,
    PRIMARY KEY (user_pubkey, program_id),
    FOREIGN KEY (user_pubkey) REFERENCES users(pubkey) ON DELETE CASCADE,
    FOREIGN KEY (program_id) REFERENCES programs(id) ON DELETE CASCADE
);

