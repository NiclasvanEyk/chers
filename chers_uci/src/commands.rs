// NOTE: The documentation is copied from the UCI specification

/// Represents commands sent from the GUI to the chess engine.
enum Request {
    /// uci
    ///
    /// Tell engine to use the uci (universal chess interface),
    /// this will be sent once as a first command after program boot
    /// to tell the engine to switch to uci mode.
    /// After receiving the uci command the engine must identify itself with the "id" command
    /// and send the "option" commands to tell the GUI which engine settings the engine supports if any.
    /// After that the engine should send "uciok" to acknowledge the uci mode.
    /// If no uciok is sent within a certain time period, the engine task will be killed by the GUI.
    Uci,

    /// debug [ on | off ]
    ///
    /// Switch the debug mode of the engine on and off.
    /// In debug mode the engine should send additional infos to the GUI, e.g. with the "info string" command,
    /// to help debugging, e.g. the commands that the engine has received etc.
    /// This mode should be switched off by default and this command can be sent
    /// any time, also when the engine is thinking.
    Debug(bool),

    /// isready
    ///
    /// This is used to synchronize the engine with the GUI. When the GUI has sent a command or
    /// multiple commands that can take some time to complete,
    /// this command can be used to wait for the engine to be ready again or
    /// to ping the engine to find out if it is still alive.
    /// E.g. this should be sent after setting the path to the tablebases as this can take some time.
    /// This command is also required once before the engine is asked to do any search
    /// to wait for the engine to finish initializing.
    /// This command must always be answered with "readyok" and can be sent also when the engine is calculating
    /// in which case the engine should also immediately answer with "readyok" without stopping the search.
    IsReady,

    /// setoption name <id> [value <x>]
    ///
    /// This is sent to the engine when the user wants to change the internal parameters
    /// of the engine. For the "button" type no value is needed.
    /// One string will be sent for each parameter and this will only be sent when the engine is waiting.
    /// The name and value of the option in <id> should not be case sensitive and can include spaces.
    /// The substrings "value" and "name" should be avoided in <id> and <x> to allow unambiguous parsing,
    /// for example do not use <name> = "draw value".
    /// Here are some strings for the example below:
    ///    "setoption name Nullmove value true\n"
    ///    "setoption name Selectivity value 3\n"
    ///    "setoption name Style value Risky\n"
    ///    "setoption name Clear Hash\n"
    ///    "setoption name NalimovPath value c:\chess\tb\4;c:\chess\tb\5\n"
    SetOption { name: String, value: Option<String> },

    /// register
    ///
    /// This is the command to try to register an engine or to tell the engine that registration
    /// will be done later. This command should always be sent if the engine has sent "registration error"
    /// at program startup.
    /// The following tokens are allowed:
    /// * later
    ///    the user doesn't want to register the engine now.
    /// * name <x>
    ///    the engine should be registered with the name <x>
    /// * code <y>
    ///    the engine should be registered with the code <y>
    /// Example:
    ///    "register later"
    ///    "register name Stefan MK code 4359874324"
    Register(Vec<String>),

    /// ucinewgame
    ///
    /// This is sent to the engine when the next search (started with "position" and "go") will be from
    /// a different game. This can be a new game the engine should play or a new game it should analyze but
    /// also the next position from a testsuite with positions only.
    /// If the GUI hasn't sent a "ucinewgame" before the first "position" command, the engine shouldn't
    /// expect any further ucinewgame commands as the GUI is probably not supporting the ucinewgame command.
    /// So the engine should not rely on this command even though all new GUIs should support it.
    /// As the engine's reaction to "ucinewgame" can take some time the GUI should always send "isready"
    /// after "ucinewgame" to wait for the engine to finish its operation.
    UciNewGame,

    /// position [fen <fenstring> | startpos ]  moves <move1> .... <movei>
    ///
    /// Set up the position described in fenstring on the internal board and
    /// play the moves on the internal chessboard.
    /// If the game was played from the start position, the string "startpos" will be sent.
    /// Note: no "new" command is needed. However, if this position is from a different game than
    /// the last position sent to the engine, the GUI should have sent a "ucinewgame" in between.
    Position {
        fen: Option<String>,
        moves: Vec<String>,
    },

