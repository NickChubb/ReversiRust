use std::io;
use rand::Rng;
use regex::Regex;

// IndexSet provides an indexed HashSet to allow returning element by index
// Used for getting random items from set in O(1) time so MCTS is more efficient
// Docs: https://docs.rs/indexmap/1.5.0/indexmap/set/struct.IndexSet.html
use indexmap::IndexSet;

// Pretty board styling
use ansi_term::Color::{Red, Green};
use ansi_term::Style;

struct Board {
    width: u8,
    height: u8,
    board_size: u8,
    board: Vec<u8>,
    perimeter: IndexSet<u8>,
    player_available_actions: IndexSet<u8>,
    cpu_available_actions: IndexSet<u8>,
    player_turn: bool
}

/**
 * Board object functions
 */
impl Board {

    /**
     * Initializes a Reversi game board
     * 
     * board elements are u8 integers, which represent:
     *      0 => empty square
     *      1 => player
     *      2 => cpu
     */
    fn new(w: u8, h: u8) -> Board {

        let size = w * h;
        let mut player_actions: IndexSet<u8> = IndexSet::new();
        let mut cpu_actions: IndexSet<u8> = IndexSet::new();
        let mut perimeter_tiles: IndexSet<u8> = IndexSet::new();
        let mut new_board = vec![0; (size).into()];
        
        new_board[28] = 1;
        new_board[35] = 1;
        new_board[27] = 2;
        new_board[36] = 2;

        player_actions.insert(26);
        player_actions.insert(19);
        player_actions.insert(37);
        player_actions.insert(44);

        cpu_actions.insert(29);
        cpu_actions.insert(20);
        cpu_actions.insert(34);
        cpu_actions.insert(43);

        perimeter_tiles.insert(18);
        perimeter_tiles.insert(19);
        perimeter_tiles.insert(20);
        perimeter_tiles.insert(21);
        perimeter_tiles.insert(26);
        perimeter_tiles.insert(29);
        perimeter_tiles.insert(34);
        perimeter_tiles.insert(37);
        perimeter_tiles.insert(42);
        perimeter_tiles.insert(43);
        perimeter_tiles.insert(44);
        perimeter_tiles.insert(45);

        Board {
            width: w,
            height: h,
            board_size: size,
            board: new_board, //must convert u8 type -> usize type
            perimeter: perimeter_tiles,
            player_available_actions: player_actions,
            cpu_available_actions: cpu_actions,
            player_turn: true // Player always takes the first turn
        }
    }

    // fn copy(b: &Board) -> Board {

    //     Board {
    //         width: b.width,
    //         height: b.height,
    //         board_size: b.board_size,
    //         board: b.board,
    //         perimeter: b.perimeter,
    //         player_available_actions: b.player_available_actions,
    //         cpu_available_actions: b.cpu_available_actions,
    //         player_turn: b.player_turn // Player always takes the first turn
    //     }
    // }

    /**
     * Print the board vec to the screen
     * 
     * Players tiles are printed in RED
     * CPUs tiles are printed in GREEN
     */
    fn print(&mut self, debug: bool) {

        println!("\n     {}", Style::default().bold().paint("A B C D E F G H") );

        let mut count = 0;
        for i in self.board.iter(){
            if count % self.width == 0 {
                if count != 0 {
                    let row_num: u8 = count / 8;
                    print!("{}\n     ", Style::default().bold().paint(row_num.to_string()));
                }else{  
                    print!("     ")
                }
            }
            if i == &1 {
                print!("{} ", Red.paint("●"));
            } else if i == &2 {
                print!("{} ", Green.paint("●"));
            } else {
                if self.player_available_actions.contains(&count) {
                    print!("{} ", Style::default().bold().paint("*"));
                } else {
                    if debug && self.cpu_available_actions.contains(&count) {
                        print!("{} ", Style::default().bold().paint("+"));
                    } else {
                        print!("- ");
                    }
                    
                }
            }
            count += 1; 
        }
        print!("{}\n\n", Style::default().bold().paint("8"));
    }

