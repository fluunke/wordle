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
}
impl WordleSettings {
    pub fn default() -> Self {
        WordleSettings {
            word_length: 5,
            max_guesses: 5,
            word_list: WordList::BuiltIn,
        }
    }
}

pub struct Wordle {
    guesses: Vec<Vec<Guess>>,
    word: String,
    word_list: Vec<String>,
    settings: WordleSettings,
    solved: bool,
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

        let mut rng = rand::thread_rng();
        let word = word_list.choose(&mut rng).unwrap().to_string();

        let guesses = vec![];

        Wordle {
            word,
            guesses,
            settings,
            word_list,
            solved: false,
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

        if !self.valid_word(&guess) {
            eprintln!("{}", WordleError::NotAWord { word: &guess });
            return;
        }

        let mut try_word: Vec<Option<char>> = self.word.chars().map(Some).collect();
        let mut try_guess: Vec<Option<char>> = guess.chars().map(Some).collect();

        let mut completed_guess: Vec<Guess> = Vec::with_capacity(self.settings.word_length);

        // Initially, we mark every character as wrong...
        (0..self.settings.word_length).for_each(|i| {
            completed_guess.push(Guess {
                letter: try_guess[i].unwrap(),
                occurrence: Occurrence::Wrong,
            });
        });

        // ...then we mark letters as correct...
        for i in 0..self.settings.word_length {
            if try_guess[i] == try_word[i] {
                completed_guess[i] = Guess {
                    letter: try_word[i].unwrap(),
                    occurrence: Occurrence::Correct,
                };
                // remove letter from possibilities
                try_guess[i] = None;
                try_word[i] = None;
            }
        }
        // ...and finally we check if any leftover letters are present,
        // but in the wrong space.
        for i in 0..self.settings.word_length {
            if let Some(g) = try_guess[i] {
                if try_word.contains(&Some(g)) {
                    completed_guess[i] = Guess {
                        letter: g,
                        occurrence: Occurrence::Present,
                    };

                    // Get the actual position of the character to remove it
                    let position = try_word.iter().position(|&f| f == Some(g));

                    try_word[position.unwrap()] = None;
                    try_guess[i] = None;
                };
            }
        }

        // Are ya winning, son?
        self.check_win(&guess);

        self.guesses.push(completed_guess);
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

    pub fn is_failed(&self) -> bool {
        self.guesses.len() == self.max_guesses()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn big_test_file() {
        #[derive(serde::Deserialize, Debug, Clone)]
        struct TestCase {
            word: String,
            guess: String,
            correct: String,
        }

        let data = include_str!("../tests.txt");

        let mut rdr = csv::Reader::from_reader(data.as_bytes());

        for result in rdr.deserialize() {
            let line: TestCase = result.unwrap();

            // Create a new game with only the line's word
            let mut game = Wordle::new(WordleSettings {
                word_length: 5,
                max_guesses: 1,
                word_list: WordList::Custom(vec![line.word]),
            });

            game.guess_word(line.guess);

            for i in 0..game.settings.word_length {
                let occurrence = match line.correct.chars().nth(i).unwrap() {
                    'c' => Occurrence::Correct,
                    'w' => Occurrence::Wrong,
                    'm' => Occurrence::Present,
                    e => panic!("uhh what: {}", e),
                };

                // ensure every cell matches the test case of the file
                assert_eq!(game.get_cell(0, i).unwrap().occurrence, occurrence);
            }
        }
    }
}
