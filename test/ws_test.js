
/* 
"create_room" - room_name
"room_list" - 
"enter_room" - player_id, room_id
"exit_room" - player_id, room_id
"reset_game" - room_id
"put_piece" - room_id, row_i, col_j, piece_type(0:Black,1:White)
*/

let ws = new WebSocket("ws://192.168.31.4:8686");
// ws.addEventListener('open', (ev) => { ws.send("--client hello--") });
ws.addEventListener('message', (ev) => {
    let data = JSON.parse(ev.data);
    console.log(data);
});
ws.addEventListener('close', (ev) => { console.log("closed", ev) });
ws.onopen = (ev) => {
    let sendCmd = (cmdList) => {
        ws.send(cmdList.join('\n'));
    }

    sendCmd(["room_list"]);
    sendCmd(["create_room", "hello"]);
    sendCmd(["create_room", "world"]);
    sendCmd(["room_list"]);
    sendCmd(["create_room", "allRight"]);
    sendCmd(["room_list"]);
    setTimeout(() => {
        ws.close();
    }, 3000);
};

