-- ============================================
-- Rust Forgotten Server (RFS) - PostgreSQL schema
-- Ported from TFS MySQL schema
-- ============================================

-- ACCOUNTS
CREATE TABLE IF NOT EXISTS accounts (
  id SERIAL PRIMARY KEY,
  name VARCHAR(32) NOT NULL UNIQUE,
  password CHAR(40) NOT NULL,
  secret CHAR(16),
  type INT NOT NULL DEFAULT 1,
  premium_ends_at INT NOT NULL DEFAULT 0,
  email VARCHAR(255) NOT NULL DEFAULT '',
  creation INT NOT NULL DEFAULT 0
);

-- PLAYERS
CREATE TABLE IF NOT EXISTS players (
  id SERIAL PRIMARY KEY,
  name VARCHAR(255) NOT NULL UNIQUE,
  group_id INT NOT NULL DEFAULT 1,
  account_id INT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
  level INT NOT NULL DEFAULT 1,
  vocation INT NOT NULL DEFAULT 0,
  health INT NOT NULL DEFAULT 150,
  healthmax INT NOT NULL DEFAULT 150,
  experience BIGINT NOT NULL DEFAULT 0,
  lookbody INT NOT NULL DEFAULT 0,
  lookfeet INT NOT NULL DEFAULT 0,
  lookhead INT NOT NULL DEFAULT 0,
  looklegs INT NOT NULL DEFAULT 0,
  looktype INT NOT NULL DEFAULT 136,
  lookaddons INT NOT NULL DEFAULT 0,
  lookmount INT NOT NULL DEFAULT 0,
  lookmounthead INT NOT NULL DEFAULT 0,
  lookmountbody INT NOT NULL DEFAULT 0,
  lookmountlegs INT NOT NULL DEFAULT 0,
  lookmountfeet INT NOT NULL DEFAULT 0,
  currentmount SMALLINT NOT NULL DEFAULT 0,
  randomizemount SMALLINT NOT NULL DEFAULT 0,
  direction SMALLINT NOT NULL DEFAULT 2,
  maglevel INT NOT NULL DEFAULT 0,
  mana INT NOT NULL DEFAULT 0,
  manamax INT NOT NULL DEFAULT 0,
  manaspent BIGINT NOT NULL DEFAULT 0,
  soul INT NOT NULL DEFAULT 0,
  town_id INT NOT NULL DEFAULT 1,
  posx INT NOT NULL DEFAULT 0,
  posy INT NOT NULL DEFAULT 0,
  posz INT NOT NULL DEFAULT 0,
  conditions BYTEA,
  cap INT NOT NULL DEFAULT 400,
  sex INT NOT NULL DEFAULT 0,
  lastlogin BIGINT NOT NULL DEFAULT 0,
  lastip BYTEA NOT NULL DEFAULT E'\\x00000000000000000000000000000000',
  save SMALLINT NOT NULL DEFAULT 1,
  skull SMALLINT NOT NULL DEFAULT 0,
  skulltime BIGINT NOT NULL DEFAULT 0,
  lastlogout BIGINT NOT NULL DEFAULT 0,
  blessings SMALLINT NOT NULL DEFAULT 0,
  onlinetime BIGINT NOT NULL DEFAULT 0,
  deletion BIGINT NOT NULL DEFAULT 0,
  balance BIGINT NOT NULL DEFAULT 0,
  offlinetraining_time SMALLINT NOT NULL DEFAULT 43200,
  offlinetraining_skill INT NOT NULL DEFAULT -1,
  stamina SMALLINT NOT NULL DEFAULT 2520,
  skill_fist INT NOT NULL DEFAULT 10,
  skill_fist_tries BIGINT NOT NULL DEFAULT 0,
  skill_club INT NOT NULL DEFAULT 10,
  skill_club_tries BIGINT NOT NULL DEFAULT 0,
  skill_sword INT NOT NULL DEFAULT 10,
  skill_sword_tries BIGINT NOT NULL DEFAULT 0,
  skill_axe INT NOT NULL DEFAULT 10,
  skill_axe_tries BIGINT NOT NULL DEFAULT 0,
  skill_dist INT NOT NULL DEFAULT 10,
  skill_dist_tries BIGINT NOT NULL DEFAULT 0,
  skill_shielding INT NOT NULL DEFAULT 10,
  skill_shielding_tries BIGINT NOT NULL DEFAULT 0,
  skill_fishing INT NOT NULL DEFAULT 10,
  skill_fishing_tries BIGINT NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_players_vocation ON players(vocation);

-- ACCOUNT BANS
CREATE TABLE IF NOT EXISTS account_bans (
  account_id INT PRIMARY KEY REFERENCES accounts(id) ON DELETE CASCADE ON UPDATE CASCADE,
  reason VARCHAR(255) NOT NULL,
  banned_at BIGINT NOT NULL,
  expires_at BIGINT NOT NULL,
  banned_by INT NOT NULL REFERENCES players(id) ON DELETE CASCADE ON UPDATE CASCADE
);

-- ACCOUNT BAN HISTORY
CREATE TABLE IF NOT EXISTS account_ban_history (
  id SERIAL PRIMARY KEY,
  account_id INT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE ON UPDATE CASCADE,
  reason VARCHAR(255) NOT NULL,
  banned_at BIGINT NOT NULL,
  expired_at BIGINT NOT NULL,
  banned_by INT NOT NULL REFERENCES players(id) ON DELETE CASCADE ON UPDATE CASCADE
);

-- ACCOUNT STORAGE
CREATE TABLE IF NOT EXISTS account_storage (
  account_id INT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
  key INT NOT NULL,
  value INT NOT NULL,
  PRIMARY KEY (account_id, key)
);

-- IP BANS
CREATE TABLE IF NOT EXISTS ip_bans (
  ip BYTEA PRIMARY KEY,
  reason VARCHAR(255) NOT NULL,
  banned_at BIGINT NOT NULL,
  expires_at BIGINT NOT NULL,
  banned_by INT NOT NULL REFERENCES players(id) ON DELETE CASCADE ON UPDATE CASCADE
);

-- PLAYER NAMELOCKS
CREATE TABLE IF NOT EXISTS player_namelocks (
  player_id INT PRIMARY KEY REFERENCES players(id) ON DELETE CASCADE ON UPDATE CASCADE,
  reason VARCHAR(255) NOT NULL,
  namelocked_at BIGINT NOT NULL,
  namelocked_by INT NOT NULL REFERENCES players(id) ON DELETE CASCADE ON UPDATE CASCADE
);

-- ACCOUNT VIPLIST
CREATE TABLE IF NOT EXISTS account_viplist (
  account_id INT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
  player_id INT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
  description VARCHAR(128) NOT NULL DEFAULT '',
  icon SMALLINT NOT NULL DEFAULT 0,
  notify SMALLINT NOT NULL DEFAULT 0,
  UNIQUE (account_id, player_id)
);

-- GUILDS
CREATE TABLE IF NOT EXISTS guilds (
  id SERIAL PRIMARY KEY,
  name VARCHAR(255) NOT NULL UNIQUE,
  ownerid INT NOT NULL UNIQUE REFERENCES players(id) ON DELETE CASCADE,
  creationdata INT NOT NULL,
  motd VARCHAR(255) NOT NULL DEFAULT ''
);

-- GUILD INVITES
CREATE TABLE IF NOT EXISTS guild_invites (
  player_id INT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
  guild_id INT NOT NULL REFERENCES guilds(id) ON DELETE CASCADE,
  PRIMARY KEY (player_id, guild_id)
);

-- GUILD RANKS
CREATE TABLE IF NOT EXISTS guild_ranks (
  id SERIAL PRIMARY KEY,
  guild_id INT NOT NULL REFERENCES guilds(id) ON DELETE CASCADE,
  name VARCHAR(255) NOT NULL,
  level INT NOT NULL
);

-- GUILD MEMBERSHIP
CREATE TABLE IF NOT EXISTS guild_membership (
  player_id INT PRIMARY KEY REFERENCES players(id) ON DELETE CASCADE ON UPDATE CASCADE,
  guild_id INT NOT NULL REFERENCES guilds(id) ON DELETE CASCADE ON UPDATE CASCADE,
  rank_id INT NOT NULL REFERENCES guild_ranks(id) ON DELETE CASCADE ON UPDATE CASCADE,
  nick VARCHAR(15) NOT NULL DEFAULT ''
);

-- GUILD WARS
CREATE TABLE IF NOT EXISTS guild_wars (
  id SERIAL PRIMARY KEY,
  guild1 INT NOT NULL DEFAULT 0,
  guild2 INT NOT NULL DEFAULT 0,
  name1 VARCHAR(255) NOT NULL,
  name2 VARCHAR(255) NOT NULL,
  status SMALLINT NOT NULL DEFAULT 0,
  started BIGINT NOT NULL DEFAULT 0,
  ended BIGINT NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_guild_wars_guild1 ON guild_wars(guild1);
CREATE INDEX IF NOT EXISTS idx_guild_wars_guild2 ON guild_wars(guild2);

-- GUILD WAR KILLS
CREATE TABLE IF NOT EXISTS guildwar_kills (
  id SERIAL PRIMARY KEY,
  killer VARCHAR(50) NOT NULL,
  target VARCHAR(50) NOT NULL,
  killerguild INT NOT NULL DEFAULT 0,
  targetguild INT NOT NULL DEFAULT 0,
  warid INT NOT NULL REFERENCES guild_wars(id) ON DELETE CASCADE,
  time BIGINT NOT NULL
);

-- HOUSES
CREATE TABLE IF NOT EXISTS houses (
  id SERIAL PRIMARY KEY,
  owner INT NOT NULL,
  paid INT NOT NULL DEFAULT 0,
  warnings INT NOT NULL DEFAULT 0,
  name VARCHAR(255) NOT NULL,
  rent INT NOT NULL DEFAULT 0,
  town_id INT NOT NULL DEFAULT 0,
  bid INT NOT NULL DEFAULT 0,
  bid_end INT NOT NULL DEFAULT 0,
  last_bid INT NOT NULL DEFAULT 0,
  highest_bidder INT NOT NULL DEFAULT 0,
  size INT NOT NULL DEFAULT 0,
  beds INT NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_houses_owner ON houses(owner);
CREATE INDEX IF NOT EXISTS idx_houses_town_id ON houses(town_id);

-- HOUSE LISTS
CREATE TABLE IF NOT EXISTS house_lists (
  house_id INT NOT NULL REFERENCES houses(id) ON DELETE CASCADE,
  listid INT NOT NULL,
  list TEXT NOT NULL
);

-- MARKET HISTORY
CREATE TABLE IF NOT EXISTS market_history (
  id SERIAL PRIMARY KEY,
  player_id INT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
  sale SMALLINT NOT NULL DEFAULT 0,
  itemtype SMALLINT NOT NULL,
  amount SMALLINT NOT NULL,
  price BIGINT NOT NULL DEFAULT 0,
  expires_at BIGINT NOT NULL,
  inserted BIGINT NOT NULL,
  state SMALLINT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_market_history_player_sale ON market_history(player_id, sale);

-- MARKET OFFERS
CREATE TABLE IF NOT EXISTS market_offers (
  id SERIAL PRIMARY KEY,
  player_id INT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
  sale SMALLINT NOT NULL DEFAULT 0,
  itemtype SMALLINT NOT NULL,
  amount SMALLINT NOT NULL,
  created BIGINT NOT NULL,
  anonymous SMALLINT NOT NULL DEFAULT 0,
  price BIGINT NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_market_offers_sale_itemtype ON market_offers(sale, itemtype);
CREATE INDEX IF NOT EXISTS idx_market_offers_created ON market_offers(created);

-- PLAYERS ONLINE (MEMORY in MySQL → UNLOGGED i Postgres för snabbhet)
CREATE UNLOGGED TABLE IF NOT EXISTS players_online (
  player_id INT PRIMARY KEY REFERENCES players(id) ON DELETE CASCADE
);

-- PLAYER DEATHS
CREATE TABLE IF NOT EXISTS player_deaths (
  player_id INT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
  time BIGINT NOT NULL DEFAULT 0,
  level INT NOT NULL DEFAULT 1,
  killed_by VARCHAR(255) NOT NULL,
  is_player SMALLINT NOT NULL DEFAULT 1,
  mostdamage_by VARCHAR(100) NOT NULL,
  mostdamage_is_player SMALLINT NOT NULL DEFAULT 0,
  unjustified SMALLINT NOT NULL DEFAULT 0,
  mostdamage_unjustified SMALLINT NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_player_deaths_killed_by ON player_deaths(killed_by);
CREATE INDEX IF NOT EXISTS idx_player_deaths_mostdamage_by ON player_deaths(mostdamage_by);

-- PLAYER DEPOT ITEMS
CREATE TABLE IF NOT EXISTS player_depotitems (
  player_id INT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
  sid INT NOT NULL,
  pid INT NOT NULL DEFAULT 0,
  itemtype SMALLINT NOT NULL,
  count SMALLINT NOT NULL DEFAULT 0,
  attributes BYTEA NOT NULL,
  UNIQUE (player_id, sid)
);

-- PLAYER INBOX ITEMS
CREATE TABLE IF NOT EXISTS player_inboxitems (
  player_id INT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
  sid INT NOT NULL,
  pid INT NOT NULL DEFAULT 0,
  itemtype SMALLINT NOT NULL,
  count SMALLINT NOT NULL DEFAULT 0,
  attributes BYTEA NOT NULL,
  UNIQUE (player_id, sid)
);

-- PLAYER STORE INBOX ITEMS
CREATE TABLE IF NOT EXISTS player_storeinboxitems (
  player_id INT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
  sid INT NOT NULL,
  pid INT NOT NULL DEFAULT 0,
  itemtype SMALLINT NOT NULL,
  count SMALLINT NOT NULL DEFAULT 0,
  attributes BYTEA NOT NULL,
  UNIQUE (player_id, sid)
);

-- PLAYER ITEMS
CREATE TABLE IF NOT EXISTS player_items (
  player_id INT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
  pid INT NOT NULL DEFAULT 0,
  sid INT NOT NULL DEFAULT 0,
  itemtype SMALLINT NOT NULL DEFAULT 0,
  count SMALLINT NOT NULL DEFAULT 0,
  attributes BYTEA NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_player_items_sid ON player_items(sid);

-- PLAYER SPELLS
CREATE TABLE IF NOT EXISTS player_spells (
  player_id INT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
  name VARCHAR(255) NOT NULL
);

-- PLAYER STORAGE
CREATE TABLE IF NOT EXISTS player_storage (
  player_id INT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
  key INT NOT NULL DEFAULT 0,
  value INT NOT NULL DEFAULT 0,
  PRIMARY KEY (player_id, key)
);

-- PLAYER OUTFITS
CREATE TABLE IF NOT EXISTS player_outfits (
  player_id INT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
  outfit_id SMALLINT NOT NULL DEFAULT 0,
  addons SMALLINT NOT NULL DEFAULT 0,
  PRIMARY KEY (player_id, outfit_id)
);

-- PLAYER MOUNTS
CREATE TABLE IF NOT EXISTS player_mounts (
  player_id INT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
  mount_id SMALLINT NOT NULL DEFAULT 0,
  PRIMARY KEY (player_id, mount_id)
);

-- SERVER CONFIG
CREATE TABLE IF NOT EXISTS server_config (
  config VARCHAR(50) PRIMARY KEY,
  value VARCHAR(256) NOT NULL DEFAULT ''
);
INSERT INTO server_config (config, value) VALUES
  ('db_version', '37'),
  ('players_record', '0')
ON CONFLICT (config) DO NOTHING;

-- SESSIONS
CREATE TABLE IF NOT EXISTS sessions (
  id SERIAL PRIMARY KEY,
  token BYTEA NOT NULL UNIQUE,   -- 16 bytes expected
  account_id INT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
  ip BYTEA NOT NULL,             -- 16 bytes expected
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  expired_at TIMESTAMP NULL
);

-- TILE STORE
CREATE TABLE IF NOT EXISTS tile_store (
  house_id INT NOT NULL REFERENCES houses(id) ON DELETE CASCADE,
  data BYTEA NOT NULL
);

-- TOWNS
CREATE TABLE IF NOT EXISTS towns (
  id SERIAL PRIMARY KEY,
  name VARCHAR(255) NOT NULL UNIQUE,
  posx INT NOT NULL DEFAULT 0,
  posy INT NOT NULL DEFAULT 0,
  posz INT NOT NULL DEFAULT 0
);

-- Triggers (PL/pgSQL)

-- When a player is deleted, release any house they owned
CREATE OR REPLACE FUNCTION ondelete_players() RETURNS trigger AS $$
BEGIN
  UPDATE houses SET owner = 0 WHERE owner = OLD.id;
  RETURN OLD;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_ondelete_players ON players;
CREATE TRIGGER trg_ondelete_players
BEFORE DELETE ON players
FOR EACH ROW
EXECUTE FUNCTION ondelete_players();

-- When a guild is created, insert default ranks
CREATE OR REPLACE FUNCTION oncreate_guilds() RETURNS trigger AS $$
BEGIN
  INSERT INTO guild_ranks (name, level, guild_id) VALUES
    ('the Leader', 3, NEW.id),
    ('a Vice-Leader', 2, NEW.id),
    ('a Member', 1, NEW.id);
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_oncreate_guilds ON guilds;
CREATE TRIGGER trg_oncreate_guilds
AFTER INSERT ON guilds
FOR EACH ROW
EXECUTE FUNCTION oncreate_guilds();
