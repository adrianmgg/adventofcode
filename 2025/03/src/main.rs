use itertools::Itertools;
use tap::{Pipe as _, Tap as _};
use winnow::{
    ModalResult, Parser as _,
    ascii::line_ending,
    combinator::{opt, repeat, separated, terminated},
    token::take,
};

// 17406 -> too low

fn main() {
    let input_file = std::env::args().nth(1).expect("no file specified");
    let txt = std::fs::read_to_string(input_file).expect("reading input file failed");

    let banks: Vec<Vec<u8>> = parse_banks
        .parse(txt.as_str())
        .expect("parsing input failed");

    let answer_pt1: u64 = banks.iter().map(|bank| logic_pt1(bank)).sum();
    dbg!(answer_pt1);
    let answer_pt2: u64 = banks.iter().map(|bank| logic_pt2(bank, 12)).sum();
    dbg!(answer_pt2);
}

fn logic_pt1(bank: &[u8]) -> u64 {
    let (tens_idx, tens) = bank
        .iter()
        .enumerate()
        // skip the last digit
        .tap_mut(|it| {
            it.next_back();
        })
        // take the earliest available instance of the highest available digit
        .max_by_key(|(idx, digit)| (*digit, std::cmp::Reverse(*idx)))
        // we can safely unwrap since we know from the parser there's at least 2 elements
        // in the list
        .unwrap();
    let ones = bank.iter().skip(tens_idx + 1).max().unwrap();
    ((*tens as u64) * 10) + (*ones as u64)
}

fn logic_pt1_bruteforce(bank: &[u8]) -> u64 {
    bank.iter()
        .enumerate()
        .tap_mut(|it| {
            it.next_back();
        })
        .flat_map(|(tens_idx, tens)| {
            bank.iter()
                .skip(tens_idx + 1)
                .map(|ones| ((*tens as u64) * 10) + (*ones as u64))
        })
        .max()
        .unwrap()
}

fn logic_pt2(bank: &[u8], ndigits: usize) -> u64 {
    let mut ret = 0u64;

    // index before which we can no longer use digits from
    let mut preclude_first_n = 0usize;

    for i in 0..ndigits {
        let (idx, digit) = bank
            .iter()
            .enumerate()
            .skip(preclude_first_n)
            // exclude the last however many digits from selection
            .tap_mut(|it| {
                it.dropping_back(ndigits - 1 - i);
            })
            // take the earliest available instance of the highest available digit value
            .max_by_key(|(idx, digit)| (*digit, std::cmp::Reverse(*idx)))
            .unwrap();
        ret = (ret * 10) + (*digit as u64);
        preclude_first_n = idx + 1;
    }

    ret
}

fn parse_bank(input: &mut &str) -> ModalResult<Vec<u8>> {
    repeat(2.., take(1usize).parse_to::<u8>()).parse_next(input)
}

fn parse_banks(input: &mut &str) -> ModalResult<Vec<Vec<u8>>> {
    separated(2.., parse_bank, line_ending)
        // trailing newline
        .pipe(|parser| terminated(parser, opt(line_ending)))
        .parse_next(input)
}