    /**
     * Handles a piece being put onto the board
     * 
     * Adds to board -> flips pieces -> update perimeter -> updates available actions -> change turns
     */
    fn ins(&mut self, pos: u8, val: u8, debug: bool) {

        // add to board
        let pos_u: usize = match self.get_available_actions(debug).contains(&pos) {
            false => {
                println!("ERROR: not a valid action");
                return;
            },
            true => pos.into()
        };

        self.board.splice(pos_u..pos_u+1, [val].iter().cloned());

        let mut u: u8 = 1;
        let mut tiles = Vec::new();

        // Manages the direction of iteration
        for direction in 0..8 {

            u = 1;
            tiles.clear();

            loop {

                // Depending on direction, changes the formula for iteration
                let new_pos: u8 = match get_new_pos(direction, pos, u, self.board_size) {
                    None => break,
                    Some(x) => Some(x).unwrap()
                };

                let new_pos_usize: usize = new_pos.into();

                let tile = self.board.get(new_pos_usize).unwrap();

                if tile != &val && tile != &0 {
                    tiles.push(new_pos);
                } else if tile == &val {
                    for t in &tiles {
                        self.add(*t, val);
                    }
                } else {
                    tiles.clear();
                    break;
                }
                
                u += 1;
            }
        }

        // Update perimeter
        self.perimeter.remove(&pos);
        
        for i in 0..3 {
            let new_pos: u8 = match pos.checked_sub(9 - i) {
                None => continue,
                Some(x) => Some(x).unwrap()
            };
            let new_pos_usize: usize = new_pos.into();
            if self.board.get(new_pos_usize).unwrap() == &0 { // implement row overflow handling
                self.perimeter.insert(new_pos);
            }
        }
        
        match pos.checked_sub(1) {
            Some(x) => {
                let new_pos = Some(x).unwrap();
                let new_pos_usize: usize = Some(x).unwrap().into();
                if self.board.get(new_pos_usize).unwrap() == &0 {
                    self.perimeter.insert(new_pos);
                }
            },
            None => {
                if debug {
                    println!("Overflow, but it's chill, I handled it")
                }
            }
        };

        match pos + 1 < self.board_size {
            true => {
                let new_pos = pos + 1;
                let new_pos_usize: usize = new_pos.into();
                if self.board.get(new_pos_usize).unwrap() == &0 {
                    self.perimeter.insert(new_pos);
                }
            },
            false => {
               if debug {
                   println!("Overflow, but it's chill, I handled it")
               }
            }
        }

        for i in 0..3 {
            let new_pos: u8 = pos + 9 - i;
            let new_pos_usize: usize = new_pos.into();
            if new_pos < self.board_size {
                if self.board.get(new_pos_usize).unwrap() == &0 {
                    self.perimeter.insert(new_pos);
                }
            }
        }

        if debug {
            println!("{:?}", self.perimeter);
        }

        // update available actions
        self.player_available_actions.remove(&pos);
        self.cpu_available_actions.remove(&pos);

        for player in 1..3 {
            for tile in self.get_perimeter(debug) {
                self.check_tile_actions(tile, player, debug);
            }
        }

        // alternate turns
        if self.player_turn {
            if debug {
                println!("CPU's turn");
            }
            self.player_turn = false
        }else {
            if debug {
                println!("Player's turn");
            }
            self.player_turn = true
        }

        if debug {
            println!("Player's Available Actions: {:?}", self.get_player_actions(debug));
            println!("CPU's Available Actions: {:?}", self.get_cpu_actions(debug));
        }
    }

    /**
     * Given a tile position it will check in all directions if it is an available option 
     * for player with the input val (1 or 2)
     */
    fn check_tile_actions(&mut self, pos: u8, val: u8, debug: bool){

        let pos_u: usize = pos.into();

        let mut u: u8 = 1;
        let mut tiles = Vec::new();

        // Manages the direction of iteration
        for direction in 0..8 {

            u = 1;
            tiles.clear();

            loop {

                // Depending on direction, changes the formula for iteration
                let new_pos: u8 = match get_new_pos(direction, pos, u, self.board_size) {
                    None => break,
                    Some(x) => Some(x).unwrap()
                };

                let new_pos_usize: usize = new_pos.into();

                let tile = self.board.get(new_pos_usize).unwrap(); // Gets value from tile at new position

                if tile != &val && tile != &0 {
                    // If the tile is not the same color as inserted, add to tiles vec
                    tiles.push(new_pos);
                } else if tile == &val && tiles.len() != 0 {
                    // If there is a tile the same color as the initial val with opposite tiles inbetween...
                    if val == 1 {
                        if debug {
                            println!("Added {} to actions for player {}", new_pos, val);
                        }
                        self.player_available_actions.insert(pos);
                        tiles.clear();
                        return;
                    } else {
                        if debug {
                            println!("Added {} to actions for player {}", new_pos, val);
                        }
                        self.cpu_available_actions.insert(pos);
                        tiles.clear();
                        return;
                    }
                } else {
                    // Else, blank tile means not available action 
                    if debug {
                        //println!("Removed {} from actions for player {}", pos, val);
                    }
                    if val == 1 {
                        self.player_available_actions.remove(&pos);
                    } else {
                        self.cpu_available_actions.remove(&pos);
                    }

                    tiles.clear();
                    break;
                }
                u += 1;

            }
        }
    }

