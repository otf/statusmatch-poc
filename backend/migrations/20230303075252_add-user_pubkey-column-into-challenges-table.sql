TRUNCATE TABLE challenges;

ALTER TABLE challenges
DROP COLUMN authenticated;

ALTER TABLE challenges
ADD user_pubkey BYTEA;

ALTER TABLE challenges
ADD FOREIGN KEY (user_pubkey) 
REFERENCES users (pubkey);
