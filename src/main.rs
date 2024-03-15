use rand::seq::SliceRandom;
use rand::thread_rng;
use std::io;
use std::collections::HashMap;

const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[0m"; // Resets the color to default

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PieceType {
    General,
    Advisor,
    Elephant,
    Chariot,
    Horse,
    Cannon,
    Soldier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Player {
    Red,
    Black,
}

#[derive(Debug, Clone, Copy)]
struct Piece {
    piece_type: PieceType,
    player: Player,
}

#[derive(Debug, Clone, Copy)]
enum Cell {
    Hidden(Option<Piece>),
    Revealed(Piece),
    Empty,
}

#[derive(Debug, Clone, Copy)]
enum ActionType {
    Flip { x: usize, y: usize },
    Move { from_x: usize, from_y: usize, to_x: usize, to_y: usize },
}

#[derive(Debug, Clone, Copy)]
struct GameMove {
    action_type: ActionType,
    piece: Option<Piece>, // Piece that was moved or flipped
    captured_piece: Option<Piece>, // Piece that was captured, if any
}

type Board = Vec<Vec<Cell>>;

fn init_board() -> Board {
    let mut pieces = Vec::new();

    // Populate the vector with two sets of pieces, one for each player
    for &player in &[Player::Red, Player::Black] {
        pieces.push(Piece { piece_type: PieceType::General, player });
        pieces.extend((0..2).map(|_| Piece { piece_type: PieceType::Advisor, player }));
        pieces.extend((0..2).map(|_| Piece { piece_type: PieceType::Elephant, player }));
        pieces.extend((0..2).map(|_| Piece { piece_type: PieceType::Chariot, player }));
        pieces.extend((0..2).map(|_| Piece { piece_type: PieceType::Horse, player }));
        pieces.extend((0..2).map(|_| Piece { piece_type: PieceType::Cannon, player }));
        pieces.extend((0..5).map(|_| Piece { piece_type: PieceType::Soldier, player }));
    }
    
    let mut rng = thread_rng();
    pieces.shuffle(&mut rng);

    // Initialize the board with hidden cells containing the pieces
    pieces
        .chunks(8)
        .map(|row| {
            row.iter()
                .map(|&piece| Cell::Hidden(Some(piece)))
                .collect::<Vec<Cell>>()
        })
        .collect::<Vec<_>>()
}

fn init_board_testing() -> Board {
    // Create a 4x8 board initialized with Empty cells
    let mut board = vec![vec![Cell::Empty; 8]; 4];

    // Setup for testing cannon and chariot movements and captures
    // - Cannons positioned to test jumping and capturing
    // - Chariots positioned to test movement and capturing
    // - Various pieces to act as targets or obstacles

    // Placing cannons for Red and Black
    board[3][1] = Cell::Revealed(Piece { piece_type: PieceType::Cannon, player: Player::Red }); // Bottom row, 2nd col
    board[0][1] = Cell::Revealed(Piece { piece_type: PieceType::Cannon, player: Player::Black }); // Top row, 2nd col

    // Placing chariots for Red and Black
    board[3][0] = Cell::Revealed(Piece { piece_type: PieceType::Chariot, player: Player::Red }); // Bottom row, 1st col
    board[0][0] = Cell::Revealed(Piece { piece_type: PieceType::Chariot, player: Player::Black }); // Top row, 1st col

    // Placing obstacles for cannons to jump over and targets for chariots
    board[2][1] = Cell::Revealed(Piece { piece_type: PieceType::Soldier, player: Player::Black }); // Cannon jump target
    board[1][0] = Cell::Revealed(Piece { piece_type: PieceType::Soldier, player: Player::Red }); // Chariot capture target

    board
}

fn flip_piece(board: &mut Board, x: usize, y: usize) -> Result<Option<GameMove>, &'static str> {
    if y >= board.len() || x >= board[0].len() {
        return Err("Coordinates out of bounds.");
    }
    
    match board[y][x] {
        Cell::Hidden(piece_option) => {
            if let Some(piece) = piece_option {
                board[y][x] = Cell::Revealed(piece);
                let game_move = GameMove {
                    action_type: ActionType::Flip { x, y },
                    piece: Some(piece),
                    captured_piece: None, // No piece is captured during a flip
                };
                Ok(Some(game_move))
            } else {
                Err("No piece to flip here.")
            }
        },
        _ => Err("Invalid flip action."),
    }
}

fn can_capture(attacker: Piece, defender: Piece) -> bool {
    use PieceType::*;

    match (attacker.piece_type, defender.piece_type) {
        // Handle the special case where Soldiers can capture Generals but not the other way around
        (Soldier, General) => true,
        (General, Soldier) => false,

        // Each piece captures pieces of the same type or lower rank, except for the special Soldier-General interaction
        // General is the highest rank and Soldier the lowest, with the order being General > Advisor > Elephant > Chariot > Horse > Cannon > Soldier
        // All pieces can capture lower-ranked pieces, except for the Soldier-General interaction
        // Cannon can capture every piece in the cannon capture jump but otherwise it can't attack anything
        // General rule: A piece can capture another piece of the same type or any type below it in the following order
        // For other cases, use a predefined order of power to determine capture ability
        _ => {
            let order = |piece_type: PieceType| -> i32 {
                match piece_type {
                    General => 7,
                    Advisor => 6,
                    Elephant => 5,
                    Chariot => 4, // Note: Chariot moves any number of spaces in a straight line, handled separately
                    Horse => 3,
                    Cannon => 2, // Note: Cannon's capturing rule needs board state, handled separately
                    Soldier => 1,
                }
            };

            // A piece can capture another piece of the same type or any type below it in the hierarchy
            order(attacker.piece_type) >= order(defender.piece_type)
        }
    }
}

fn is_valid_cannon_capture(board: &Board, from_x: usize, from_y: usize, to_x: usize, to_y: usize) -> bool {
    // Ensure movement is in a straight line and the target is a revealed piece
    if from_x != to_x && from_y != to_y || matches!(board[to_y][to_x], Cell::Empty | Cell::Hidden(_)) {
        return false;
    }

    let mut obstacles_encountered = 0;

    // Count obstacles in the path
    if from_x == to_x { // Vertical movement
        for y in (std::cmp::min(from_y, to_y) + 1)..std::cmp::max(from_y, to_y) {
            if !matches!(board[y][from_x], Cell::Empty) {
                obstacles_encountered += 1;
            }
        }
    } else { // Horizontal movement
        for x in (std::cmp::min(from_x, to_x) + 1)..std::cmp::max(from_x, to_x) {
            if !matches!(board[from_y][x], Cell::Empty) {
                obstacles_encountered += 1;
            }
        }
    }

    // Valid if exactly one obstacle is jumped over, regardless of its allegiance
    obstacles_encountered == 1 && matches!(board[to_y][to_x], Cell::Revealed(_))
}

fn is_valid_chariot_move_or_capture(board: &Board, from_x: usize, from_y: usize, to_x: usize, to_y: usize) -> bool {
    if from_x != to_x && from_y != to_y {
        return false; // Chariots must move straight.
    }

    let path_clear = if from_x == to_x {
        // Check vertical path
        (std::cmp::min(from_y, to_y) + 1..std::cmp::max(from_y, to_y)).all(|y| matches!(board[y][from_x], Cell::Empty))
    } else {
        // Check horizontal path
        (std::cmp::min(from_x, to_x) + 1..std::cmp::max(from_x, to_x)).all(|x| matches!(board[from_y][x], Cell::Empty))
    };

    path_clear && matches!(board[to_y][to_x], Cell::Revealed(_) | Cell::Empty) // Ensure path is clear and target is either empty or a revealed piece for capturing
}

fn valid_move_for_piece(piece: Piece, from_x: usize, from_y: usize, to_x: usize, to_y: usize, board: &Board) -> bool {
    match piece.piece_type {
        // Use the same logic for cannons and chariots for non-capturing moves.
        PieceType::Cannon | PieceType::Chariot => is_valid_chariot_move_or_capture(board, from_x, from_y, to_x, to_y),
        // Direct adjacent move for all other pieces.
        _ => (from_x as i32 - to_x as i32).abs() + (from_y as i32 - to_y as i32).abs() == 1,
    }
}

fn move_piece(board: &mut Board, from_x: usize, from_y: usize, to_x: usize, to_y: usize) -> Result<Option<GameMove>, &'static str> {
    if from_y >= board.len() || from_x >= board[0].len() || to_y >= board.len() || to_x >= board[0].len() {
        return Err("Coordinates out of bounds.");
    }

    match board[from_y][from_x] {
        Cell::Revealed(attacker) => {
            match board[to_y][to_x] {
                Cell::Hidden(_) | Cell::Empty => {
                    // Handle non-capturing moves
                    if valid_move_for_piece(attacker, from_x, from_y, to_x, to_y, board) {
                        let game_move = GameMove {
                            action_type: ActionType::Move { from_x, from_y, to_x, to_y },
                            piece: Some(attacker),
                            captured_piece: None,
                        };
                        board[to_y][to_x] = Cell::Revealed(attacker);
                        board[from_y][from_x] = Cell::Empty;
                        Ok(Some(game_move))
                    } else {
                        Err("Invalid move.")
                    }
                },
                Cell::Revealed(defender) => {
                    // Handle capturing moves
                    if attacker.player == defender.player {
                        return Err("Cannot capture your own piece.");
                    }

                    if (attacker.piece_type != PieceType::Cannon && can_capture(attacker, defender)) ||
                       (attacker.piece_type == PieceType::Cannon && is_valid_cannon_capture(board, from_x, from_y, to_x, to_y)) {
                        let game_move = GameMove {
                            action_type: ActionType::Move { from_x, from_y, to_x, to_y },
                            piece: Some(attacker),
                            captured_piece: Some(defender),
                        };
                        board[to_y][to_x] = Cell::Revealed(attacker);
                        board[from_y][from_x] = Cell::Empty;
                        Ok(Some(game_move))
                    } else {
                        Err("Cannot capture this piece.")
                    }
                },
                _ => Err("Invalid target."),
            }
        },
        _ => Err("No piece to move."),
    }
}

