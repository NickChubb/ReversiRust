//use std::io;
//use rand::Rng;
/*
impl<T> std::fmt::Display for Vec<T> {
    fn fmt(&self, _: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        Ok(())
    }
}
*/

fn print_board(b: &Vec<u8>){

    let x = &b;

    for i in 0..7 {
        println!("{:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}", 
        x.get((i * 8) + 0),
        x.get((i * 8) + 1),
        x.get((i * 8) + 2),
        x.get((i * 8) + 3),
        x.get((i * 8) + 4),
        x.get((i * 8) + 5),
        x.get((i * 8) + 6),
        x.get((i * 8) + 7));
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
    let board_size = width * height;

    // Defines an initial board of 0s with size = width * height 
    let mut board: Vec<u8> = vec![0; board_size];

    println!("{:?}", board.get(0));

    //print_board(&board);

    /*
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
