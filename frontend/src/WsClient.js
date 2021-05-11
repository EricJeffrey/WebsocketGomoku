/* 
"create_room" - room_name
"room_list" - 
"enter_room" - player_id, room_id
"exit_room" - player_id, room_id
"reset_game" - room_id
"put_piece" - room_id, row_i, col_j, piece_type(0:Black,1:White)
*/

export function sendMsg(wsClient, cmdAndData) {
    wsClient.send(cmdAndData.join('\n'));
}