fn undo_last_move(board: &mut Board, moves_history: &mut Vec<GameMove>) -> Result<(), &'static str> {
    if let Some(last_move) = moves_history.pop() {
        match last_move.action_type {
            ActionType::Flip { x, y } => {
                // If the last action was a flip, simply hide the piece again.
                board[y][x] = Cell::Hidden(last_move.piece);
            },
            ActionType::Move { from_x, from_y, to_x, to_y } => {
                // If the last action was a move, move the piece back to its original position.
                let piece = last_move.piece.expect("A moved piece must exist.");
                board[from_y][from_x] = Cell::Revealed(piece);

                // If a piece was captured during the move, restore it to its position.
                // Otherwise, set the cell to empty.
                match last_move.captured_piece {
                    Some(captured_piece) => board[to_y][to_x] = Cell::Revealed(captured_piece),
                    None => board[to_y][to_x] = Cell::Empty,
                }
            }
        }
        Ok(())
    } else {
        Err("No moves to undo.")
    }
}

fn check_game_over(board: &Board) -> bool {
    let mut red_pieces = 0;
    let mut black_pieces = 0;
    let mut hidden_pieces = 0;
    let mut empty_cells = 0; // Counting empty cells for completeness
    
    for row in board {
        for cell in row {
            match cell {
                Cell::Hidden(_) => hidden_pieces += 1,
                Cell::Revealed(piece) => match piece.player {
                    Player::Red => red_pieces += 1,
                    Player::Black => black_pieces += 1,
                },
                Cell::Empty => empty_cells += 1,
            }
        }
    }

    // Do not end the game if there are still hidden pieces
    if hidden_pieces > 0 {
        return false;
    }

    // End the game if either player has no pieces left
    red_pieces == 0 || black_pieces == 0
}