    /// go
    ///
    /// Start calculating on the current position set up with the "position" command.
    /// There are a number of commands that can follow this command, all will be sent in the same string.
    /// If one command is not sent its value should be interpreted as it would not influence the search.
    /// * searchmoves <move1> .... <movei>
    ///    restrict search to these moves only.
    /// * ponder
    ///    start searching in pondering mode.
    /// * wtime <x>
    ///    white has x msec left on the clock
    /// * btime <x>
    ///    black has x msec left on the clock
    /// * winc <x>
    ///    white increment per move in mseconds if x > 0
    /// * binc <x>
    ///    black increment per move in mseconds if x > 0
    /// * movestogo <x>
    ///    there are x moves to the next time control,
    ///    this will only be sent if x > 0,
    ///    if you don't get this and get the wtime and btime it's sudden death
    /// * depth <x>
    ///    search x plies only.
    /// * nodes <x>
    ///    search x nodes only,
    /// * mate <x>
    ///    search for a mate in x moves
    /// * movetime <x>
    ///    search exactly x mseconds
    /// * infinite
    ///    search until the "stop" command. Do not exit the search without being told so in this mode!
    Go {
        search_moves: Vec<String>,
        ponder: bool,
        wtime: Option<u32>,
        btime: Option<u32>,
        winc: Option<u32>,
        binc: Option<u32>,
        movestogo: Option<u32>,
        depth: Option<u32>,
        nodes: Option<u64>,
        mate: Option<u32>,
        movetime: Option<u32>,
        infinite: bool,
    },

    /// stop
    ///
    /// Stop calculating as soon as possible, don't forget the "bestmove" and possibly the "ponder" token
    /// to indicate which move the engine would play.
    Stop,

    /// ponderhit
    ///
    /// This will be sent if the engine was told to ponder on the same move the user has played.
    /// The engine should continue searching but switch from pondering to normal search.
    PonderHit,

    /// quit
    ///
    /// Quit the program as soon as possible.
    Quit,
}

/// Represents responses sent from the chess engine to the GUI.
enum Response {
    /// id
    ///
    /// This must be sent after receiving the "uci" command to identify the engine.
    /// Example: "id name Shredder X.Y\n"
    Id { name: String },

    /// uciok
    ///
    /// Must be sent after the "id" and optional options to tell the GUI that the engine
    /// has sent all info and is ready in uci mode.
    UciOk,

    /// readyok
    ///
    /// This must be sent when the engine has received an "isready" command and has
    /// processed all input and is ready to accept new commands now.
    /// It can be used anytime, even when the engine is searching.
    ReadyOk,

    /// bestmove <move1> [ ponder <move2> ]
    ///
    /// The engine has stopped searching and found the move <move1> best in this position.
    /// The engine can send the move it likes to ponder on, but it must not start pondering automatically.
    /// This command must always be sent if the engine stops searching.
    BestMove {
        best_move: String,
        ponder: Option<String>,
    },

    /// copyprotection
    ///
    /// This is needed for copy-protected engines. The engine can tell the GUI that it will check the copy protection now.
    /// After checking, the engine should send either "copyprotection ok" or "copyprotection error."
    CopyProtection,

    /// registration
    ///
    /// This is needed for engines that need a username and/or a code to function with all features.
    /// After checking registration, the engine should send either "registration ok" or "registration error."
    Registration,

    /// info
    ///
    /// The engine wants to send information to the GUI. This should be done whenever one of the info has changed.
    /// The engine can send only selected info or multiple infos with one info command.
    /// See the UCI specification for various info fields.
    Info(Vec<String>),

    /// option
    ///
    /// This command tells the GUI which parameters can be changed in the engine.
    /// The GUI should parse this and build a dialog for the user to change the settings.
    /// See the UCI specification for details on option parameters.
    Option(Vec<String>),
}
