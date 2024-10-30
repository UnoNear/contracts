use near_sdk::store::{LookupMap, UnorderedMap};
use near_sdk::{env, near_bindgen, serde, AccountId, BorshStorageKey};
// use near_sdk::collections::{LookupMap, UnorderedMap};
// use near_sdk::collections::LookupMap;
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::serde_json::json;
use std::collections::HashMap;
use near_sdk::PanicOnDefault;
// use near_sdk::serde;
use md5;

use  near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

// Define the data structure for the game
#[derive(Clone,BorshDeserialize, BorshSerialize)]
pub struct Game {
    id: u64,
    players: Vec<AccountId>,
    is_active: bool,
    current_player_index: usize,
    state_hash: String,
    last_action_timestamp: u64,
    turn_count: u64,
    direction_clockwise: bool,
    is_started: bool,
}




// Define an action structure to keep track of player actions
#[derive(Clone,BorshDeserialize, BorshSerialize)]
pub struct Action {
    player: AccountId,
    action_hash: String,
    timestamp: u64,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            games: UnorderedMap::new(b"g".to_vec()),
            game_actions: LookupMap::new(b"a".to_vec()),
            game_counter: 0,
        }
    }
}

// Define the contract structure with storage keys
#[derive()]
enum StorageKey {
    Games,
    GameActions,
}


// Define the contract
#[near_bindgen]
#[derive()]
pub struct Contract {
    games: UnorderedMap<u64, Game>,
    game_actions: LookupMap<u64, Vec<Action>>,
    game_counter: u64,
}



#[near_bindgen]
impl Contract {
    // Create a new game
    pub fn create_game(&mut self, creator: AccountId) -> u64 {
        self.game_counter += 1;
        let new_game_id = self.game_counter;
        let initial_state_hash = self.hash_state(new_game_id, &creator.as_str());

        let game = Game {
            id: new_game_id,
            players: vec![creator],
            is_active: true,
            current_player_index: 0,
            state_hash: initial_state_hash,
            last_action_timestamp: env::block_timestamp(),
            turn_count: 0,
            direction_clockwise: true,
            is_started: false,
        };

        self.games.insert(&new_game_id, &game);
        new_game_id
    }

    // Start a game
    #[handle_result]
    pub fn start_game(&mut self, game_id: u64) -> Result<(), String> {
        let mut game = self.games.get(&game_id).ok_or("Game not found")?;
        
        if game.is_started {
            return Err("Game already started".to_string());
        }
        if game.players.len() < 2 {
            return Err("Not enough players".to_string());
        }

        game.is_started = true;
        game.state_hash = self.hash_state(game_id,  &game.players.iter().map(|account| account.to_string()).collect::<Vec<String>>().join(","));
        game.last_action_timestamp = env::block_timestamp();

        self.games.insert(&game_id, &game);
        Ok(())
    }
    
    // Join a game
    #[handle_result]

    pub fn join_game(&mut self, game_id: u64, joinee: AccountId) -> Result<(), String> {
        let mut game = self.games.get(&game_id).ok_or("Game not found")?;

        if !game.is_active {
            return Err("Game is not active".to_string());
        }
        if game.players.len() >= 10 {
            return Err("Game is full".to_string());
        }

        game.players.push(joinee);
        self.games.insert(&game_id, &game);
        Ok(())
    }

    // Submit an action in the game
    #[handle_result]
    pub fn submit_action(&mut self, game_id: u64, action_hash: String, actor: AccountId) -> Result<(), String> {
        let mut game = self.games.get(&game_id).ok_or("Game not found")?;

        if !game.is_active {
            return Err("Game is not active".to_string());
        }
        if !self.is_player_turn(&game, &actor) {
            return Err("Not your turn".to_string());
        }

        game.state_hash = self.hash_action(game.state_hash, &action_hash);
        let action = Action {
            player: actor,
            action_hash,
            timestamp: env::block_timestamp(),
        };

        self.game_actions.entry(game_id).or_default().push(action);
        self.update_game_state(&mut game);
        self.games.insert(&game_id, &game);
        Ok(())
    }

    // End a game
    #[handle_result]

    pub fn end_game(&mut self, game_id: u64, actor: AccountId) -> Result<(), String> {
        let mut game = self.games.get(&game_id).ok_or("Game not found")?;
        
        if !game.is_active {
            return Err("Game is not active".to_string());
        }
        if !self.is_player_turn(&game, &actor) {
            return Err("Not your turn".to_string());
        }

        game.is_active = false;
        self.games.insert(&game_id, &game);
        Ok(())
    }

    // Get game state
    pub fn get_game_state(&self, game_id: u64) -> Option<Game> {
        self.games.get(&game_id).cloned()
    }

    // Get game actions
    pub fn get_game_actions(&self, game_id: u64) -> Vec<Action> {
        self.game_actions.get(&game_id).cloned().unwrap_or_default()
    }

    // Helper functions
    fn hash_state(&self, game_id: u64, seed: &str) -> String {
        format!("{:x}", md5::compute(format!("{}{}", game_id, seed)))
    }

    fn hash_action(&self, state_hash: String, action_hash: &str) -> String {
        format!("{:x}", md5::compute(format!("{}{}", state_hash, action_hash)))
    }

    fn is_player_turn(&self, game: &Game, player: &AccountId) -> bool {
        &game.players[game.current_player_index] == player
    }

    fn update_game_state(&self, game: &mut Game) {
        game.turn_count += 1;
        game.current_player_index = (game.current_player_index + 1) % game.players.len();
        game.last_action_timestamp = env::block_timestamp();
    }
}