fn parse_input(input: &str) -> Result<(String, Vec<usize>), &'static str> {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    let command = parts.get(0).ok_or("Missing command")?.to_string();

    let coordinates = parts[1..]
        .iter()
        .map(|&x| x.parse::<usize>())
        .collect::<Result<Vec<usize>, _>>()
        .map_err(|_| "Invalid coordinates")?;

    Ok((command, coordinates))
}
    
fn flip_all_pieces(board: &mut Board) {
    // For testing
    for row in board.iter_mut() {
        for cell in row.iter_mut() {
            if let Cell::Hidden(Some(piece)) = cell {
                *cell = Cell::Revealed(*piece);
            }
        }
    }
}

fn print_move_history(moves_history: &[GameMove], symbols: &HashMap<(Player, PieceType), &'static str>) {
    println!("Move History:");
    for (index, game_move) in moves_history.iter().enumerate() {
        let player = match game_move.piece {
            Some(piece) => piece.player,
            None => continue,
        };
        
        let player_symbol = match player {
            Player::Red => "Red",
            Player::Black => "Black",
        };

        let piece_symbol = game_move.piece.map_or("Unknown", |p| symbols.get(&(p.player, p.piece_type)).unwrap_or(&"Unknown"));
        
        let action_description = match game_move.action_type {
            ActionType::Flip { x, y } => format!("Flip at ({}, {})", x, y),
            ActionType::Move { from_x, from_y, to_x, to_y } => format!("Move from ({}, {}) to ({}, {})", from_x, from_y, to_x, to_y),
        };
        
        let capture_description = match game_move.captured_piece {
            Some(captured_piece) => format!(", captured {}", symbols.get(&(captured_piece.player, captured_piece.piece_type)).unwrap_or(&"Unknown")),
            None => String::new(),
        };

        println!("{}. {} {} made a {}{}", index + 1, player_symbol, piece_symbol, action_description, capture_description);
    }
}

