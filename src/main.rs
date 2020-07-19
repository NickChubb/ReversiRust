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
            board: vec![0; (size - 1).into()] //must convert u8 type -> usize type
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
    }

    fn ins(&mut self, pos: usize, val: u8) {
        self.board.insert(pos, val)
    }

    fn remove(&mut self, pos: usize){
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
