use thiserror::Error;

#[derive(Error, Debug)]
pub enum WordleError<'a> {
    #[error("you have no guesses left\n the word was: {word}")]
    NoGuessesLeft { word: &'a str },
    #[error("your guess should be {expected:?} characters long")]
    WrongLength { expected: usize },
    #[error("\"{word}\" is not a valid word")]
    NotAWord { word: &'a str },
}