    /**
     * Returns a clone of the IndexSet of available actions depending on which players turn it is
     * 
     * Should only use this function to get the available actions, don't individually
     * reference the player or cpu sets
     */
    fn get_available_actions(&self, debug: bool) -> IndexSet<u8> {
        if self.player_turn {
            let actions: IndexSet<u8> = IndexSet::clone(&self.player_available_actions);
            if debug {
                println!("Player Available Actions: {:?}", actions);
            }
            return actions;
        } else {
            let actions: IndexSet<u8> = IndexSet::clone(&self.cpu_available_actions);
            if debug {
                println!("CPU Available Actions: {:?}", actions);
            }
            return actions;
        }
    }

    fn get_player_actions(&self, debug: bool) -> IndexSet<u8> {
        IndexSet::clone(&self.player_available_actions)
    }

    fn get_cpu_actions(&self, debug: bool) -> IndexSet<u8> {
        IndexSet::clone(&self.cpu_available_actions)
    }

    fn get_direction(&self) -> u8 {
        42
    }

    /**
     * Returns IndexSet of the tiles in the perimeter of the board pieces
     */
    fn get_perimeter(&self, debug: bool) -> IndexSet<u8> {
        IndexSet::clone(&self.perimeter)
    }

    /**
     * Add value at position on board
     * 
     * val = 0: unused square
     * val = 1: player piece
     * val = 2: cpu piece
     */
    fn add(&mut self, pos: u8, val: u8){
        let pos_u: usize = pos.into();
        self.board.splice(pos_u..(pos_u + 1), [val].iter().cloned());
    }

    fn rm(&mut self, pos: u8){
        let pos_u: usize = pos.into();
        self.board.insert(pos_u, 0);
        self.player_available_actions.insert(pos);
    }
    
}

/** 
 * Returns a new position based on direction, initial pos, iteration, and board size
 * Intended to be used in a loop (such as in the Board.ins() function)
 * 
 * @returns: Some(x) if new position is on board, or
 * @returns: None if position overflows board
 */
fn get_new_pos(dir: u8, pos: u8, iter: u8, size: u8) -> Option<u8> {
    let new_pos: Option<u8> = match dir {

        0 => { // Right
            let position = pos + iter;
            if position % 8 == 0 {
                None
            } else {
                Some(position)
            }
        },

        1 => { // Left
            let position = match pos.checked_sub(iter) {
                None => None,
                Some(x) => {
                    if Some(x).unwrap() % 8 == 7 {
                        None
                    } else {
                        Some(x)
                    }
                }
            };
            position
        },

        2 => { // Down
            let position = pos + (iter * 8);
            if position < size {
                Some(position)
            } else {
                None
            }
        },

        3 => { // Up
            let new_pos = match pos.checked_sub(iter * 8) {
                None => None,
                Some(x) => Some(x)
            };
            new_pos
        },

        4 => { // Up left: must check that doesn't % 8 = 7 and doesn't overflow
            let new_pos = match pos.checked_sub(iter * 8 + iter) {
                None => None,
                Some(x) => {
                    if Some(x).unwrap() % 8 != 7 {
                        Some(x)
                    } else {
                        None
                    }
                }
            }; 
            new_pos
        },

        5 => { // Up right: must check that doesn't % 8 = 0 and doesn't overflow
            let new_pos = match pos.checked_sub(iter * 8 - iter) {
                None => None,
                Some(x) => {
                    if Some(x).unwrap() % 8 != 0 {
                        Some(x)
                    } else {
                        None
                    }
                }
            };
            new_pos
            
        },

        6 => { // Down left: must check that doesnt % 8 = 7 and 
            let position = pos + (iter * 8) - iter;
            if position < size && position % 8 != 7 {
                Some(position)
            } else {
                None
            }
        },

        7 => { // Down left: must check that doesnt % 8 = 7 and 
            let position = pos + (iter * 8) + iter;
            if position < size && position % 8 != 0 {
                Some(position)
            } else {
                None
            }
        },

        _ => None
    };

    new_pos
}

/**
 * Convert 2d string index to vector index
 * @params:     s: &str - len 2 string of char A-H followed by int 1-8
 * @returns:    u8 position in 1d Vec
 */
