use std::io;
use rand::Rng;
use regex::Regex;
//use math::round;

// HashMap is used in order to assign a count to each element inside win/draw/loss stats
use std::collections::HashMap;

// IndexSet provides an indexed HashSet to allow returning element by index
// Used for getting random items from set in O(1) time so MCTS is more efficient
// Docs: https://docs.rs/indexmap/1.5.0/indexmap/set/struct.IndexSet.html
use indexmap::IndexSet;

// Used to limit MCTS duration
use std::time::{Duration, Instant};

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

    fn clone(&self) -> Board {
      
        let mut new_board: Board = Board {
            width: self.width,
            height: self.height,
            board_size: self.board_size,
            board: self.board.clone(),
            perimeter: self.perimeter.clone(),
            player_available_actions: self.player_available_actions.clone(),
            cpu_available_actions: self.cpu_available_actions.clone(),
            player_turn: self.player_turn // Player always takes the first turn
        };

        new_board
    }

    /**
     * Print the board vec to the screen
     * 
     * Players tiles are printed in RED
     * CPUs tiles are printed in GREEN
     */
    fn print(&self, debug: bool) {

        let (player_score, cpu_score): (u8, u8) = self.get_score();

        println!("\n     {}", Style::default().bold().paint("A B C D E F G H") );

        let mut count = 0;
        for i in self.board.iter() {
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
                    print!("- "); 
                }
            }
            count += 1; 
        }
        print!("{}\n\n", Style::default().bold().paint("8"));

        println!("     Player: {}, CPU: {}", Red.paint(player_score.to_string()), Green.paint(cpu_score.to_string()));

        print_actions(self.get_player_actions(debug));
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
                println!("ERROR: {} is not a valid action", pos);
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
            let actions = self.get_player_actions(debug);
            if debug {
                println!("Player Available Actions: {:?}", actions);
            }  
            actions
        } else {
            let actions = self.get_cpu_actions(debug);
            if debug {
                println!("CPU Available Actions: {:?}", actions);
            }
            actions
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

    fn is_player_turn(&self) -> bool {
        self.player_turn
    }

    /**
     * Returns IndexSet of the tiles in the perimeter of the board pieces
     */
    fn get_perimeter(&self, debug: bool) -> IndexSet<u8> {
        IndexSet::clone(&self.perimeter)
    }

    fn check_game_state(&self, debug: bool) -> u8 {
        let player_actions = self.get_player_actions(debug);
        let cpu_actions = self.get_cpu_actions(debug);

        // GAME ENDED
        if cpu_actions.len() == 0 || player_actions.len() == 0 {
            
            // 0 incomplete
            // 1 player win
            // 2 cpu win
            // 3 draw            

            let (player_score, cpu_score): (u8, u8) = self.get_score();

            if debug {
                println!("  Player: {}, CPU: {}", Red.paint(player_score.to_string()), Green.paint(cpu_score.to_string()));
            }
            
            if player_score > cpu_score {
                return 1;
            } else if cpu_score > player_score {
                return 2;
            } else {
                return 3;
            }
        }

        else { 0 }
    
    }
    /**
     * get_score() -> returns tuple containing current score for player and cpu
     */
    fn get_score(&self) -> (u8, u8) {
        let mut count_player = 0;
        let mut count_cpu = 0;

        for i in 0..64 {
            match self.board.get(i).unwrap() {
                0 => continue,
                1 => count_player += 1,
                2 => count_cpu += 1,
                _ => println!("Error Code: ID10T" )
            }
        }

        (count_player, count_cpu)
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

    let val: f64 = (num / 8).into();

    let letter: &str = match val.floor() {
        0f64 => "A",
        1f64 => "B",
        2f64 => "C",
        3f64 => "D",
        4f64 => "E",
        5f64 => "F",
        6f64 => "G",
        7f64 => "H",
        _ => "x"
    };

    format!("{}{}", letter, num % 8 + 1)
}

fn print_title() {
    println!("################################################################");
    println!("#                                                              #");
    println!("#                {}                #", Style::default().bold().paint("Welcome to Reversi against AI!"));
    println!("#                                                              #");
    println!("################################################################\n\n");
}

fn print_help() {
    println!("\nCommands:\n");
    println!("  {}  -  print the current available actions", Style::default().bold().paint("actions"));
    println!("  {}  -  show game rules", Style::default().bold().paint("rules"));
    println!("  {}    -  toggles showing debug information", Style::default().bold().paint("debug"));
    println!("  {}     -  quit the game", Style::default().bold().paint("exit"));
    println!();
}

fn print_actions(actions: IndexSet<u8>) {
    print!("\nPlayer's Actions: ");
    for action in actions {
        print!("{} ", Style::default().bold().paint(convert_num(action)));
    }
    println!("\n");
}

fn print_rules(){
    println!("      #                {}                #\n", Style::default().bold().paint("REVERSI RULES"));
    println!(" * {} tiles represent the user's spots, {} represent the CPUs.\n", Red.paint("Red"), Green.paint("Green"));
    println!(" * The user starts by placing a tile adjacent to a green tile.\n Possible actions are marked by asterisks (*) on the board.\n");
    println!(" * The game ends when either player cannot play a piece or the\n board is full.  The player with the most tiles wins.\n");
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

/**
 * Simplified Monte Carlo Tree Search which performs random playouts until completion 
 * and records the win/draw/loss statistics for each available action at current board state.
 *  Parameters:
 *      b              -    the current board state to initialize the playout board
 *      max_steps      -    maximum number of iterations 
 *      timer          -    maximum amount of time to spend during the mcts in seconds
 *      debug          -    used to print extra debug statements
 * 
 */
 fn monte_carlo_tree_search(mut b: &Board, max_steps: usize, timer: usize, diff: &String, debug: bool) -> u8 {

    let mut stats: [Vec<u8>; 3] = [vec![], vec![], vec![]];
    let start_time = Instant::now();
    
    println!("CPU calculating {} random playouts...", max_steps);
    
    for i in 0..max_steps {
        if start_time.elapsed() > Duration::new(timer as u64, 0) { break }
        
        let actions = b.get_available_actions(debug);

        if debug { println!("{:?}", actions); }
        
        for action in actions {

            let mut playout_board: Board = b.clone();

            match random_playout(&mut playout_board, action, diff, debug) {
                1 => stats[1].push(action),
                2 => stats[0].push(action),
                3 => stats[2].push(action),
                _ => continue
            };
        }
    }

    // Populate hashmap with frequency of elements in win list
    let mut a = HashMap::new();
    for i in stats[0].iter() {
        if a.contains_key(i) {
            *(a.get_mut(&i).unwrap()) += 1;
        } else {
            a.insert(i, 1);
        }
    }

    if debug {
        println!("Player wins: {:?}", stats[1]);
        println!("CPU wins: {:?}", stats[0]);
        println!("Draws: {:?}", stats[2]);
        for (pos, wins) in &a {
            println!("{}: {}", pos, wins);
        } 
    }

    // Returns the highest value in frequency hashmap as best play
    **a.iter().max_by(|a, b| a.1.cmp(&b.1)).map(|(k, _v)| k).unwrap()
}


/**
*   Performs random playouts 
*/
fn random_playout(mut b: &mut Board, action: u8, diff: &String, debug: bool) -> u8 {
    
    if debug { println!("Playing action: {}", action); }

    // Play a game until completion
    loop {
        match b.check_game_state(debug) {
            0 => { // Game not done
                if !b.player_turn { 
                    let actions = b.get_cpu_actions(debug);
                    let actions_size = actions.len();

                    match diff.as_str() {
                        "1" => {
                            let rand_index = rand::thread_rng().gen_range(0, actions_size);
                            let rand_val = actions.get_index(rand_index).unwrap();
                            b.ins(*rand_val, 2, debug);
                        },
                        
                        "2" => {
                            let new_val = get_max_tile(b, debug);

                            if new_val == 99 {
                                continue;
                            }
                                // Someone ran out of moves
                            if debug { println!("new_val: {}", new_val); }
                            b.ins(new_val, 2, debug);
                        }
                        _ => println!("How did you get here?")
                    };
                    
                   
                }

                else {
                    let actions = b.get_player_actions(debug);
                    let actions_size = actions.len();
                    let rand_index = rand::thread_rng().gen_range(0, actions_size);
                    let rand_val = actions.get_index(rand_index).unwrap();
                    b.ins(*rand_val, 1, debug);     
                }

                if debug { b.print(debug); }
                continue;
            },
            1 => return 1, // Player Wins
            2 => return 2, // CPU Wins
            3 => return 3, // Draw
            _ => return 42
        };
    }
}

/**
 * Max Tile Heuristic - Returns the position that results in the highest
 *                      score out of all possible actions
 */
fn get_max_tile(mut b: &Board, debug: bool) -> u8 {

    let actions = b.get_available_actions(debug);

    if actions.len() == 0 {
        return 99;
    }

    if debug { println!("{:?}", actions); }
    
    let (prev_player_score, prev_cpu_score): (u8, u8) = b.get_score();

    let best_score = prev_cpu_score;
    let mut best_pos: u8 = 0;


    for action in actions {
        // check increase in value of tiles
        let mut new_board: Board = b.clone();
        
        new_board.ins(action, 2, debug);

        let (player_score, cpu_score): (u8, u8) = new_board.get_score();

        if cpu_score > best_score {
            best_pos = action;
        }
        
    }
    
    best_pos
}


fn initial_user_input() -> (String, String, String) {

    println!("Select a Game Mode: ");
    println!("[1] Player VS CPU");
    println!("[2] CPU VS CPU");
    println!("*Note: CPU VS CPU might take a while. To stop the playout use 'Ctrl' + 'C'\n");
    let mut mode = String::new();
    let mut cpu_diff = [String::new(), String::new()];

    io::stdin().read_line(&mut mode).expect("Failed to read line");
    match mode.as_str().trim() {
        "1" => {
            println!("Select CPU Difficulty: ");
            println!("[1] Easy");
            println!("[2] Hard\n");
            io::stdin().read_line(&mut cpu_diff[0]).expect("Failed to read line");
        },
        "2" => {
            for i in 0..2 {
                println!("Select CPU-{} Difficulty: ", i + 1);
                println!("[1] Easy");
                println!("[2] Hard\n");
                io::stdin().read_line(&mut cpu_diff[i]).expect("Failed to read line");
            }
        }
        _ => println!("Invalid input. Expected integer 1 or 2")
    };

    (mode.trim().to_string(), cpu_diff[0].trim().to_string(), cpu_diff[1].trim().to_string())
}

fn main() {
    
    print_title();
    print_rules();
    let game_settings = initial_user_input();
    println!("Mode: {} CPU-1 Diff: {} CPU-2 Diff: {}", game_settings.0, game_settings.1, game_settings.2);

    const MAX_STEPS: usize = 10;
    const TIME: usize = 100; 

    let width = 8;
    let height = 8;
    let mut board = Board::new(width, height);
    let re = Regex::new(r"([aA-hH][1-8])").unwrap();
    let mut debug = true;

    // Player VS CPU
    if game_settings.0 == "1" {
        loop{
            match board.check_game_state(debug) {
                1 => {
                    println!("Player has won");
                    break;
                },
                2 => {
                    println!("CPU has won");
                    break;
                },
                3 => {
                    println!("Game is a draw");
                    break;
                },
                _ => ()
            };
    
            board.print(true);
    
            if board.is_player_turn() == true {
                println!("Place piece at position: ");
                let mut input = String::new();
                io::stdin().read_line(&mut input).expect("Failed to read line");
                
                // Validate input string
                match re.is_match(&input) {
                    true => {
                        let input_u8: u8 = convert_2d(&input);
                        board.ins(input_u8, 1, debug);
                    },
    
                    false => {
                        match input.as_str() {
                            "help\n" => {
                                print_help();
                                continue;
                            },
                            "actions\n" => {
                                print_actions(board.get_player_actions(debug));
                                continue;
                            },
                            "rules\n" => {
                                print_rules();
                                continue;
                            }
                            "debug\n" => {
                                debug = toggle_debug(debug);
                                continue;
                            },
                            "exit\n" => break,
                            _ => {
                                println!("ERROR: invalid input, enter 'help' for command information"); 
                                continue;
                            }
        
                        };
                    }
                };
            }
    
            else {
                let best_play: u8 = monte_carlo_tree_search(&board, MAX_STEPS, TIME, &game_settings.1, debug);
                println!("CPU found {} as best play", best_play);
                board.ins(best_play, 2, debug);
            }     
    
        } // loop
    }

    // CPU VS CPU
    else {

        let mut start: bool = true; 
        
        loop {

            match board.check_game_state(debug) {
                1 => {
                    println!("CPU-1 has won");
                    break;
                },
                2 => {
                    println!("CPU-2 has won");
                    break;
                },
                3 => {
                    println!("Game is a draw");
                    break;
                },
                _ => ()
            };

            board.print(true);
            
            if start == true {
                println!("PRESS ENTER TO START CPU BATTLE: ");
                let mut input = String::new();
                io::stdin().read_line(&mut input).expect("Failed to read line");
                match input.as_str() {
                    "\n" => {
                        start = false;
                        continue;
                    }
                    "help\n" => {
                        print_help();
                        continue;
                    },
                    "actions\n" => {
                        print_actions(board.get_player_actions(debug));
                        continue;
                    },
                    "rules\n" => {
                        print_rules();
                        continue;
                    }
                    "debug\n" => {
                        debug = toggle_debug(debug);
                        continue;
                    },
                    "exit\n" => break,
                    _ => {
                        println!("ERROR: invalid input, enter 'help' for command information"); 
                        continue;
                    }

                };
                
            }

            // Assume CPU-1 is considered player for CPU vs CPU.
            match board.is_player_turn() {
                true => {
                    let best_play: u8 = monte_carlo_tree_search(&board, MAX_STEPS, TIME, &game_settings.1, debug);
                    println!("CPU-1 found {} as best play", best_play);
                    board.ins(best_play, 1, debug);    
                },

                false => {
                    let best_play: u8 = monte_carlo_tree_search(&board, MAX_STEPS, TIME, &game_settings.2, debug);
                    println!("CPU-2 found {} as best play", best_play);
                    board.ins(best_play, 2, debug);    
                }
            };
        }
    }

    
}