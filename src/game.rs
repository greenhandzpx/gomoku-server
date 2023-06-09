use std::{collections::BTreeMap, sync::{Arc}};

use futures_util::{StreamExt, SinkExt};
use log::{error, info};
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};
use tokio::{net::TcpStream, sync::{Mutex, mpsc}};
use tokio_tungstenite::WebSocketStream;


const SERERATOR: char = ',';
const HEIGHT: usize = 10;
const WIDTH: usize = 10;


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
    player1: Option<Player>,
    player2: Option<Player>,
}

impl Game {

    pub fn new(player1: Player, player2: Player, game_id: usize) -> Self {
        Self {
            player1: Some(player1),
            player2: Some(player2),
            game_id,
        }
    }

    pub fn init(&mut self) {
        let (tx1, rx1) = mpsc::channel(100);
        let (tx2, rx2) = mpsc::channel(100);
        self.player1.as_mut().unwrap().join_game(tx1, rx2);
        self.player2.as_mut().unwrap().join_game(tx2, rx1);
    }

    pub async fn run(&mut self) {
        // self.player1.run().await;
        // self.player2.run().await;
        let mut player1 = self.player1.take().unwrap();
        tokio::spawn(async move {
            player1.run().await;
        });
        let mut player2 = self.player2.take().unwrap();
        tokio::spawn(async move {
            player2.run().await;
        });
    }
}


pub struct ChessBoard {
    pub board: [[i32; WIDTH]; HEIGHT],
}

impl ChessBoard {
    pub fn new() -> Self {
        Self {
            board: [[-1; WIDTH]; HEIGHT],
        }
    }
}

pub struct Player {
    id: i32,
    ws: WebSocketStream<TcpStream>,
    name: String,
    chess_board: Arc<Mutex<ChessBoard>>,
    send_ch: Option<mpsc::Sender<MoveMsg>>,
    recv_ch: Option<mpsc::Receiver<MoveMsg>>,

}

pub struct MoveMsg {
    is_win: bool,
    name: String,
    x: usize,
    y: usize
}



#[derive(Serialize, Deserialize)]
pub struct CSMsg {
    x: usize,
    y: usize,
    name: String,
    turn: i32,
    msg_type: String,
}

pub enum MsgType {
    Start, 
    Moving,
    Win,
    Fail,
}

impl Player {

    pub fn new(ws: WebSocketStream<TcpStream>) -> Self {
        Self {
            id: 0,
            ws,
            chess_board: Arc::new(Mutex::new(ChessBoard::new())),
            name: String::new(),
            send_ch: None,
            recv_ch: None,
        }
    }

    pub fn join_game(&mut self, send_ch: mpsc::Sender<MoveMsg>, recv_ch: mpsc::Receiver<MoveMsg>) {
        self.send_ch = Some(send_ch);
        self.recv_ch = Some(recv_ch);
        // self.cond = Some(cond);
        // self.flag = Some(flag);
    }

