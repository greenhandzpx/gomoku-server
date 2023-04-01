use std::{collections::BTreeMap, cell::{RefCell, UnsafeCell}, sync::{Condvar, Arc}};

use futures_util::{StreamExt, SinkExt};
use once_cell::sync::Lazy;
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::WebSocketStream;



pub struct GameManager {
    pub games: BTreeMap<usize, Game>,
    pub waiting_player: Option<Player>,
    pub cnt: usize,
}

impl GameManager {
    pub fn new() -> Self {
        Self {
            games: BTreeMap::new(),
            waiting_player: None,
            cnt: 0,
        }
    }

    pub fn alloc_game_id(&mut self) -> usize {
        self.cnt += 1;
        self.cnt - 1
    }
}

static GAME_MANAGER: Lazy<Mutex<GameManager>> = Lazy::new(||
    Mutex::new(GameManager::new()));

pub struct Game {
    game_id: usize,
    player1: Player,
    player2: Player,
    cond: Arc<Condvar>,
    flag: Arc<Mutex<bool>>,
}

impl Game {

    pub fn new(player1: Player, player2: Player, game_id: usize) -> Self {
        Self {
            player1,
            player2,
            game_id,
            cond: Arc::new(Condvar::new()),
            flag: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn run(&self) {
        // self.player1.run().await;
        // self.player2.run().await;
    }

}

enum State {

}


pub struct Player {
    ws: WebSocketStream<TcpStream>,

}

impl Player {

    pub fn new(ws: WebSocketStream<TcpStream>) -> Self {
        Self {
            ws
        }
    }

    pub async fn run(&mut self) {
        while let Some(msg) = self.ws.next().await {
            let msg = msg.unwrap();
            if msg.is_text() || msg.is_binary() {
                println!("Server on message: {:?}", &msg);

                // We don't need to worry about the client's error move
                // since we will prevent that in the client side
                // so we just wait here until the another player make a move








                self.ws.send(msg).await.unwrap();
            }
        }
    }
}




// pub async fn check_waiting_player(new_player: Player) -> bool {
//     let mut game_manager = GAME_MANAGER.lock().await;
//     if game_manager.waiting_player.is_some() {
//         let old_player = game_manager.waiting_player.take().unwrap();
//         let game_id = game_manager.alloc_game_id();
//         let new_game = Game::new(old_player, new_player, game_id);
//         // new_game.run().await;
//         game_manager.games.insert(game_id, new_game);
//         tokio::spawn(new_game.run());
//         true
//     } else {
//         game_manager.waiting_player = Some(new_player);
//         false
//     }
// }


