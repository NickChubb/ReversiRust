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
    player_available_actions: IndexSet<u8>,
    cpu_available_actions: IndexSet<u8>,
    player_turn: bool
}

/**
 * Board object functions
 */
impl Board {

    fn new(w: u8, h: u8) -> Board {

        let size = w * h;
        let mut player_actions: IndexSet<u8> = IndexSet::new();
        let mut cpu_actions: IndexSet<u8> = IndexSet::new();
        let mut new_board = vec![0; (size).into()];

        new_board[27] = 2;
        new_board[28] = 1;
        new_board[35] = 1;
        new_board[36] = 2;

        player_actions.insert(26);
        player_actions.insert(19);
        player_actions.insert(37);
        player_actions.insert(44);


        Board {
            width: w,
            height: h,
            board_size: size,
            board: new_board, //must convert u8 type -> usize type
            player_available_actions: player_actions,
            cpu_available_actions: cpu_actions,
            player_turn: true
        }
    }

    fn print(&mut self) {

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
                    print!("- ");
                }
            }
            count += 1; 
        }
        print!("{}\n\n", Style::default().bold().paint("8"));
    }

    fn ins(&mut self, pos: u8, val: u8, debug: bool) {

        // add to board
        let pos_u: usize = match self.get_available_actions().contains(&pos) {
            false => {
                println!("ERROR: not a valid action");
                return;
            },
            true => pos.into()
        };

        self.board.splice(pos_u..pos_u+1, [val].iter().cloned());

        let mut u: usize = 1;
        let mut tiles = Vec::new();
        
        // Iterate right
        loop {
            let position = (pos_u + u);
            let new_pos = match position % 8 {
                0 => break,
                _ => position
            };
            let tile = self.board.get(new_pos).unwrap();
            if tile != &val && tile != &0 {
                tiles.push(new_pos);
            } else if tile == &val {
                for t in &tiles {
                    self.add(*t, 1);
                }
            } else {
                tiles.clear();
                break;
            }
            u += 1;
        }

        // Iterate left
        u = 1;
        loop {
            let position = (pos_u - u);
            let new_pos = match position % 8 {
                7 => break,
                _ => position
            };
            let tile = self.board.get(new_pos).unwrap();
            if tile != &val && tile != &0 {
                tiles.push(new_pos);
            } else if tile == &val {
                for t in &tiles {
                    self.add(*t, 1);
                }
            } else {
                tiles.clear();
                break;
            }
            u += 1;
        }

        // Iterate down
        u = 1;
        loop {
            let position = pos_u + (u * 8);
            let new_pos = match position < self.board_size.into() {
                false => break,
                true => position
            };
            let tile = self.board.get(new_pos).unwrap();
            if tile != &val && tile != &0 {
                tiles.push(new_pos);
            } else if tile == &val {
                for t in &tiles {
                    self.add(*t, 1);
                }
            } else {
                tiles.clear();
                break;
            }
            u += 1;
        }
        
        // Iterate up
        u = 1;
        loop {
            let new_pos = match pos_u.checked_sub(u * 8) {
                None => break,
                Some(x) => Some(x).unwrap()
            };
            let tile = self.board.get(new_pos).unwrap();
            if tile != &val && tile != &0 {
                tiles.push(new_pos);
            } else if tile == &val {
                for t in &tiles {
                    self.add(*t, 1);
                }
            } else {
                tiles.clear();
                break;
            }
            u += 1;
        }

        // flip diagonal

        // update available actions
        self.player_available_actions.remove(&pos);
        self.cpu_available_actions.remove(&pos);
        self.update_surrounding_actions(pos, val, debug);

        // alternate turns
        if self.player_turn {
            self.player_turn = false
        }else {
            self.player_turn = true
        }
    }

    /**
     *  For inserted tile, will check if the tiles surrounding it are valid actions
     */
    fn update_surrounding_actions(&mut self, pos: u8, val: u8, debug: bool) {

        // Update top row
        for i in 0..3 {
            let new_pos: u8 = match pos.checked_sub(9 - i) { // overflow when on right most col
                None => continue,
                Some(x) => Some(x).unwrap()
            };
            let new_pos_u: usize = new_pos.into();
            let tile = self.board.get(new_pos_u).unwrap();
            if tile == &0 {
                for j in 1..3 {
                    self.check_tile_actions(new_pos, j, debug);
                }
            }
        }

        // Update middle row

        // Update bottom row
        for i in 0..3 {
            let new_pos: u8 = match (pos + 9 - i) < self.board_size.into() { // overflow when on right most col
                true => continue,
                false => pos + 9 - i
            };
            let new_pos_u: usize = new_pos.into();
            let tile = self.board.get(new_pos_u).unwrap();
            if tile == &0 {
                for j in 1..3 {
                    self.check_tile_actions(new_pos, j, debug);
                }
            }
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
                let new_pos: usize = match direction {

                    0 => { // Right
                        let position = pos + u;
                        if position % 8 == 0 {
                            break;
                        } else {
                            position.into()
                        }
                    },

                    1 => { // Left
                        let position = match pos.checked_sub(u) {
                            None => break,
                            Some(x) => Some(x).unwrap()
                        };
                        if position % 8 == 7 {
                            break;
                        } else {
                            position.into()
                        }
                    },

                    2 => { // Down
                        let position = pos + (u * 8);
                        if position < self.board_size.into() {
                            position.into()
                        } else {
                            break;
                        }
                    },

                    3 => { // Up
                        let new_pos = match pos.checked_sub(u * 8) {
                            None => break,
                            Some(x) => Some(x).unwrap()
                        }; 
                        new_pos.into()
                    },

                    _ => return
                };

                let tile = self.board.get(new_pos).unwrap(); // Gets value from tile at new position

                if tile != &val && tile != &0 {
                    // If the tile is not the same color as inserted, add to tiles vec
                    tiles.push(new_pos);
                } else if tile == &val && tiles.len() != 0 {
                    // If there is a tile the same color as the initial val with opposite tiles inbetween...
                    if val == 1 {
                        if debug {
                            println!("Added {} from actions for player {}", new_pos, val);
                        }
                        self.player_available_actions.insert(*tile);
                        tiles.clear();
                        break;
                    } else {
                        if debug {
                            println!("Removed {} from actions for player {}", new_pos, val);
                        }
                        self.cpu_available_actions.insert(*tile);
                        tiles.clear();
                        break;
                    }
                } else {
                    // Else, blank tile means not available action 
                    if debug {
                        println!("Removed {} from actions for player {}", pos, val);
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
     * Returns the IndexSet of available actions depending on which players turn it is
     * 
     * Should only use this function to get the available actions, don't individually
     * reference the player or cpu sets
     */
    fn get_available_actions(&mut self) -> &IndexSet<u8> {
        if self.player_turn {
            return &self.player_available_actions;
        } else {
            return &self.cpu_available_actions;
        }
    }

    /**
     * Add value at position on board
     * 
     * val = 0: unused square
     * val = 1: player piece
     * val = 2: cpu piece
     */
    fn add(&mut self, pos: usize, val: u8){
        self.board.splice(pos..(pos + 1), [val].iter().cloned());
    }

    fn rm(&mut self, pos: u8){
        let pos_u: usize = pos.into();
        self.board.insert(pos_u, 0);
        self.player_available_actions.insert(pos);
    }
    
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
 * Recursively solves a puzzle by MCTS
 */
fn monte_carlo_tree_search(b: Board, max_steps: usize, timer: usize) {

    let actions_size = b.player_available_actions.len();

    for i in 0..max_steps {

        let rand_index = rand::thread_rng().gen_range(0, actions_size);
        let rand_val = b.player_available_actions.get_index(rand_index);
        
        

        //monte_carlo_tree_search(b, max_steps, timer)
    }

}

fn main() {
    
    println!("Play a game of Reversi against AI!");

    const MAX_STEPS: usize = 100;
    const TIME: usize = 5;

    let width = 8;
    let height = 8;
    let mut board = Board::new(width, height);
    let re = Regex::new(r"([aA-hH][1-8])").unwrap();

    loop{

        board.print();

        println!("Place piece at position: ");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");

        let res: u8 = match re.is_match(&input) {
            true => convert_2d(&input),
            false => {println!("ERROR: invalid input"); continue},
        };
        
        let value = match board.player_turn {
            true => 1,
            false => 2
        };
        board.ins(res, value, true);

        //monte_carlo_tree_search(board, MAX_STEPS, TIME);

    }

}