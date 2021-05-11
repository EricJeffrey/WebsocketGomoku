import React from 'react';
import { sendMsg } from "./WsClient";

function roomDiv(room, key, enterRoom) {
    return <div key={key}>
        <span>{room.name}</span>
        <button onClick={() => enterRoom(room)}>进入房间</button>
    </div>;
}

class Lobby extends React.Component {
    constructor(props) {
        super(props);

        this.handleMessage = this.handleMessage.bind(this);
        this.createRoom = this.createRoom.bind(this);
        this.updateNewRoomName = this.updateNewRoomName.bind(this);

        this.wsClient = props.wsClient;
        this.playerId = props.playerId;
        this.newRoomName = "";
        this.enterRoom = props.enterRoom;

        this.state = {
            roomList: [],
        }
    }

    componentDidMount() {
        if (this.wsClient.readyState === WebSocket.OPEN) {
            this.wsClient.addEventListener("message", this.handleMessage);
            sendMsg(this.wsClient, ["room_list"]);
        }
    }

    componentWillUnmount() {
        // remove listener
        if (this.wsClient.readyState === WebSocket.OPEN) {
            this.wsClient.removeEventListener("message", this.handleMessage);
        }
    }

    handleMessage(ev) {
        let data = JSON.parse(ev.data);
        if ("type" in data) {
            switch (data.type) {
                case "room_list":
                    this.setState({ roomList: data.data });
                    break;
                case "create_room":
                    sendMsg(this.wsClient, ["room_list"]);
                    break;
                default:
                    break;
            }
        }
        if ("msg_others" in data) {
            switch (data.msg_others) {
                case "room_list":
                    this.setState({ roomList: data.data });
                    break;
                default:
                    break;
            }
        }
    }

    createRoom(ev) {
        if (this.newRoomName.length > 0) {
            sendMsg(this.wsClient, ["create_room", this.newRoomName]);
        }
    }

    updateNewRoomName(ev) { this.newRoomName = ev.target.value; }

    render() {
        let rooms;
        if (this.state.roomList.length === 0) {
            rooms = <div>没有房间</div>;
        } else {
            rooms = this.state.roomList.map((room, i) => roomDiv(room, i, this.enterRoom));
        }
        return <div>
            <div style={{ display: 'flex', flexDirection: 'column' }}>
                <div style={{ display: 'flex', flexDirection: 'row', alignItems: 'center' }} >
                    <h4>房间列表</h4>
                    <button onClick={this.createRoom} style={{ marginLeft: "10px", height: "fit-content" }}>创建房间:</button>
                    <input
                        onInput={this.updateNewRoomName}
                        placeholder="房间名，不能为空"
                        type="text"
                        style={{ marginLeft: "5px", height: "fit-content" }} />
                </div>
                <div > {rooms} </div>
            </div>
        </div>;
    }
};
export default Lobby;