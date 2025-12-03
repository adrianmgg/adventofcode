use winnow::{
    ascii::{dec_uint, line_ending},
    combinator::{alt, eof, repeat, seq, terminated},
    prelude::*,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Entry {
    direction: Direction,
    n: u64,
}

fn parse_entry(input: &mut &str) -> Result<Entry, ()> {
    seq! {Entry{
        direction: alt(('L'.value(Direction::Left), 'R'.value(Direction::Right))),
        n: dec_uint::<_, u64, ()>,
    }}
    .parse_next(input)
}

impl Entry {
    fn normalize_to_positive(&self) -> u64 {
        match self.direction {
            Direction::Right => self.n,
            Direction::Left => {
                let mut n = self.n;
                while n > 100 {
                    n -= 100
                }
                100 - n
            }
        }
    }

    fn with_n(&self, n: u64) -> Self {
        Self {
            direction: self.direction,
            n,
        }
    }
}

fn calculate<F: Fn(u64, u64, Entry) -> (u64, u64)>(input: &str, logic: F) -> u64 {
    repeat(0.., terminated(parse_entry, alt((line_ending, eof))))
        .fold(|| (50, 0), |(dial, zeros), entry| logic(dial, zeros, entry))
        .map(|(_dial, zeros)| zeros)
        .parse(input)
        .unwrap()
}

fn part1_logic(dial: u64, zeros: u64, entry: Entry) -> (u64, u64) {
    let n = entry.normalize_to_positive();
    let new_dial = (dial + n) % 100;
    (new_dial, zeros + if new_dial == 0 { 1 } else { 0 })
}

fn part2_logic(mut dial: u64, mut zeros: u64, entry: Entry) -> (u64, u64) {
    for _ in 0..entry.n {
        dial = (dial + entry.with_n(1).normalize_to_positive()) % 100;
        if dial == 0 {
            zeros += 1
        }
    }
    (dial, zeros)
}

fn main() {
    for inputfile in std::env::args().skip(1) {
        println!("{inputfile}");
        let txt = std::fs::read_to_string(inputfile).unwrap();
        let answer_pt1 = calculate(&txt, part1_logic);
        println!("  pt1: {answer_pt1}");
        let answer_pt2 = calculate(&txt, part2_logic);
        println!("  pt2: {answer_pt2}");
    }
}
