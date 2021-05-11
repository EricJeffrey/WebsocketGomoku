use gomoku_game_websocket::gomoku_ol::{Context, PieceType};
use std::str;
use std::{
    collections::{HashMap, VecDeque},
    net::TcpStream,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread, usize,
};

use websocket::sync::Server;
use websocket::{sync::Client, Message};

enum ThreadJobCmd {
    SendData,
}
struct ThreadJob {
    job_cmd: ThreadJobCmd,
    data: String,
}
impl ThreadJob {
    fn new(job_cmd: ThreadJobCmd, data: &String) -> ThreadJob {
        ThreadJob {
            job_cmd,
            data: data.clone(),
        }
    }
}

fn format_res(data: Option<String>, resp_for: &str) -> String {
    format!(
        "{{\"ok\":{},\"type\":\"{}\",\"data\":{}}}",
        data.is_some(),
        resp_for,
        data.unwrap_or("\"no data\"".to_string())
    )
}

/// handle message of a client, return -1 if any send or recv failed
fn handle_message(
    context: &Arc<Mutex<Context>>,
    channels_map: &Arc<Mutex<HashMap<i32, Sender<ThreadJob>>>>,
    ws_client: &mut Client<TcpStream>,
    msg: &String,
) -> i32 {
    let lines: Vec<&str> = msg.split('\n').collect();
    if lines.len() == 0 {
        match ws_client.send_message(&Message::text(format_res(
            Some("Invalid Data".to_string()),
            "invalid data",
        ))) {
            Ok(_) => {}
            Err(_) => {
                eprintln!("send INVALID_DATA msg failed");
                return -1;
            }
        };
        return 0;
    }

    let mut players_to_resp: Vec<i32> = Vec::new();
    let mut resp_msg_to_all_player: Option<String> = None;
    let mut resp_msg: Option<String> = None;

    match lines[0] {
        "create_room" if lines.len() == 2 => {
            let room_name = lines[1];
            let mut tmp_context = context.lock().unwrap();
            resp_msg = Some(tmp_context.create_room(room_name.to_string()));
            resp_msg_to_all_player = Some(format!(
                "{{\"msg_others\":\"room_list\",\"data\":{}}}",
                tmp_context.room_list_json().unwrap()
            ));
            players_to_resp = tmp_context.all_players();
        }
        "room_list" => {
            resp_msg = context.lock().unwrap().room_list_json();
        }
        "enter_room" if lines.len() == 3 => {
            let player_id = lines[1].parse::<i32>();
            let room_id = lines[2].parse::<i32>();
            if player_id.is_ok() && room_id.is_ok() {
                let player_id = player_id.unwrap();
                let room_id = room_id.unwrap();
                let mut tmp_context = context.lock().unwrap();
                resp_msg = tmp_context.player_enter_room(player_id, room_id);
                if resp_msg.is_some() {
                    players_to_resp = tmp_context.players_of_room(room_id).unwrap_or_default();
                    resp_msg_to_all_player = Some(format!(
                        "{{\"msg_others\":\"enter_room\",\"data\":{{\"room_id\":{},\"player_id\":{},\"player_type\":{}}}}}",
                        room_id, player_id, tmp_context.type_of_player(player_id, room_id).unwrap()
                    ));
                }
            }
        }
        "exit_room" if lines.len() == 3 => {
            let player_id = lines[1].parse::<i32>();
            let room_id = lines[2].parse::<i32>();
            if player_id.is_ok() && room_id.is_ok() {
                let player_id = player_id.unwrap();
                let room_id = room_id.unwrap();
                let mut tmp_context = context.lock().unwrap();
                let player_type = tmp_context.type_of_player(player_id, room_id);
                resp_msg = tmp_context.player_exit_room(player_id, room_id);
                if resp_msg.is_some() {
                    players_to_resp = tmp_context.players_of_room(room_id).unwrap_or_default();
                    resp_msg_to_all_player = Some(format!(
                        "{{\"msg_others\":\"exit_room\",\"data\":{{\"room_id\":{},\"player_id\":{},\"player_type\":{}}}}}",
                        room_id, player_id,player_type.unwrap()
                    ));
                }
            }
        }
        "reset_game" => {
            if lines.len() == 2 {
                let room_id = lines[1].parse::<i32>();
                if room_id.is_ok() {
                    let room_id = room_id.unwrap();
                    let mut tmp_context = context.lock().unwrap();
                    resp_msg = tmp_context.reset_game(room_id);
                    if resp_msg.is_some() {
                        players_to_resp = tmp_context.players_of_room(room_id).unwrap_or_default();
                        resp_msg_to_all_player = Some(String::from("{\"msg_others\":\"reset\"}"));
                    }
                }
            }
        }
        "put_piece" => {
            if lines.len() == 5 {
                let room_id = lines[1].parse::<i32>();
                let row_i = lines[2].parse::<usize>();
                let col_j = lines[3].parse::<usize>();
                let piece_type = lines[4].parse::<i32>();
                if room_id.is_ok() && row_i.is_ok() && col_j.is_ok() && piece_type.is_ok() {
                    let room_id = room_id.unwrap();
                    let row_i = row_i.unwrap();
                    let col_j = col_j.unwrap();
                    let piece_type = PieceType::from_i32(piece_type.unwrap());
                    let mut tmp_context = context.lock().unwrap();
                    resp_msg = tmp_context.put_piece(room_id, row_i, col_j, piece_type);
                    if resp_msg.is_some() {
                        players_to_resp = tmp_context.players_of_room(room_id).unwrap_or_default();
                        resp_msg_to_all_player = Some(format!(
                            "{{\"msg_others\":\"put_piece\",\"data\":{{\"room_id\":{},\"row_i\":{},\"col_j\":{},\"piece_type\":{}}}}}",
                            room_id,
                            row_i,
                            col_j,
                            piece_type.to_i32()
                        ));
                    }
                }
            }
        }
        // "unput_piece" => {},
        _ => {
            resp_msg = Some("\"data\":\"unknown message\"".to_string());
        }
    };
    let resp_msg = format_res(resp_msg, lines[0]);

    if resp_msg_to_all_player.is_some() {
        let msg = resp_msg_to_all_player.unwrap();
        players_to_resp.iter().for_each(|v| {
            // ignore failure
            match channels_map.lock().unwrap().get_mut(v) {
                Some(sender) => {
                    sender
                        .send(ThreadJob::new(ThreadJobCmd::SendData, &msg))
                        .unwrap_or_default();
                }
                // ignore
                None => {}
            }
        });
    }

    loop {
        match ws_client.send_message(&Message::text(&resp_msg)) {
            Ok(_) => {
                break;
            }
            Err(err) => match err {
                websocket::WebSocketError::IoError(v) => match v.kind() {
                    std::io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    _ => {
                        eprintln!("send response msg failed:{}", v);
                        return -1;
                    }
                },
                _ => {
                    eprintln!("send response msg failed: {}", err);
                    return -1;
                }
            },
        }
    }
    return 0;
}

