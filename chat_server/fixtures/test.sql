-- insert 3 workspaces
INSERT INTO workspaces (name, owner_id) VALUES ('acme', 0), ('foo', 0), ('bar', 0);

-- insert 5 users, all with hash password 'password'
INSERT INTO users (ws_id, email, fullname, password_hash) VALUES (1, 'startdusk@acme.org', 'Startdusk Shelby', '$argon2id$v=19$m=19456,t=2,p=1$cIMjf3nKudgSDzWs86Amew$3ocu+PUaA/cqcEdZAVJCOi9ul7FhNwJ07bawhwQ9Ikw')
, (1, 'john@acme.org', 'John Joy', '$argon2id$v=19$m=19456,t=2,p=1$cIMjf3nKudgSDzWs86Amew$3ocu+PUaA/cqcEdZAVJCOi9ul7FhNwJ07bawhwQ9Ikw')
, (1, 'ben@acme.org', 'Ben Chole', '$argon2id$v=19$m=19456,t=2,p=1$cIMjf3nKudgSDzWs86Amew$3ocu+PUaA/cqcEdZAVJCOi9ul7FhNwJ07bawhwQ9Ikw')
, (1, 'curry@acme.org', 'Curry Step', '$argon2id$v=19$m=19456,t=2,p=1$cIMjf3nKudgSDzWs86Amew$3ocu+PUaA/cqcEdZAVJCOi9ul7FhNwJ07bawhwQ9Ikw')
, (1, 'haden@acme.org', 'Haden Job', '$argon2id$v=19$m=19456,t=2,p=1$cIMjf3nKudgSDzWs86Amew$3ocu+PUaA/cqcEdZAVJCOi9ul7FhNwJ07bawhwQ9Ikw');

-- insert 4 chats
-- insert public/private channel
INSERT INTO chats (ws_id, name, type, members) VALUES (1, 'general', 'public_channel', '{1,2,3,4,5}')
    ,(1, 'private', 'private_channel', '{1,2,3}');

-- insert unnamed chat
INSERT INTO chats (ws_id, type, members) VALUES (1, 'single', '{1,2}'), (1, 'group', '{1,2,3}');
