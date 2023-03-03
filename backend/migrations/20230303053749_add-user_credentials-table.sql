CREATE TABLE IF NOT EXISTS user_credentials (
    user_pubkey BYTEA NOT NULL,
    program_id INT NOT NULL,
    username VARCHAR(255) NOT NULL,
    password VARCHAR(255) NOT NULL,
    PRIMARY KEY (user_pubkey, program_id),
    FOREIGN KEY (user_pubkey) REFERENCES users(pubkey) ON DELETE CASCADE,
    FOREIGN KEY (program_id) REFERENCES programs(id) ON DELETE CASCADE
);
