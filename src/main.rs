//use std::io;
//use rand::Rng;

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
            board: vec![0; size.into()] //must convert u8 size -> usize type
        }
    }

    fn print(&mut self) {
        for i in 0..self.height.into() {
            println!("{:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}", 
            self.board.get((i * 8) + 0),
            self.board.get((i * 8) + 1),
            self.board.get((i * 8) + 2),
            self.board.get((i * 8) + 3),
            self.board.get((i * 8) + 4),
            self.board.get((i * 8) + 5),
            self.board.get((i * 8) + 6),
            self.board.get((i * 8) + 7));
        }
    }

    fn ins(&mut self, pos: usize, val: u8) {
        self.board.insert(pos, val)
    }
}

/**
 * Recursively solves a puzzle by MCTS
 */
fn monte_carlo_tree_search(board: &Vec<u8>, action: u8) {

}

fn main() {
    
    println!("Welcome to MCTS Reversi Solver!");

    let width = 8;
    let height = 8;

    let mut board = Board::new(width, height);

    board.ins(5, 7);
    board.print();


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