fn main() {
    let port = 8686;
    let ws_server = Server::bind(format!("0.0.0.0:{}", port))
        .expect(&format!("bind websocket to port {} failed", port));

    let context = Arc::new(Mutex::new(Context::new()));
    let channels_map: Arc<Mutex<HashMap<i32, Sender<ThreadJob>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    for connection in ws_server.filter_map(Result::ok) {
        let cloned_context = Arc::clone(&context);
        let cloned_channels_map = Arc::clone(&channels_map);
        thread::spawn(move || {
            let ws_client = connection.accept();
            if ws_client.is_err() {
                eprintln!("accept failed");
                return;
            }
            let mut ws_client = ws_client.unwrap();
            let peer_ip_addr = ws_client.peer_addr().unwrap().to_string();
            eprintln!("connection to {} established", &peer_ip_addr);

            // add to player list
            let player_id: i32 = { cloned_context.lock().unwrap().add_player(&peer_ip_addr) };
            // create a channel
            let receiver: Receiver<ThreadJob>;
            let (tx, rx) = mpsc::channel::<ThreadJob>();
            receiver = rx;
            {
                cloned_channels_map.lock().unwrap().insert(player_id, tx);
            }
            let shut_down =
                |ws_client: &Client<TcpStream>,
                 cloned_channels_map: &Arc<Mutex<HashMap<i32, Sender<ThreadJob>>>>| {
                    // remove channel
                    cloned_channels_map
                        .lock()
                        .unwrap()
                        .remove(&player_id)
                        .unwrap();
                    // shutdown
                    ws_client.shutdown().unwrap_or(());
                    cloned_context.lock().unwrap().remove_player(player_id);
                };

            // send id
            match ws_client.send_message(&Message::text(format_res(
                Some(format!("{{\"id\":{}}}", player_id)),
                "your_id",
            ))) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("send player id failed: {}", err);
                    shut_down(&ws_client, &cloned_channels_map);
                    return;
                }
            }

            // set non-blocking
            if ws_client.set_nonblocking(true).is_err() {
                eprintln!("set non-blocking failed");
                return;
            }

            let mut job_queue: VecDeque<ThreadJob> = VecDeque::new();
            let mut client_failure_or_closed = false;

            loop {
                // receive clients msg
                match ws_client.recv_message() {
                    Ok(v) => match v {
                        websocket::OwnedMessage::Text(msg) => {
                            let handle_res = handle_message(
                                &cloned_context,
                                &cloned_channels_map,
                                &mut ws_client,
                                &msg,
                            );
                            if handle_res == -1 {
                                eprintln!("handle message of client {} failed", &peer_ip_addr);
                                client_failure_or_closed = true;
                            }
                        }
                        websocket::OwnedMessage::Close(_) => {
                            eprintln!("client closing connection");
                            client_failure_or_closed = true;
                        }
                        _ => {
                            eprintln!("unsupported message type");
                        }
                    },
                    Err(err) => {
                        match err {
                            websocket::WebSocketError::IoError(v) => {
                                match v.kind() {
                                    // err will be IoError when non_blocking
                                    std::io::ErrorKind::WouldBlock => {}
                                    _ => {
                                        eprintln!("recv msg failed, io error: {}", v);
                                        client_failure_or_closed = true;
                                    }
                                }
                            }
                            websocket::WebSocketError::NoDataAvailable => {}
                            _ => {
                                eprintln!("recv msg failed: {}", err);
                                client_failure_or_closed = true;
                            }
                        }
                    }
                }
                if client_failure_or_closed {
                    break;
                }

                // check any msg from other threads
                match receiver.try_recv() {
                    Ok(v) => job_queue.push_back(v),
                    Err(err) => match err {
                        mpsc::TryRecvError::Empty => {}
                        mpsc::TryRecvError::Disconnected => {
                            eprintln!("sender of {} become disconnected", &player_id);
                            client_failure_or_closed = true;
                        }
                    },
                }
                if client_failure_or_closed {
                    break;
                }

                // handle those msg
                while !job_queue.is_empty() {
                    let tmp_job = job_queue.front().unwrap();
                    match tmp_job.job_cmd {
                        ThreadJobCmd::SendData => {
                            match ws_client.send_message(&Message::text(&tmp_job.data)) {
                                Ok(_) => {
                                    job_queue.pop_front();
                                }
                                Err(err) => match err {
                                    websocket::WebSocketError::IoError(v) => match v.kind() {
                                        std::io::ErrorKind::WouldBlock => {}
                                        _ => {
                                            eprintln!(
                                                "send msg to client {} failed, io error: {}",
                                                player_id, v
                                            );
                                            client_failure_or_closed = true;
                                        }
                                    },
                                    _ => {
                                        eprintln!(
                                            "send msg to client {} failed: {}",
                                            player_id, err
                                        );
                                        client_failure_or_closed = true;
                                    }
                                },
                            }
                        }
                    }
                }
                if client_failure_or_closed {
                    break;
                }
            }
            // thread::sleep(Duration::from_millis(10));
            shut_down(&ws_client, &cloned_channels_map);
        });
    }
}
