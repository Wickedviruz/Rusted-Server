use anyhow::Result;
use mlua::{Lua, Table, StdLib, LuaOptions};
use serde::Deserialize;
use tracing::warn;

#[derive(Debug, Deserialize, Clone)]
pub struct ExperienceStage {
    pub minlevel: u32,
    pub maxlevel: Option<u32>,
    pub multiplier: f32,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub world_type: String,
    pub hotkey_aimbot_enabled: bool,
    pub protection_level: i32,
    pub kills_to_red_skull: i32,
    pub kills_to_black_skull: i32,
    pub pz_locked: i32,
    pub remove_charges_from_runes: bool,
    pub remove_charges_from_potions: bool,
    pub remove_weapon_ammunition: bool,
    pub remove_weapon_charges: bool,
    pub time_to_decrease_frags: i32,
    pub white_skull_time: i32,
    pub stair_jump_exhaustion: i32,
    pub experience_by_killing_players: bool,
    pub exp_from_players_level_range: i32,
    pub ip: String,
    pub bind_only_global_address: bool,
    pub login_protocol_port: u16,
    pub game_protocol_port: u16,
    pub status_protocol_port: u16,
    pub max_players: i32,
    pub motd: String,
    pub one_player_online_per_account: bool,
    pub allow_clones: bool,
    pub allow_walkthrough: bool,
    pub server_name: String,
    pub status_timeout: i32,
    pub replace_kick_on_login: bool,
    pub max_packets_per_second: i32,
    pub death_lose_percent: i32,
    pub house_price_each_sqm: i32,
    pub house_rent_period: String,
    pub house_owned_by_account: bool,
    pub house_door_show_price: bool,
    pub only_invited_can_move_house_items: bool,
    pub time_between_actions: i32,
    pub time_between_ex_actions: i32,
    pub map_name: String,
    pub map_author: String,
    pub market_offer_duration: i32,
    pub premium_to_create_market_offer: bool,
    pub check_expired_market_offers_each_minutes: i32,
    pub max_market_offers_at_a_time_per_player: i32,
    pub mysql_host: String,
    pub mysql_user: String,
    pub mysql_pass: String,
    pub mysql_database: String,
    pub mysql_port: i32,
    pub mysql_sock: String,
    pub allow_change_outfit: bool,
    pub free_premium: bool,
    pub kick_idle_player_after_minutes: i32,
    pub max_message_buffer: i32,
    pub emote_spells: bool,
    pub classic_equipment_slots: bool,
    pub classic_attack_speed: bool,
    pub show_scripts_log_in_console: bool,
    pub show_online_status_in_charlist: bool,
    pub yell_minimum_level: i32,
    pub yell_always_allow_premium: bool,
    pub force_monster_types_on_load: bool,
    pub clean_protection_zones: bool,
    pub lua_item_desc: bool,
    pub show_player_log_in_console: bool,
    pub vip_free_limit: i32,
    pub vip_premium_limit: i32,
    pub depot_free_limit: i32,
    pub depot_premium_limit: i32,
    pub default_world_light: bool,
    pub server_save_notify_message: bool,
    pub server_save_notify_duration: i32,
    pub server_save_clean_map: bool,
    pub server_save_close: bool,
    pub server_save_shutdown: bool,
    pub experience_stages: Vec<ExperienceStage>,
    pub rate_exp: i32,
    pub rate_skill: i32,
    pub rate_loot: i32,
    pub rate_magic: i32,
    pub rate_spawn: i32,
    pub despawn_range: i32,
    pub despawn_radius: i32,
    pub remove_on_despawn: bool,
    pub walk_to_spawn_radius: i32,
    pub stamina_system: bool,
    pub warn_unsafe_scripts: bool,
    pub convert_unsafe_scripts: bool,
    pub default_priority: String,
    pub startup_database_optimization: bool,
    pub owner_name: String,
    pub owner_email: String,
    pub url: String,
    pub location: String,
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let lua = Lua::new_with(StdLib::ALL_SAFE, LuaOptions::default())?;
        let globals = lua.globals();

        let content = std::fs::read_to_string(path)?;
        if let Err(e) = lua.load(&content).set_name("config.lua").exec() {
            println!("Lua error: {:?}", e);
            return Err(e.into());
        }
        

        // Helper: fetch key or fallback
        fn get_or_default<T>(globals: &Table, key: &str, default: T) -> T
        where
            for<'lua> T: mlua::FromLua<'lua> + Clone + std::fmt::Debug,
        {
            match globals.get::<_, T>(key) {
                Ok(val) => val,
                Err(_) => {
                    warn!("Missing config key '{}', using default {:?}", key, default);
                    default
                }
            }
        }