    pub async fn run(&mut self) {

        // receive the username
        if let Some(msg) = self.ws.next().await {
            info!("[player {}] receive msg {:?}", self.id, &msg);
            let msg = msg.unwrap().to_string();
            self.name = msg;
            let move_msg = MoveMsg {
                is_win: false,
                name: self.name.clone(),
                x: 0,
                y: 0,
            };
            // send username to the opposite
            if let Err(_) = self.send_ch.as_mut().unwrap().send(move_msg).await {
                error!("Send msg failed!!");
            }
        }

        // wait for the opposite to send his name
        let opposite_msg = self.recv_ch.as_mut().unwrap().recv().await.unwrap();
        let opposite_name = opposite_msg.name;
        info!("[player {}] receive opposite name {}", self.id, &opposite_name);

        // send the `start` msg
        let cs_msg = CSMsg {
            x: 0,
            y: 0,
            name: opposite_name,
            turn: self.id,
            msg_type: "start".to_string(),
        };
        let msg = serde_json::to_string(&cs_msg).unwrap();
        info!("player {} start", self.id);
        self.ws.send(msg.into()).await.unwrap();


        if self.id == 2 {
            // Awaken by another player(because of his move)
            let opposite_msg = self.recv_ch.as_mut().unwrap().recv().await.unwrap();

            if opposite_msg.is_win {
                self.ws.send("fail".into()).await.unwrap();
            } else {
                let cs_msg = CSMsg {
                    x: opposite_msg.x,
                    y: opposite_msg.y,
                    name: self.name.clone(),
                    turn: 0,
                    msg_type: "moving".to_string(),
                };
                let msg = serde_json::to_string(&cs_msg).unwrap();
                self.ws.send(msg.into()).await.unwrap();
            }
        }
        
        while let Some(msg) = self.ws.next().await {
            let msg = msg.unwrap();
            info!("[player {}] receive msg {:?}", self.id, &msg);
            if msg.is_text() || msg.is_binary() {

                // println!("Server on message: {:?}", &msg);

                let msg_str = msg.to_string();

                let locs: Vec<&str> = msg_str.split(SERERATOR).collect();
                assert_eq!(locs.len(), 2);

                // // Check validity
                let x = locs[0].parse::<usize>().unwrap();
                let y = locs[1].parse::<usize>().unwrap();

                // if !self.valid(x, y).await {
                //     self.ws.send("error".into()).await.unwrap();
                //     continue;
                // }

                // // Modify chess board
                // self.chess_board.lock().await.board[x][y] = self.id;

                // // Check whether the player wins
                // // Inform the client
                // let is_win = self.check_win();
                // if is_win {
                //     self.ws.send("win".into()).await.unwrap();
                // } else {
                //     self.ws.send("ok".into()).await.unwrap();
                // }

                // Inform the other player
                let is_win = false;
                if let Err(_) = self.send_ch.as_mut().unwrap().send(
                    MoveMsg { is_win, x, y, name: "".to_string() }
                ).await {
                    error!("Send msg failed!!");
                }

                // if is_win {
                //     break;
                // }

                // We don't need to worry about the client's error move
                // since we will prevent that in the client side
                // so we just wait here until the another player make a move

                // Awaken by another player(because of his move)
                let opposite_msg = self.recv_ch.as_mut().unwrap().recv().await.unwrap();

                if opposite_msg.is_win {
                    self.ws.send("fail".into()).await.unwrap();
                    break;
                } else {
                    let cs_msg = CSMsg {
                        x: opposite_msg.x,
                        y: opposite_msg.y,
                        name: self.name.clone(),
                        turn: 0,
                        msg_type: "moving".to_string(),
                    };
                    info!("[player {}] send ({} {}) to client", self.id, cs_msg.x, cs_msg.y);
                    let msg = serde_json::to_string(&cs_msg).unwrap();
                    self.ws.send(msg.into()).await.unwrap();
                }

            }
        }
    }

    async fn valid(&self, x: usize, y: usize) -> bool {
        if x >= HEIGHT || y >= WIDTH {
            return false;
        }
        if self.chess_board.lock().await.board[x][y] != -1 {
            return false;
        }
        true
    }

    fn check_win(&self) -> bool {
        todo!();
    }
}




pub async fn check_waiting_player(mut new_player: Player) -> bool {
    let mut game_manager = GAME_MANAGER.lock().await;
    if game_manager.waiting_player.is_some() {
        let mut old_player = game_manager.waiting_player.take().unwrap();
        old_player.id = 1;
        new_player.id = 2;
        let game_id = game_manager.alloc_game_id();
        let mut new_game = Game::new(old_player, new_player, game_id);
        new_game.init();

        // new_game.run().await;
        game_manager.games.insert(game_id, new_game);
        let game = game_manager.games.get_mut(&game_id).unwrap();
        info!("start game! game id {}", game.game_id);
        game.run().await;
        // tokio::spawn(new_game.run());
        true
    } else {
        game_manager.waiting_player = Some(new_player);
        info!("waiting for another player");
        false
    }
}


