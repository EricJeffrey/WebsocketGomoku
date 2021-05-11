import React from 'react';
import { Graphics, Application } from "pixi.js";
import { sendMsg } from './WsClient';

const PIECE_TYPE = {
    EMPTY: -1,
    BLACK: 0,
    WHITE: 1,
    toColor: function (pieceType) {
        if (pieceType === this.WHITE) {
            return 0xffffff;
        }
        if (pieceType === this.BLACK) {
            return 0x0;
        }
        return 0x333333;

    }
}
const PLAYER_TYPE = {
    PLAYER1: 0,
    PLAYER2: 1,
    OBSERVER: -1,
    toStr: function (type) {
        switch (type) {
            case this.PLAYER1:
                return "黑子";
            case this.PLAYER2:
                return "白子";
            case this.OBSERVER:
                return "观战";
            default:
                return "错误";
        }
    },
    fromI32: function (v) {
        switch (v) {
            case 0:
                return this.PLAYER1;
            case 1:
                return this.PLAYER2;
            default:
                return this.OBSERVER;
        }
    },
    toPieceType: function (v) {
        switch (v) {
            case PLAYER_TYPE.PLAYER1:
                return PIECE_TYPE.BLACK;
            case PLAYER_TYPE.PLAYER2:
                return PIECE_TYPE.WHITE;
            default:
                return PIECE_TYPE.EMPTY;
        }
    }
}

class GomokuGame extends React.Component {
    constructor(props) {
        super(props);

        this.startGame = this.resetGame.bind(this);
        this.handleMessage = this.handleMessage.bind(this);
        this.putPiece = this.putPiece.bind(this);
        this.winCheck = this.winCheck.bind(this);
        this.onBoardClick = this.onBoardClick.bind(this);

        this.exitRoom = props.exitRoom;
        this.wsClient = props.wsClient;
        this.currentRoom = props.currentRoom;
        this.playerId = props.playerId;
        this.playerType = PLAYER_TYPE.fromI32(this.currentRoom.game_players[this.playerId]);
        this.pieceType = PLAYER_TYPE.toPieceType(this.playerType);

        this.board = {
            rowSize: 13,
            colSize: 13,
            grid: [],
            isEmpty: function (row, col) { return this.grid[row][col] === PIECE_TYPE.EMPTY; },
            putPiece: function (rowI, colJ, type) {
                this.grid[rowI][colJ] = type;
            }
        };

        for (let i = 0; i < this.board.rowSize; i++) {
            this.board.grid.push([]);
            for (let j = 0; j < this.board.colSize; j++) {
                this.board.grid[i].push(PIECE_TYPE.EMPTY);
            }
        }
        this.pixiHolder = null;
        this.pixiApp = new Application(
            { width: 800, height: 800, antialias: true }
        );
        this.pixiApp.renderer.backgroundColor = 0xffffff;

        this.boardPainter = {
            rowHeight: 50,
            colWidth: 50,
            boardBgColor: 0xccb97b,
            gridLineColor: 0x008888,
            gridLineWidth: 2,
            boardX: 10,
            boardY: 10,
            app: this.pixiApp,
            board: this.board,
            pieceType: this.pieceType,
            onBoardClick: this.onBoardClick,

            redrawBoard: function () {
                if (this.app != null) { this.app.stage.removeChildren(); }
                let rectangle = new Graphics();
                rectangle.interactive = true;

                const boardHeight = (this.board.rowSize) * this.gridLineWidth + this.board.rowSize * this.rowHeight;
                const boardWidth = (this.board.colSize) * this.gridLineWidth + this.board.colSize * this.colWidth;
                rectangle.beginFill(this.boardBgColor);
                rectangle.drawRect(0, 0, boardWidth, boardHeight);
                rectangle.endFill();
                rectangle.x = this.boardX;
                rectangle.y = this.boardY;
                rectangle.addListener("click", this.onBoardClick);
                this.app.stage.addChild(rectangle);

                for (let i = 0; i < this.board.rowSize + 1; i++) {
                    let tmpLine = new Graphics();
                    tmpLine.lineStyle(this.gridLineWidth, this.gridLineColor, 1);
                    tmpLine.moveTo(0, 0);
                    tmpLine.lineTo(boardWidth, 0);
                    tmpLine.x = this.boardX;
                    tmpLine.y = this.boardY + (i * (this.rowHeight + this.gridLineWidth));
                    this.app.stage.addChild(tmpLine);
                }
                for (let i = 0; i < this.board.colSize + 1; i++) {
                    let tmpLine = new Graphics();
                    tmpLine.lineStyle(this.gridLineWidth, this.gridLineColor, 1);
                    tmpLine.moveTo(0, 0);
                    tmpLine.lineTo(0, boardHeight);
                    tmpLine.x = this.boardX + (i * (this.colWidth + this.gridLineWidth));
                    tmpLine.y = this.boardY;
                    this.app.stage.addChild(tmpLine);
                }
            },
            drawPiece: function (row, col, color) {
                let ellipse = new Graphics();
                ellipse.beginFill(color);
                ellipse.drawEllipse(0, 0, this.colWidth / 2, this.rowHeight / 2);
                ellipse.endFill();
                ellipse.x = this.boardY + col * (this.colWidth + this.gridLineWidth) + this.colWidth / 2;
                ellipse.y = this.boardX + row * (this.rowHeight + this.gridLineWidth) + this.rowHeight / 2;
                this.app.stage.addChild(ellipse);
            },
            eventPosToRowCol: function (x, y) {
                x -= this.boardX;
                y -= this.boardY;
                let res = {
                    row: -1,
                    col: -1
                }
                if (x % (this.colWidth + this.gridLineWidth) <= this.gridLineWidth) {
                    return res;
                }
                if (y % (this.rowHeight + this.gridLineWidth) <= this.gridLineWidth) {
                    return res;
                }
                res.row = Math.floor(y / (this.rowHeight + this.gridLineWidth));
                res.col = Math.floor(x / (this.colWidth + this.gridLineWidth));
                return res;
            }
        };


        let heOrShe = null;
        let heOrSheId = null;
        for (const id in this.currentRoom.game_players) {
            if (id !== (this.playerId).toString()) {
                heOrSheId = id;
                break;
            }
        }
        if (heOrSheId != null)
            heOrShe = { id: heOrSheId, type: PLAYER_TYPE.fromI32(this.currentRoom.game_players[heOrSheId]) }

        this.state = {
            winnerGot: false,
            heOrShe: heOrShe,
            observers: []
        }
    }

