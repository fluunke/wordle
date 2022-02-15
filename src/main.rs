mod error;
mod wordle;
use wordle::{Wordle, WordleSettings};

use crate::error::WordleError;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = ansi_term::enable_ansi_support().unwrap();

    let mut game = Wordle::new(WordleSettings::default());

    println!("{game}");

    loop {
        let mut guess = String::new();
        std::io::stdin().read_line(&mut guess)?;
        game.guess_word(guess);

        println!("{game}");

        if game.is_solved() {
            println!(
                "you won!\nthe word was: {}\nguessed in {}/{}",
                game.word(),
                game.guess_amount(),
                game.max_guesses()
            );
            break;
        }

        if game.failed() {
            println!("{}", WordleError::NoGuessesLeft { word: game.word() });
            break;
        }
    }

    Ok(())
}