        // Special: experienceStages
        let mut experience_stages = Vec::new();
        if let Ok(stages_table) = globals.get::<_, Table>("experienceStages") {
            for pair in stages_table.sequence_values::<Table>() {
                if let Ok(t) = pair {
                    let minlevel: u32 = get_or_default(&t, "minlevel", 1);
                    let maxlevel: Option<u32> = t.get("maxlevel").ok();
                    let multiplier: f32 = get_or_default(&t, "multiplier", 1.0);
                    experience_stages.push(ExperienceStage { minlevel, maxlevel, multiplier });
                }
            }
        }

        Ok(Config {
            world_type: get_or_default(&globals, "worldType", "pvp".to_string()),
            hotkey_aimbot_enabled: get_or_default(&globals, "hotkeyAimbotEnabled", false),
            protection_level: get_or_default(&globals, "protectionLevel", 1),
            kills_to_red_skull: get_or_default(&globals, "killsToRedSkull", 3),
            kills_to_black_skull: get_or_default(&globals, "killsToBlackSkull", 6),
            pz_locked: get_or_default(&globals, "pzLocked", 60000),
            remove_charges_from_runes: get_or_default(&globals, "removeChargesFromRunes", true),
            remove_charges_from_potions: get_or_default(&globals, "removeChargesFromPotions", true),
            remove_weapon_ammunition: get_or_default(&globals, "removeWeaponAmmunition", true),
            remove_weapon_charges: get_or_default(&globals, "removeWeaponCharges", true),
            time_to_decrease_frags: get_or_default(&globals, "timeToDecreaseFrags", 24 * 60 * 60),
            white_skull_time: get_or_default(&globals, "whiteSkullTime", 15 * 60),
            stair_jump_exhaustion: get_or_default(&globals, "stairJumpExhaustion", 2000),
            experience_by_killing_players: get_or_default(&globals, "experienceByKillingPlayers", false),
            exp_from_players_level_range: get_or_default(&globals, "expFromPlayersLevelRange", 75),
            ip: get_or_default(&globals, "ip", "127.0.0.1".to_string()),
            bind_only_global_address: get_or_default(&globals, "bindOnlyGlobalAddress", false),
            login_protocol_port: get_or_default(&globals, "loginProtocolPort", 7171),
            game_protocol_port: get_or_default(&globals, "gameProtocolPort", 7172),
            status_protocol_port: get_or_default(&globals, "statusProtocolPort", 7171),
            max_players: get_or_default(&globals, "maxPlayers", 0),
            motd: get_or_default(&globals, "motd", "Welcome!".to_string()),
            one_player_online_per_account: get_or_default(&globals, "onePlayerOnlinePerAccount", true),
            allow_clones: get_or_default(&globals, "allowClones", false),
            allow_walkthrough: get_or_default(&globals, "allowWalkthrough", true),
            server_name: get_or_default(&globals, "serverName", "Rusted-Server".to_string()),
            status_timeout: get_or_default(&globals, "statusTimeout", 5000),
            replace_kick_on_login: get_or_default(&globals, "replaceKickOnLogin", true),
            max_packets_per_second: get_or_default(&globals, "maxPacketsPerSecond", 25),
            death_lose_percent: get_or_default(&globals, "deathLosePercent", -1),
            house_price_each_sqm: get_or_default(&globals, "housePriceEachSQM", 1000),
            house_rent_period: get_or_default(&globals, "houseRentPeriod", "never".to_string()),
            house_owned_by_account: get_or_default(&globals, "houseOwnedByAccount", false),
            house_door_show_price: get_or_default(&globals, "houseDoorShowPrice", true),
            only_invited_can_move_house_items: get_or_default(&globals, "onlyInvitedCanMoveHouseItems", true),
            time_between_actions: get_or_default(&globals, "timeBetweenActions", 200),
            time_between_ex_actions: get_or_default(&globals, "timeBetweenExActions", 1000),
            map_name: get_or_default(&globals, "mapName", "forgotten".to_string()),
            map_author: get_or_default(&globals, "mapAuthor", "Unknown".to_string()),
            market_offer_duration: get_or_default(&globals, "marketOfferDuration", 30 * 24 * 60 * 60),
            premium_to_create_market_offer: get_or_default(&globals, "premiumToCreateMarketOffer", true),
            check_expired_market_offers_each_minutes: get_or_default(&globals, "checkExpiredMarketOffersEachMinutes", 60),
            max_market_offers_at_a_time_per_player: get_or_default(&globals, "maxMarketOffersAtATimePerPlayer", 100),
            mysql_host: get_or_default(&globals, "mysqlHost", "127.0.0.1".to_string()),
            mysql_user: get_or_default(&globals, "mysqlUser", "root".to_string()),
            mysql_pass: get_or_default(&globals, "mysqlPass", "".to_string()),
            mysql_database: get_or_default(&globals, "mysqlDatabase", "rusted".to_string()),
            mysql_port: get_or_default(&globals, "mysqlPort", 3306),
            mysql_sock: get_or_default(&globals, "mysqlSock", "".to_string()),
            allow_change_outfit: get_or_default(&globals, "allowChangeOutfit", true),
            free_premium: get_or_default(&globals, "freePremium", false),
            kick_idle_player_after_minutes: get_or_default(&globals, "kickIdlePlayerAfterMinutes", 15),
            max_message_buffer: get_or_default(&globals, "maxMessageBuffer", 4),
            emote_spells: get_or_default(&globals, "emoteSpells", false),
            classic_equipment_slots: get_or_default(&globals, "classicEquipmentSlots", false),
            classic_attack_speed: get_or_default(&globals, "classicAttackSpeed", false),
            show_scripts_log_in_console: get_or_default(&globals, "showScriptsLogInConsole", false),
            show_online_status_in_charlist: get_or_default(&globals, "showOnlineStatusInCharlist", false),
            yell_minimum_level: get_or_default(&globals, "yellMinimumLevel", 2),
            yell_always_allow_premium: get_or_default(&globals, "yellAlwaysAllowPremium", false),
            force_monster_types_on_load: get_or_default(&globals, "forceMonsterTypesOnLoad", true),
            clean_protection_zones: get_or_default(&globals, "cleanProtectionZones", false),
            lua_item_desc: get_or_default(&globals, "luaItemDesc", false),
            show_player_log_in_console: get_or_default(&globals, "showPlayerLogInConsole", true),
            vip_free_limit: get_or_default(&globals, "vipFreeLimit", 20),
            vip_premium_limit: get_or_default(&globals, "vipPremiumLimit", 100),
            depot_free_limit: get_or_default(&globals, "depotFreeLimit", 2000),
            depot_premium_limit: get_or_default(&globals, "depotPremiumLimit", 10000),
            default_world_light: get_or_default(&globals, "defaultWorldLight", true),
            server_save_notify_message: get_or_default(&globals, "serverSaveNotifyMessage", true),
            server_save_notify_duration: get_or_default(&globals, "serverSaveNotifyDuration", 5),
            server_save_clean_map: get_or_default(&globals, "serverSaveCleanMap", false),
            server_save_close: get_or_default(&globals, "serverSaveClose", false),
            server_save_shutdown: get_or_default(&globals, "serverSaveShutdown", true),
            experience_stages: Vec::new(),
            rate_exp: get_or_default(&globals, "rateExp", 5),
            rate_skill: get_or_default(&globals, "rateSkill", 3),
            rate_loot: get_or_default(&globals, "rateLoot", 2),
            rate_magic: get_or_default(&globals, "rateMagic", 3),
            rate_spawn: get_or_default(&globals, "rateSpawn", 1),
            despawn_range: get_or_default(&globals, "deSpawnRange", 2),
            despawn_radius: get_or_default(&globals, "deSpawnRadius", 50),
            remove_on_despawn: get_or_default(&globals, "removeOnDespawn", true),
            walk_to_spawn_radius: get_or_default(&globals, "walkToSpawnRadius", 15),
            stamina_system: get_or_default(&globals, "staminaSystem", true),
            warn_unsafe_scripts: get_or_default(&globals, "warnUnsafeScripts", true),
            convert_unsafe_scripts: get_or_default(&globals, "convertUnsafeScripts", true),
            default_priority: get_or_default(&globals, "defaultPriority", "high".to_string()),
            startup_database_optimization: get_or_default(&globals, "startupDatabaseOptimization", false),
            owner_name: get_or_default(&globals, "ownerName", "".to_string()),
            owner_email: get_or_default(&globals, "ownerEmail", "".to_string()),
            url: get_or_default(&globals, "url", "".to_string()),
            location: get_or_default(&globals, "location", "".to_string()),
        })
    }
}
