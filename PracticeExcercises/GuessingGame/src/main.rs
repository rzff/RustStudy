use rand::Rng;
use std::cmp::Ordering;
use std::io;

fn main() {
    let secret_number = rand::thread_rng().gen_range(1..=100);
    println!("The secret nmbrs = {secret_number}");
    println!("Please input a guess");

    let mut guess = String::new();

    io::stdin()
        .read_line(&mut guess)
        .expect("Failed to read line");

    println!("You guessed: : {guess}");

    match guess.cmp(&secret_number) {
        Ordering::Less => println!("Small"),
        Ordering::Equal => println!("Perfect!"),
        Ordering::Greater => println!("Big"),
    }
}
