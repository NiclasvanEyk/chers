import type { State, Coordinate, Color, PromotedFigure } from "@/generated/chers/chers";

export type Move = {
  from: Coordinate;
  to: Coordinate;
  promotion: PromotedFigure | null;
};

export type User = {
  id: string;
  name: string;
};

export type ClientMessage =
  | { type: "authenticate"; secret: string; name: string }
  | { type: "command"; payload: Command };

export type ServerMessage =
  | { type: "event"; payload: Event }
  | { type: "state"; payload: RoomStateMirror }
  | { type: "error"; reason: string };

export type Command =
  | { Lobby: LobbyCommand }
  | { Game: GameCommand }
  | { PostGame: PostGameCommand }
  | { RequestState: { user: User } };

export type LobbyCommand =
  | { Join: { secret: string; name: string } }
  | { Leave: { user: User } }
  | { ChangeName: { user: User; new_name: string } }
  | { ChangeReady: { user: User; is_ready: boolean } };

export type GameCommand =
  | { Reconnect: { secret: string } }
  | { Leave: { user: User } }
  | { MakeMove: { user: User; move_: Move } }
  | { Resign: { user: User } };

export type PostGameCommand =
  | { Reconnect: { secret: string } }
  | { Leave: { user: User } }
  | { OfferRematch: { user: User } }
  | { AcceptRematch: { user: User } }
  | { DeclineRematch: { user: User } };

export type Event =
  | { Lobby: LobbyEvent }
  | { Game: GameEvent }
  | { PostGame: PostGameEvent }
  | { System: SystemEvent };

export type LobbyEvent =
  | { PlayerJoined: { user: User } }
  | { PlayerNameChanged: { user: User } }
  | { PlayerReadyChanged: { user: User; is_ready: boolean } }
  | { PlayerLeft: { user: User } }
  | { JoinRejected: { user: User; reason: string } };

export type GameEvent =
  | { GameStarted: { white: User; black: User } }
  | { Turn: { author: User; move_: Move } }
  | { GameEnded: { winner: User | null; reason: GameEndReason } }
  | { PlayerReconnected: { user: User } }
  | { PlayerLeft: { user: User } }
  | { MoveRejected: { author: User; reason: string } };

export type PostGameEvent =
  | { PlayerReconnected: { user: User } }
  | { PlayerLeft: { user: User } }
  | { RematchOffered: { by: User } }
  | { RematchAccepted: { by: User } }
  | { RematchDeclined: { by: User } };

export type SystemEvent =
  | { CommandRejected: { user: User; reason: string } };

export type RoomStateMirror =
  | { Lobby: { you: User; opponent: User | null; you_are_ready: boolean; opponent_is_ready: boolean } }
  | { Game: { you: User; your_color: Color; opponent: User; opponent_color: Color; board_state: State; is_your_turn: boolean } }
  | { PostGame: { you: User; your_color: Color; opponent: User; opponent_color: Color; winner: Color | null; you_won: boolean; reason: GameEndReason; final_board: State } };

export type GameEndReason =
  | "Checkmate"
  | "Stalemate"
  | "Resignation"
  | "DrawAgreement"
  | "FiftyMoveRule"
  | "InsufficientMaterial"
  | "ThreefoldRepetition"
  | "Timeout"
  | "Abandoned";
