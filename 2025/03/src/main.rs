use itertools::Itertools;
use tap::{Pipe as _, Tap as _};
use winnow::{
    ModalResult, Parser as _,
    ascii::line_ending,
    combinator::{opt, repeat, separated, terminated},
    token::take,
};

fn main() {
    let input_file = std::env::args().nth(1).expect("no file specified");
    let txt = std::fs::read_to_string(input_file).expect("reading input file failed");

    let banks = parse_banks
        .parse(txt.as_str())
        .expect("parsing input failed");

    let answer_pt1: u64 = banks.iter().map(|bank| logic(bank, 2)).sum();
    dbg!(answer_pt1);
    let answer_pt2: u64 = banks.iter().map(|bank| logic(bank, 12)).sum();
    dbg!(answer_pt2);
}

fn logic(bank: &[u8], ndigits: usize) -> u64 {
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
