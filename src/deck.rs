use std::mem;

use rand::{seq::SliceRandom, thread_rng};
use serde::Serialize;

#[derive(Serialize, Copy, Clone, PartialEq)]
pub enum Card {
    Clubs(u8),
    Diamonds(u8),
    Hearts(u8),
    Spade(u8),
    Joker,
}

impl Card {
    pub fn get_value(&self) -> Option<u8> {
        match self {
            Card::Clubs(n) | Card::Diamonds(n) | Card::Hearts(n) | Card::Spade(n) => Some(*n),
            Card::Joker => None,
        }
    }
    pub fn get_score(&self) -> i8 {
        match self {
            Card::Hearts(13) => -1,
            Card::Joker => 0,
            Card::Clubs(n) | Card::Diamonds(n) | Card::Hearts(n) | Card::Spade(n) => *n as i8,
        }
    }
}

pub struct Deck {
    cards: Vec<Card>,
    discard_pile: Vec<Card>,
}

impl Default for Deck {
    fn default() -> Self {
        Self::new()
    }
}

impl Deck {
    pub fn new() -> Self {
        let mut cards = vec![];
        for i in 1..=13 {
            cards.push(Card::Clubs(i));
            cards.push(Card::Diamonds(i));
            cards.push(Card::Hearts(i));
            cards.push(Card::Spade(i));
        }

        cards.push(Card::Joker);
        cards.push(Card::Joker);

        cards.shuffle(&mut thread_rng());
        Deck {
            cards,
            discard_pile: vec![],
        }
    }

    pub fn draw(&mut self) -> Card {
        if self.cards.is_empty() {
            mem::swap(&mut self.cards, &mut self.discard_pile);
        }
        self.cards.pop().unwrap()
    }

    pub fn discard(&mut self, card: Card) {
        self.discard_pile.push(card);
    }

    pub fn get_last_discarded(&self) -> Option<&Card> {
        self.discard_pile.last()
    }
}

pub fn testing_deck(cards: Vec<Card>) -> Deck {
    Deck {
        cards,
        discard_pile: vec![],
    }
}
