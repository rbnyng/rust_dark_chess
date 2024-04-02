# Chinese Dark Chess in Rust

## Overview
This is a version of the Chinese Dark Chess strategy board game developed in Rust. It is a variant of traditional Chinese Chess (Xiangqi) with elements of hidden information and uncertainty. The game supports two players, Red and Black, who compete to capture the opponent's pieces and win the game.

## Features
- Console-based gameplay.
- Hidden piece positions until revealed by players.
- Piece capture mechanics following traditional Chinese Chess rules.
- Commands for flipping, moving, and capturing pieces.
- Special movement and capture rules for different piece types.

## Getting Started

### Prerequisites
- Rust programming language setup on your system.
- Basic familiarity with compiling and running Rust applications.

### Installation
1. Clone this repository to your local machine:
    ```sh
    git clone https://github.com/rbnyng/rust_dark_chess
    ```
2. Navigate to the project directory:
    ```sh
    cd rust_dark_chess
    ```

### Running the Game

To compile and run the game, use the following command from the project directory:
    ```sh
    cargo run
    ```

Or you can build it into an executable with:
    ```sh
    cargo build --release
    ```

Alternatively there is a precompiled executable available.

## Gameplay Instructions

- The game starts with all pieces hidden. Players take turns to flip or move pieces.
- Pieces can only move to adjacent squares unless specified by their type (e.g., Chariots and Cannons have special movement rules).
- To capture an opponent's piece, move your piece to the same square. Capturing rules vary by piece type.
- The objective is to capture the opponent's General or to leave the opponent with no legal moves.

### Commands

- `flip <row> <col>`: Flips a hidden piece at the specified coordinates.
- `move <from_row> <from_col> <to_row> <to_col>`: Moves a piece from the starting coordinates to the destination coordinates.
- `exit`: Exits the game.
- `help`: Displays a help message with game instructions and commands.
- `undo`: Undo the last move.
- `state`: Prints the current game state in a simple text format.
- `history`: Prints the move history.
- `flip all`: (For Testing) Flips all hidden pieces on the board.

### Piece Capture Order and Special Rules

- General (將/帥): Can be captured by Soldiers (卒/兵) but cannot capture Soldiers directly.
- Advisor (士/仕), Elephant (象/相), Horse (馬/傌): Capture pieces of the same or lower rank, except the Soldier-General interaction.
- Chariot (車/俥): Moves and captures in any number of unblocked squares vertically or horizontally.
- Cannon (砲/炮): Moves like the Chariot (車/俥), can capture any piece by jumping over exactly one piece (of any type) along its movement path.
- Soldier (卒/兵): Can capture the General and pieces of the same or lower rank.
- The capture order from highest to lowest is General (將/帥) > Advisor (士/仕) > Elephant (象/相) > Chariot (車/俥) > Horse (馬/傌) > Cannon (砲/炮) > Soldier (卒/兵).

Note: The game supports two players: Red and Black. Players must alternate turns.

## License

This project is open source and available under the [MIT License](LICENSE).


