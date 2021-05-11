import React from 'react';
import './App.css';
import Lobby from './Lobby';
import GomokuGame from './GomokuGame';
import { sendMsg } from './WsClient';

class App extends React.Component {
    constructor(props) {
        super(props);

        this.handleServerMsg = this.handleServerMsg.bind(this);
        this.enterRoom = this.enterRoom.bind(this);
        this.exitRoom = this.exitRoom.bind(this);

        this.wsClient = null;
        this.state = {
            onLine: false,
            playerId: null,
            inLobbyElseGame: true,
            currentRoom: null
        }
    }

    componentDidMount() {
        // todo change to 192.168.31.4
        let wsClient = new WebSocket("ws://localhost:8686");
        // wsClient.onopen = () => { };
        wsClient.onclose = (ev) => {
            console.log("websocket closed:", ev);
            this.setState({ onLine: false });
        };
        wsClient.addEventListener("message", this.handleServerMsg);
        this.wsClient = wsClient;
    }

    componentWillUnmount() {
        if (this.wsClient.readyState === WebSocket.OPEN) {
            this.wsClient.close();
            this.setState({ onLine: false });
        }
    }

    enterRoom(room) {
        if (this.state.currentRoom === null) {
            sendMsg(this.wsClient, ["enter_room", (this.state.playerId).toString(), (room.id).toString()]);
        }
    }

    exitRoom(room) {
        sendMsg(this.wsClient, ["exit_room", (this.state.playerId).toString(), (room.id).toString()]);
    }

    handleServerMsg(ev) {
        let data = JSON.parse(ev.data);
        if ("type" in data) {
            switch (data.type) {
                case "your_id":
                    this.setState({ playerId: data.data.id, onLine: true });
                    break;
                case "enter_room":
                    this.setState({ inLobbyElseGame: false, currentRoom: data.data });
                    break;
                case "exit_room":
                    this.setState({ inLobbyElseGame: true, currentRoom: null });
                    break;
                default:
                    break;
            }
        }
    }

    render() {
        let tmpClose = () => { this.wsClient.close(); };
        let content;
        if (this.state.onLine) {
            if (this.state.inLobbyElseGame) {
                content = <Lobby
                    enterRoom={this.enterRoom}
                    wsClient={this.wsClient}
                    playerId={this.state.playerId}
                />;
            } else {
                content = <GomokuGame
                    playerId={this.state.playerId}
                    wsClient={this.wsClient}
                    currentRoom={this.state.currentRoom}
                    exitRoom={this.exitRoom} />;
            }
        } else {
            content = <div></div>;
        }
        return (
            <div>
                <div style={{ display: 'flex', flexDirection: "row" }}>
                    <div>id:{this.state.playerId}</div>
                    <div style={{ marginLeft: "10px" }}>{this.state.onLine ? "在线🟢" : "离线🔴"}</div>
                    {
                        this.state.onLine ? <button onClick={tmpClose} style={{ marginLeft: "10px" }}>断开连接</button> : <></>
                    }
                </div>
                <div> {content} </div>
            </div>
        );
    }
}

export default App;