fn print_game_state(board: &Board) {
    let symbols = piece_symbols(); // Retrieve the symbols mapping
    println!("Game State:");
    for row in board {
        let row_state: Vec<String> = row.iter().map(|cell| match cell {
            Cell::Hidden(_) => String::from("?"),
            Cell::Revealed(piece) => symbols.get(&(piece.player, piece.piece_type)).unwrap_or(&" ").to_string(),
            Cell::Empty => String::from("."),
        }).collect();

        // Join the cell states with a comma for readability
        println!("{}", row_state.join(", "));
    }
}

fn print_help() {
    println!("Available commands:");
    println!("  flip <row> <col>        - Flips a hidden piece at the specified coordinates.");
    println!("  move <from_row> <from_col> <to_row> <to_col> - Moves a piece from the starting coordinates to the destination coordinates.");
    println!("  undo                    - Undo the last move.");
    println!("  state                   - Prints the current game state in a simple text format.");
    println!("  history                 - Prints the move history.");
    println!("  exit                    - Exits the game.");
    println!("  flip all                - (For Testing) Flips all hidden pieces on the board.");

    println!("\nGameplay Instructions:");
    println!("  1. The game starts with all pieces hidden. Players take turns to either flip or move pieces.");
    println!("  2. Pieces can only move to adjacent squares unless specified by their type (e.g., Chariots and Cannons have special movement rules).");
    println!("  3. To capture an opponent's piece, move your piece to the same square. Capturing rules vary by piece type.");
    println!("  4. The winning condition is when the opponent has no legal moves left.");

    println!("\nPiece Capture Order and Special Rules:");
    println!("  - General (將/帥): Can be captured by Soldiers (卒/兵) but cannot capture Soldiers directly.");
    println!("  - Advisor (士/仕), Elephant (象/相), Chariot (車/俥), Horse (馬/傌): Capture pieces of the same or lower rank, except the Soldier-General interaction.");
    println!("  - Cannon (砲/炮): Can capture any piece by jumping over exactly one piece (of any type) along its movement path.");
    println!("  - Soldier (卒/兵): Can capture the General and pieces of the same or lower rank.");
    println!("  - The capture order from highest to lowest is General (將/帥) > Advisor (士/仕) > Elephant (象/相) > Chariot (車/俥) > Horse (馬/傌) > Cannon (砲/炮) > Soldier (卒/兵).");

    println!("\nSpecial Movement Rules:");
    println!("  - Chariot (車/俥): Moves and captures in any number of unblocked squares vertically or horizontally.");
    println!("  - Cannon (砲/炮): Moves like the Chariot but must jump over exactly one piece to capture any piece.");
    println!("\nNote: The game supports two players: Red and Black. Players must alternate turns.");
}