    componentDidMount() {
        this.wsClient.addEventListener("message", this.handleMessage);
        this.boardPainter.redrawBoard();
    }
    componentWillUnmount() {
        this.wsClient.removeEventListener("message", this.handleMessage);
    }

    handleMessage(ev) {
        let data = JSON.parse(ev.data);
        if ("type" in data) {
            switch (data.type) {
                case "reset_game":
                    // this.boardPainter.redrawBoard();
                    break;
                default:
                    break;
            }
        }
        if ("msg_others" in data) {
            let playerType = null;
            switch (data.msg_others) {
                case "enter_room":
                    playerType = PLAYER_TYPE.fromI32(data.data.player_type);
                    if (playerType === PLAYER_TYPE.OBSERVER) {
                        if (!(data.data.player_id in this.state.observers))
                            this.setState((prevState) => ({ observers: [data.data.player_id, ...prevState.observers] }));
                    } else if (data.data.player_id !== this.playerId) {
                        this.setState({
                            heOrShe: { id: data.data.player_id, type: playerType }
                        });
                    }
                    break;
                case "put_piece":
                    this.putPiece(data.data.row_i, data.data.col_j, data.data.piece_type);
                    break;
                case "reset":
                    this.boardPainter.redrawBoard();
                    break;
                case "exit_room":
                    playerType = PLAYER_TYPE.fromI32(data.data.player_type);
                    if (playerType === PLAYER_TYPE.OBSERVER) {
                        if (this.state.observers.indexOf(data.data.player_id) !== -1) {
                            this.setState((prevState) => {
                                let newObs = [];
                                for (let i = 0; i < prevState.observers.length; i++) {
                                    const v = prevState.observers[i];
                                    if (v !== data.data.player_id)
                                        newObs.push(v);
                                }
                                return { observers: newObs };
                            });
                        }
                    } else if (this.playerType !== PLAYER_TYPE.OBSERVER) {
                        alert("你的对手退出了游戏");
                    } else {
                        alert("选手已经退出游戏");
                    }
                    break;
                default:
                    break;
            }
        }
    }

    onBoardClick(ev) {
        if (!this.state.winnerGot) {
            let x = ev.data.global.x;
            let y = ev.data.global.y;
            let pos = this.boardPainter.eventPosToRowCol(x, y);
            if (pos.row !== -1 && this.board.isEmpty(pos.row, pos.col)) {
                sendMsg(this.wsClient, [
                    "put_piece",
                    (this.currentRoom.id).toString(),
                    (pos.row).toString(),
                    (pos.col).toString(),
                    this.pieceType.toString()]
                );
            }
        }
    }

    resetGame() { sendMsg(this.wsClient, ["reset_game", (this.currentRoom.id).toString()]); }

    putPiece(row, col, pieceType) {
        this.board.putPiece(row, col, this.pieceType);
        this.boardPainter.drawPiece(row, col, PIECE_TYPE.toColor(pieceType));
        this.winCheck();
    }

    winCheck() {

    }

    render() {
        return (
            <div style={{ display: 'flex' }}>
                <div style={{ display: 'flex', flexDirection: 'column' }}>
                    <div>当前房间: {this.currentRoom.name}</div>
                    {
                        this.state.heOrShe
                            ? <div>对手-{this.state.heOrShe.id}号选手:{PLAYER_TYPE.toStr(this.state.heOrShe.type)}</div>
                            : <></>
                    }
                    <div ref={(element) => {
                        if (element && element.childElementCount <= 0) {
                            this.pixiHolder = element;
                            this.pixiHolder.appendChild(this.pixiApp.view);
                        }
                    }}></div>
                    <div style={{ display: "flex" }}>
                        <div>我-{this.playerId}号选手:{PLAYER_TYPE.toStr(this.playerType)}</div>
                        <button onClick={(ev) => { this.resetGame(); }}>重开</button>
                        <button onClick={(ev) => { this.exitRoom(this.currentRoom); }}>退出</button>
                    </div>
                </div>
                <div>
                    <div>我:{this.playerId}号选手</div>
                    {this.state.heOrShe ? <div>对手:{this.state.heOrShe.id}号选手</div> : <></>}
                    {this.state.observers.map((v, i) => <div key={i}>观战者:{v}号选手</div>)}
                </div>
            </div>
        );
    }
};
export default GomokuGame;