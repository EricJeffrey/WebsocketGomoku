pub mod gomoku_ol {
    use std::{
        collections::{HashMap, HashSet},
        usize,
    };

    struct Player {
        _id: i32,
        _ip_addr: String,
    }

    #[derive(Clone, Copy)]
    pub enum PieceType {
        EMPTY,
        BLACK,
        WHITE,
    }
    impl PieceType {
        pub fn from_i32(v: i32) -> PieceType {
            match v {
                0 => PieceType::BLACK,
                1 => PieceType::WHITE,
                _ => PieceType::EMPTY,
            }
        }
        pub fn to_i32(&self) -> i32 {
            match self {
                PieceType::EMPTY => -1,
                PieceType::BLACK => 0,
                PieceType::WHITE => 1,
            }
        }
    }

    #[derive(Clone, Copy, Eq, PartialEq)]
    enum PlayerType {
        OBSERVER,
        PLAYER1,
        PLAYER2,
    }
    impl PlayerType {
        pub fn to_i32(&self) -> i32 {
            match self {
                PlayerType::PLAYER1 => 0,
                PlayerType::PLAYER2 => 1,
                PlayerType::OBSERVER => -1
            }
        }
    }

    pub struct Context {
        rooms: HashMap<i32, Room>,
        players: HashMap<i32, Player>,
        player_id_cnt: i32,
        room_id_cnt: i32,
    }
    impl Context {
        pub fn new() -> Context {
            Context {
                rooms: HashMap::new(),
                players: HashMap::new(),
                player_id_cnt: 0,
                room_id_cnt: 0,
            }
        }

        pub fn add_player(&mut self, ip_addr: &String) -> i32 {
            self.player_id_cnt += 1;
            let id = self.player_id_cnt;
            self.players.insert(
                id,
                Player {
                    _id: id,
                    _ip_addr: ip_addr.clone(),
                },
            );
            id
        }

        pub fn remove_player(&mut self, player_id: i32) {
            self.players.remove(&player_id);
            self.rooms.iter_mut().for_each(|v| {
                v.1.remove_player(player_id);
            });
        }

        pub fn create_room(&mut self, name: String) -> String {
            self.room_id_cnt += 1;
            let id = self.room_id_cnt;
            self.rooms.insert(id, Room::new(id, name.clone()));
            format!("{{\"room\":{{\"id\":{},\"name\":\"{}\"}}}}", id, name)
        }

        pub fn room_list_json(&self) -> Option<String> {
            Some(format!(
                "[{}]",
                self.rooms
                    .iter()
                    .map(|v| -> String { v.1.to_json() })
                    .collect::<Vec<String>>()
                    .join(",")
            ))
        }

        pub fn player_enter_room(&mut self, player_id: i32, room_id: i32) -> Option<String> {
            if self.players.contains_key(&player_id) {
                return match self.rooms.get_mut(&room_id) {
                    Some(room) => {
                        room.add_player(player_id);
                        Some(format!("{}", room.to_json()))
                    }
                    None => None,
                };
            }
            None
        }

        pub fn player_exit_room(&mut self, player_id: i32, room_id: i32) -> Option<String> {
            if self.players.contains_key(&player_id) {
                return match self.rooms.get_mut(&room_id) {
                    Some(room) => {
                        room.remove_player(player_id);
                        Some("{}".to_string())
                    }
                    None => None,
                };
            }
            None
        }

        pub fn reset_game(&mut self, room_id: i32) -> Option<String> {
            match self.rooms.get_mut(&room_id) {
                Some(room) => {
                    room.game.reset();
                    Some("{}".to_string())
                }
                None => None,
            }
        }

        pub fn put_piece(
            &mut self,
            room_id: i32,
            row_i: usize,
            col_j: usize,
            piece_type: PieceType,
        ) -> Option<String> {
            match piece_type {
                PieceType::EMPTY => None,
                _ => match self.rooms.get_mut(&room_id) {
                    Some(room) => {
                        if row_i < room.game.row_size && col_j < room.game.col_size {
                            room.game.put_piece(row_i, col_j, piece_type);
                            return Some("{}".to_string());
                        }
                        return None;
                    }
                    None => None,
                },
            }
        }

        pub fn players_of_room(&self, room_id: i32) -> Option<Vec<i32>> {
            match self.rooms.get(&room_id) {
                Some(room) => Some(room.all_players()),
                None => None,
            }
        }

        pub fn all_players(&self) -> Vec<i32> {
            self.players
                .iter()
                .map(|(k, _)| k.clone())
                .collect::<Vec<i32>>()
        }

        pub fn type_of_player(&self, player_id: i32, room_id: i32) -> Option<i32> {
            match self.rooms.get(&room_id) {
                Some(room) => {
                    if room.game_players.contains_key(&player_id) {
                        Some(room.game_players.get(&player_id).unwrap().to_i32())
                    } else if room.game_observers.contains(&player_id) {
                        Some(PlayerType::OBSERVER.to_i32())
                    } else {
                        None
                    }
                }
                None => None
            }
        }
    }

    pub struct Game {
        row_size: usize,
        col_size: usize,
        board: Vec<Vec<PieceType>>,
    }
    impl Game {
        fn new(row_size: usize, col_size: usize) -> Game {
            let mut game = Game {
                row_size,
                col_size,
                board: Vec::with_capacity(row_size),
            };
            game.board.resize(row_size, Vec::with_capacity(col_size));
            for i in 0..col_size {
                game.board[i].resize(col_size, PieceType::EMPTY);
            }
            game
        }
        fn to_json(&self) -> String {
            format!(
                "{{\"row_size\":{},\"col_size\":{}}}",
                self.row_size, self.col_size
            )
        }
        fn reset(&mut self) {
            for i in 0..self.row_size {
                self.board[i].fill(PieceType::EMPTY);
            }
        }
        fn put_piece(&mut self, row_i: usize, col_j: usize, piece_type: PieceType) {
            self.board[row_i][col_j] = piece_type;
        }
    }

    struct Room {
        id: i32,
        game_players: HashMap<i32, PlayerType>,
        game_observers: HashSet<i32>,
        name: String,
        pub game: Game,
    }
    impl Room {
        fn new(id: i32, name: String) -> Room {
            Room {
                id,
                name,
                game_players: HashMap::new(),
                game_observers: HashSet::new(),
                game: Game::new(10, 10),
            }
        }
        fn to_json(&self) -> String {
            format!(
                "{{\"id\":{},\"name\":\"{}\",\"game_players\":{{{}}},\"game_observers\": [{}],\"game\":{}}}",
                self.id,
                self.name,
                self.game_players
                    .iter()
                    .map(|(id,player_type)| {format!("\"{}\":{}", id, player_type.to_i32())})
                    .collect::<Vec<String>>()
                    .join(","),
                self.game_observers
                    .iter()
                    .map(|v| {format!("{}", v)})
                    .collect::<Vec<String>>()
                    .join(","),
                self.game.to_json()
            )
        }
        fn add_player(&mut self, player_id: i32) {
            match self.game_players.len() {
                0 => {
                    self.game_players.insert(player_id, PlayerType::PLAYER1);
                }
                1 => {
                    match self.game_players.iter().next().unwrap().1 {
                        PlayerType::PLAYER1 => {
                            self.game_players.insert(player_id, PlayerType::PLAYER2);
                        }
                        PlayerType::PLAYER2 => {
                            self.game_players.insert(player_id, PlayerType::PLAYER1);
                        }
                        PlayerType::OBSERVER => {}
                    };
                }
                _ => {
                    self.game_observers.insert(player_id);
                }
            };
        }
        fn remove_player(&mut self, player_id: i32) {
            self.game_players.remove(&player_id);
            self.game_observers.remove(&player_id);
        }
        pub fn all_players(&self) -> Vec<i32> {
            self.game_players
                .iter()
                .map(|(k, _)| k)
                .chain(self.game_observers.iter())
                .map(|v| v.clone())
                .collect::<Vec<i32>>()
        }
    }
}
