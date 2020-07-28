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
    available_actions: IndexSet<u8>,
    player_turn: bool
}

/**
 * Board object functions
 */
impl Board {

    fn new(w: u8, h: u8) -> Board {

        let size = w * h;
        let mut actions: IndexSet<u8> = IndexSet::new();
        for i in 0..size {
            actions.insert(i);
        }

        let mut new_board = vec![0; (size).into()];
        new_board[27] = 1;
        new_board[28] = 2;
        new_board[35] = 2;
        new_board[36] = 1;

        Board {
            width: w,
            height: h,
            board_size: size,
            board: new_board, //must convert u8 type -> usize type
            available_actions: actions,
            player_turn: true
        }
    }

    fn print(&mut self) {
        // Super hacky, find nicer way to do this
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
                print!("- ");
            }
            count += 1; 
        }
        print!("{}\n\n", Style::default().bold().paint("8"));
    }

    fn ins(&mut self, pos: u8, val: u8) {

        // alternate turns
        if self.player_turn {
            self.player_turn = false
        }else {
            self.player_turn = true
        }

        // add to board
        let pos_u: usize = pos.into();
        self.board.splice(pos_u..pos_u+1, [val].iter().cloned());
        self.available_actions.remove(&pos);

        // flip

    }

    fn rm(&mut self, pos: u8){
        let pos_u: usize = pos.into();
        self.board.insert(pos_u, 0);
        self.available_actions.insert(pos);
    }

    fn flip(&mut self, pos: usize) {
        if self.board[pos] == 1 {
            self.board.insert(pos, 2)
        }else if self.board[pos] == 2 {
            self.board.insert(pos, 1)
        }
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

    let actions_size = b.available_actions.len();

    for i in 0..max_steps {

        let rand_index = rand::thread_rng().gen_range(0, actions_size);
        let rand_val = b.available_actions.get_index(rand_index);
        
        

        //monte_carlo_tree_search(b, max_steps, timer)
    }

}

fn main() {
    
    println!("Welcome to MCTS Reversi Solver!");

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

        board.ins(res, 1);

        //monte_carlo_tree_search(board, MAX_STEPS, TIME);

    }

}