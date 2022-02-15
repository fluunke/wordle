use std::fmt::Display;

use crate::error::WordleError;

#[derive(PartialEq, Debug)]
enum Occurrence {
    Wrong,
    Present,
    Correct,
}

pub struct Guess {
    letter: char,
    occurrence: Occurrence,
}

#[derive(Clone)]
pub enum WordList {
    BuiltIn,
    Custom(Vec<String>),
}

pub struct WordleSettings {
    pub word_length: usize,
    pub max_guesses: usize,
    pub word_list: WordList,
    pub set_word: Option<String>,
}
impl WordleSettings {
    pub fn default() -> Self {
        WordleSettings {
            word_length: 5,
            max_guesses: 6,
            word_list: WordList::BuiltIn,
            set_word: None,
        }
    }
}

pub struct Wordle {
    guesses: Vec<Vec<Guess>>,
    word: String,
    word_list: Vec<String>,
    settings: WordleSettings,
    solved: bool,
    failed: bool,
}

impl Display for Wordle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for x in 0..self.settings.max_guesses {
            for y in 0..self.settings.word_length {
                let guess = self.get_cell(x, y);
                write!(f, "[{}]", self.print(guess))?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Wordle {
    pub fn new(settings: WordleSettings) -> Wordle {
        use rand::prelude::SliceRandom;

        let word_list: Vec<String> = match settings.word_list.clone() {
            WordList::BuiltIn => include_str!("../list")
                .lines()
                .into_iter()
                .map(|f| f.to_string())
                .collect(),
            WordList::Custom(list) => list,
        };

        let word = match settings.set_word.clone() {
            Some(word) => word,
            None => {
                let mut rng = rand::thread_rng();
                word_list.choose(&mut rng).unwrap().to_string()
            }
        };

        let guesses = vec![];

        Wordle {
            word,
            guesses,
            settings,
            word_list,
            solved: false,
            failed: false,
        }
    }
    pub fn guess_word(&mut self, guess: String) {
        let guess = guess.trim().to_lowercase();

        if guess.len() != self.settings.word_length {
            eprintln!(
                "{}",
                WordleError::WrongLength {
                    expected: self.settings.word_length
                }
            );
            return;
        }

        if self.guesses.len() >= self.settings.max_guesses - 1 {
            self.failed = true;
            return;
        }

        if !self.valid_word(&guess) {
            eprintln!("{}", WordleError::NotAWord { word: &guess });
            return;
        }

        let mut guesses = Vec::new();
        let mut word: Vec<char> = self.word.chars().collect();

        for (letter, true_letter) in guess.chars().zip(word.clone().iter()) {
            let occurrence = if &letter == true_letter {
                Occurrence::Correct
            } else if word.contains(&letter) {
                Occurrence::Present
            } else {
                Occurrence::Wrong
            };

            // broken! check double_letters test case
            for w in word.iter_mut() {
                if *w == letter {
                    *w = ' ';
                    break;
                }
            }

            guesses.push(Guess { letter, occurrence })
        }

        self.check_win(&guess);

        self.guesses.push(guesses);
    }

    pub(super) fn get_cell(&self, x: usize, y: usize) -> Option<&Guess> {
        use if_chain::if_chain;

        if_chain! {
            if let Some(guess) = self.guesses.get(x);
            if let Some(guess) = guess.get(y);
            then {
                 Some(guess)
            } else {
                None
            }
        }
    }

    /// Prints a guess in the correct color
    fn print(&self, guess: Option<&Guess>) -> String {
        use ansi_term::Colour;

        if guess.is_none() {
            return " ".to_string();
        }

        let guess = guess.unwrap();
        let letter = guess.letter.to_string();

        match guess.occurrence {
            Occurrence::Wrong => Colour::Red.paint(letter),
            Occurrence::Present => Colour::Yellow.paint(letter),
            Occurrence::Correct => Colour::Green.paint(letter),
        }
        .to_string()
    }

    fn valid_word(&self, word: &str) -> bool {
        // allow any word if we're in debug mode
        cfg!(not(debug)) || self.word_list.contains(&word.to_string())
    }

    fn check_win(&mut self, guess: &str) -> bool {
        if self.word == guess {
            self.solved = true;
            return true;
        }
        false
    }

    pub fn guess_amount(&self) -> usize {
        self.guesses.len()
    }
    pub fn max_guesses(&self) -> usize {
        self.settings.max_guesses
    }
    pub fn word(&self) -> &str {
        &self.word
    }

    pub fn is_solved(&self) -> bool {
        self.solved
    }

    pub fn failed(&self) -> bool {
        self.failed
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn words() -> Vec<String> {
        vec![
            "rises".to_string(),
            "sises".to_string(),
            "crabs".to_string(),
            "clone".to_string(),
        ]
    }
    fn new_game(words: &[String]) -> Wordle {
        Wordle::new(WordleSettings {
            max_guesses: 5,
            word_list: WordList::Custom(words.to_owned()),
            word_length: 5,
            set_word: Some(words.first().unwrap().to_string()),
        })
    }

    #[test]
    fn simple_game() {
        let words = words();
        let mut game = new_game(&words);

        for word in words {
            game.guess_word(word);
        }
        // game should be solved since we have guessed every single word in the wordlist
        assert!(game.is_solved());
    }

    #[test]
    fn occurrences() {
        let words = words();
        let mut game = new_game(&words);

        game.guess_word("sibel".to_string());
        assert_eq!(game.get_cell(0, 0).unwrap().occurrence, Occurrence::Present);
        assert_eq!(game.get_cell(0, 1).unwrap().occurrence, Occurrence::Correct);
        assert_eq!(game.get_cell(0, 2).unwrap().occurrence, Occurrence::Wrong);
        assert_eq!(game.get_cell(0, 3).unwrap().occurrence, Occurrence::Correct);
        assert_eq!(game.get_cell(0, 4).unwrap().occurrence, Occurrence::Wrong);
    }

    #[test]
    fn double_letters() {
        let words = words();
        let mut game = new_game(&words);

        game.guess_word("sises".to_string());
        assert_eq!(game.get_cell(0, 0).unwrap().occurrence, Occurrence::Wrong);
    }
}