fn main() {
    // Initialize the game board
    let mut board = init_board();
    
    // Decide who starts the game, for simplicity we start with Red
    let mut current_player = Player::Red;

    // Game loop flag
    let mut game_over = false;

    // Tracks moves for undo functionality
    let mut moves_history: Vec<GameMove> = Vec::new();

    let symbols = piece_symbols();
    
    // Main game loop
    while !game_over {
        let mut turn_completed = false;

        while !turn_completed {
            // Display the board to the current player
            print_board(&board);
            
            // Prompt for player action
            println!("Player {:?}, enter your action (e.g., 'flip row col', 'move from_row from_col to_row to_col', 'undo', or 'exit'):", current_player);

            let mut action_input = String::new();
            io::stdin().read_line(&mut action_input).expect("Failed to read line");
            let trimmed_input = action_input.trim();

            // Check for the exit command
            match trimmed_input.to_lowercase().as_str() {
                "state" => print_game_state(&board),
                "history" => print_move_history(&moves_history, &symbols),
                "help" => print_help(),
                "exit" => {
                    println!("Exiting game.");
                    game_over = true;
                    break;
                },
                "flip all" => {
                    flip_all_pieces(&mut board);
                    println!("All pieces flipped for testing.");
                    turn_completed = true;
                },
                "undo" => {
                    if let Err(e) = undo_last_move(&mut board, &mut moves_history) {
                        println!("{}", e);
                    } else {
                        println!("Last move undone.");
                        // Switch back the player if undo was successful
                        current_player = match current_player {
                            Player::Red => Player::Black,
                            Player::Black => Player::Red,
                        };
                        turn_completed = false;
                    }
                },
                _ => {
                    // Handle action input
                    match parse_input(trimmed_input) {
                        Ok((command, coordinates)) => {
                            if command == "flip" && coordinates.len() == 2 {
                                match flip_piece(&mut board, coordinates[0], coordinates[1]) {
                                    Ok(Some(game_move)) => {
                                        moves_history.push(game_move); // Record the flip move
                                        println!("Piece flipped.");
                                        turn_completed = true;
                                    },
                                    Ok(None) => println!("No piece to flip here."),
                                    Err(e) => println!("Error: {}", e),
                                }
                            } else if command == "move" && coordinates.len() == 4 {
                                match move_piece(&mut board, coordinates[0], coordinates[1], coordinates[2], coordinates[3]) {
                                    Ok(Some(game_move)) => {
                                        moves_history.push(game_move); // Record the move
                                        println!("Piece moved.");
                                        turn_completed = true;
                                    },
                                    Ok(None) => println!("Invalid move."),
                                    Err(e) => println!("Error: {}", e),
                                }
                            } else {
                                println!("Invalid command or number of coordinates.");
                            }
                        },
                        Err(e) => println!("Error parsing input: {}", e),
                    }
                }
            }
        }

        if game_over {
            break;
        }

        // Check for game over condition after a valid turn
        game_over = check_game_over(&board);

        // Switch players if the turn was completed successfully and the game isn't over
        if !game_over {
            current_player = match current_player {
                Player::Red => Player::Black,
                Player::Black => Player::Red,
            };
        }
    }

    // Game is over, either by exit command or natural end
    println!("Game over. Thanks for playing!");
}

