CREATE TABLE IF NOT EXISTS programs (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS program_statuses (
    program_id INT NOT NULL,
    level INT NOT NULL,
    name VARCHAR(255) NOT NULL,
    PRIMARY KEY (program_id, level),
    FOREIGN KEY (program_id) REFERENCES programs(id) ON DELETE CASCADE
);

CREATE TYPE report_result AS ENUM ('deny', 'challenge', 'match');

CREATE TABLE IF NOT EXISTS reports (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    from_program_id INT NOT NULL,
    from_status_level INT NOT NULL,
    to_program_id INT NOT NULL,
    to_status_level INT NOT NULL,
    result report_result NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (from_program_id) REFERENCES programs(id),
    FOREIGN KEY (from_program_id, from_status_level) REFERENCES program_statuses(program_id, level),
    FOREIGN KEY (to_program_id) REFERENCES programs(id),
    FOREIGN KEY (to_program_id, to_status_level) REFERENCES program_statuses(program_id, level)
);