use std::io;
//use rand::Rng;
use regex::Regex;

/* TO DISPLAY PRINT PROPERLY IMPLEMENT LATER
impl<T> std::fmt::Display for Vec<T> {
    fn fmt(&self, _: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        Ok(())
    }
}
*/

struct Board {
    width: u8,
    height: u8,
    board_size: u8,
    board: Vec<u8>
}

impl Board {

    fn new(w: u8, h: u8) -> Board {

        let size = w * h;

        Board {
            width: w,
            height: h,
            board_size: size,
            board: vec![0; (size).into()] //must convert u8 type -> usize type
        }
    }

    fn print(&mut self) {
        // Super hacky, find nicer way to do this
        let mut count = 0;
        for i in self.board.iter(){
            if count % self.width == 0 {
                print!("\n");
            }
            print!("{} ", i);
            count += 1;
        }
        println!();
    }

    fn ins(&mut self, pos: usize, val: u8) {
        self.board.splice(pos..pos+1, [val].iter().cloned());
    }

    fn rm(&mut self, pos: usize){
        self.board.insert(pos, 0)
    }

    fn flip(&mut self, pos: usize) {
        if self.board[pos] == 1 {
            self.board.insert(pos, 2)
        }else if self.board[pos] == 2 {
            self.board.insert(pos, 1)
        }
    }
}

fn convert_2d(s: &str) -> usize{
    // pattern match a - h, 1- 8
    // ()

    let letter = s.chars().next().unwrap().to_ascii_lowercase();
    let num = s.chars().nth(1).unwrap();

    let col: usize = match letter {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        'h' => 7,
        _ => 42
    };

    // Probably better way to do this.... but I couldn't find it
    let row: usize = match num {
        '1' => 0,
        '2' => 1,
        '3' => 2,
        '4' => 3,
        '5' => 4,
        '6' => 5,
        '7' => 6,
        '8' => 7,
        _ => 42
    };

    row * 8 + col
    
}

/**
 * Recursively solves a puzzle by MCTS
 */
fn monte_carlo_tree_search(board: Board, action: u8) {

}

fn main() {
    
    println!("Welcome to MCTS Reversi Solver!");

    let width = 8;
    let height = 8;
    let mut board = Board::new(width, height);
    let re = Regex::new(r"([aA-hH][1-8])").unwrap();

    loop{

        board.print();

        println!("Place piece at position: ");

        let mut input = String::new();

        io::stdin().read_line(&mut input).expect("Failed to read line");

        let res: usize = match re.is_match(&input) {
            true => convert_2d(&input),
            false => {println!("ERROR: invalid input"); continue},
        };

        board.ins(res, 1);


    }



    /* CODE FROM TUTORIAL, IGNORE ONLY FOR REFERENCE LOL
    loop {
        println!("Please input your guess");

        let mut guess = String::new();

        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read line");

        let guess: u32 = match guess.trim().parse() {
	    Ok(num) => num,
	    Err(_) => continue,
	};

        println!("You guessed: {}", guess);

        match guess.cmp(&secret_number) {
            Ordering::Less => println!("Too small!"),
            Ordering::Greater => println!("Too big!"),
            Ordering::Equal => {
		println!("You win!");
		break;
	    }
        }
    }
    */

}