fn piece_symbols() -> HashMap<(Player, PieceType), &'static str> {
    use PieceType::*;
    use Player::*;

    let mut symbols = HashMap::new();

    symbols.insert((Red, General), "帥");
    symbols.insert((Black, General), "將");
    symbols.insert((Red, Advisor), "仕");
    symbols.insert((Black, Advisor), "士");
    symbols.insert((Red, Elephant), "相");
    symbols.insert((Black, Elephant), "象");
    symbols.insert((Red, Chariot), "俥");
    symbols.insert((Black, Chariot), "車");
    symbols.insert((Red, Horse), "傌");
    symbols.insert((Black, Horse), "馬");
    symbols.insert((Red, Cannon), "炮");
    symbols.insert((Black, Cannon), "砲");
    symbols.insert((Red, Soldier), "兵");
    symbols.insert((Black, Soldier), "卒");

    symbols
}

fn piece_symbols_eng() -> HashMap<(Player, PieceType), &'static str> {
    use PieceType::*;
    use Player::*;

    let mut symbols = HashMap::new();

    symbols.insert((Red, General), "RG");
    symbols.insert((Black, General), "BG");
    symbols.insert((Red, Advisor), "RA");
    symbols.insert((Black, Advisor), "BA");
    symbols.insert((Red, Elephant), "RE");
    symbols.insert((Black, Elephant), "BE");
    symbols.insert((Red, Chariot), "RC");
    symbols.insert((Black, Chariot), "BC");
    symbols.insert((Red, Horse), "RH");
    symbols.insert((Black, Horse), "BH");
    symbols.insert((Red, Cannon), "RC");
    symbols.insert((Black, Cannon), "BC");
    symbols.insert((Red, Soldier), "RS");
    symbols.insert((Black, Soldier), "BS");

    symbols
}

fn print_board(board: &Board) {
    let symbols: HashMap<(Player, PieceType), &str> = piece_symbols(); // Retrieve the symbol mapping

    // Print the column headers
    print!("   "); // Margin for row labels
    for x in 0..board[0].len() {
        print!(" {:^1} ", x); // Adjust to match the cell width
    }
    println!();

    // Print the top border of the board
    print!("  +"); // Start of the top border
    for _ in 0..board[0].len() {
        print!("--+"); // Top border for each cell, adjusted for double-width characters
    }
    println!();

    for (y, row) in board.iter().enumerate() {
        // Print the row numbers
        print!("{:<2}|", y); // Print row labels with space for alignment

        // Print each cell with the appropriate symbol
        for cell in row {
            let symbol = match cell {
                Cell::Hidden(_) => " ?".to_string(),
                Cell::Revealed(piece) => {
                    let piece_symbol = symbols.get(&(piece.player, piece.piece_type)).unwrap_or(&" ");
                    match piece.player {
                        Player::Red => format!("{}{}{}", RED, piece_symbol, RESET),
                        Player::Black => piece_symbol.to_string(),
                    }
                },
                Cell::Empty => "  ".to_string(),
            };
            print!("{}|", symbol); // Print the cell contents followed by a vertical separator
        }
        println!();

        // Print the horizontal separator for the board
        print!("  +"); // Start of the separator
        for _ in 0..row.len() {
            print!("--+"); // Separator for each cell, adjusted for double-width characters
        }
        println!(); // End the row
    }
}
