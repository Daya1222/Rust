use rand::Rng;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    let mut rng = rand::rng();
    let n: u32 = rng.random_range(..10);

    let mut counter = 0;
    loop {
        counter += 1;
        let result = ask_input();
        match result {
            Ok(x) => {
                if x == n {
                    println!("You guessed the right number: {} in {} turns", n, counter);
                    break;
                } else if x > n {
                    println!("Your guess was higher");
                } else {
                    println!("Your guess was lower");
                }
            }
            _ => println!("Error occured"),
        }
    }

    println!("You guessed in {} turns", counter);
    Ok(())
}

fn ask_input() -> io::Result<u32> {
    let mut input = String::new();

    print!("Enter your guess: ");
    io::stdout().flush()?; // flush to show the prompt immediately

    io::stdin().read_line(&mut input)?; // read input
    let x: u32 = input
        .trim()
        .parse::<u32>()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid number"))?;
    Ok(x)
}