fn convert_2d(s: &str) -> u8{

    //Handle panic
    let letter = s.chars().next().unwrap().to_ascii_lowercase();
    let num = s.chars().nth(1).unwrap();

    let col: u8 = match letter {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        'h' => 7,
        _   => 42
    };

    // Probably better way to do this.... but I couldn't find it
    let row: u8 = match num {
        '1' => 0,
        '2' => 1,
        '3' => 2,
        '4' => 3,
        '5' => 4,
        '6' => 5,
        '7' => 6,
        '8' => 7,
        _   => 42
    };

    row * 8 + col
}

/**
 * Convert integer vector index into 2d string index
 * Note: this function is the inverse of convert_2d()
 * @params:     num: less than 64 valued integer representing 1d index of vector
 * @returns:    String of values [a-h][1-8]
 */
fn convert_num(num: u8) -> String {

    let letter: &str = match num / 8 {
        0 => "a",
        1 => "b",
        2 => "c",
        3 => "d",
        4 => "e",
        5 => "f",
        6 => "g",
        7 => "h",
        _ => "x"
    };

    format!("{}{}", letter, num % 8 + 1)
}

fn print_help() {
    println!("\nCommands:\n");
    println!("  {}  -  print the current available actions", Style::default().bold().paint("actions"));
    println!("  {}    -  toggles showing debug information", Style::default().bold().paint("debug"));
    println!("  {}     -  quit the game", Style::default().bold().paint("exit"));
    println!();
}

fn toggle_debug(mut debug: bool) -> bool{
    if debug {
        println!("Debug turned OFF");
        false
    } else {
        println!("Debug turned ON");
        true
    }
}

fn print_actions(actions: IndexSet<u8>) {
    print!("\nPlayer's Actions: ");
    for action in actions {
        print!("{} ", Style::default().bold().paint(convert_num(action)));
    }
    println!("\n");
}

// Returns true if game has ended, and false otherwise
fn check_game_state(b: &Board) -> bool {
    return true;
}

/**
 * Recursively solves a puzzle by MCTS
 */
 fn monte_carlo_tree_search(b: &Board, max_steps: usize, timer: usize, debug: bool) -> u8 {
    
    
    let test = 20;
    let mut stats: [u8; 3] = [0; 3];
    

    println!("CPU calculating {} random playouts", max_steps);
    for i in 0..max_steps {
        
        for action in b.get_available_actions(true) {
            let playout = random_playout(b, action, &stats);
        }

    }

    return test;
}

fn random_playout(b: &Board, action: u8, stats: &[u8; 3]) {
    let playout_board = b.clone();
    let counter = 0;
    println!("action: {}", action);

    // while !check_game_state(&playout_board) {
    //     if counter % 2 == 0 { // even: CPU's Turn
    //         let actions_size = playout_board.cpu_available_actions.len();
    //         let rand_index = rand::thread_rng().gen_range(0, actions_size);
    //         let rand_val = playout_board.cpu_available_actions.get_index(rand_index).unwrap();
    //         playout_board.ins(*rand_val, 2, true)
    //     }
        
    //     else { // odd: Player's Turn
    //         let actions_size = playout_board.player_available_actions.len();
    //         let rand_index = rand::thread_rng().gen_range(0, actions_size);
    //         let rand_val = playout_board.player_available_actions.get_index(rand_index).unwrap();
    //         playout_board.ins(*rand_val, 1, true)
    //     }

    // }
    


}

fn user_input() {

}

fn main() {
    
    println!("\nPlay a game of Reversi against AI!");

    const MAX_STEPS: usize = 5;
    const TIME: usize = 5;

    let width = 8;
    let height = 8;
    let mut board = Board::new(width, height);
    let re = Regex::new(r"([aA-hH][1-8])").unwrap();

    let mut debug = false;

    loop{

        board.print(true);

        if board.player_turn == true {
            println!("Place piece at position: ");
            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("Failed to read line");
            
            match re.is_match(&input) {
                true => {
                    let input_u8: u8 = convert_2d(&input);
                    board.ins(input_u8, 1, true);
                }
                false => {
                    match input.as_str() {
                        "help\n" => {
                            print_help();
                            continue
                        },
                        "debug\n" => {
                            debug = toggle_debug(debug);
                            continue
                        },
                        "actions\n" => {
                            print_actions(board.get_player_actions(debug));
                            continue
                        }
                        "exit\n" => break,
                        _ => {
                            println!("ERROR: invalid input, enter 'help' for command information"); 
                            continue
                        }
    
                    };
                }
            };
        }

        else {
            let best_play: u8 = monte_carlo_tree_search(&board, MAX_STEPS, TIME, true);
            println!("CPU found {} as best play", best_play);
            board.ins(best_play, 2, true)
        }     


    } // loop
}