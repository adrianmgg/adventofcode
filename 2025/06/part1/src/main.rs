#![allow(unused)]

use tap::Pipe as _;
use winnow::Parser as _;

fn main() {
    let input_files = std::env::args().skip(1).collect::<Vec<_>>().pipe(|files| {
        if files.is_empty() {
            vec!["sample.txt".into(), "input.txt".into()]
        } else {
            files
        }
    });

    for input_file in input_files {
        println!("{input_file}");

        let txt = std::fs::read_to_string(input_file).expect("reading input file failed");

        let worksheet = parse::worksheet.parse(txt.as_str()).unwrap();

        let part1_solution: u64 = {
            let (numrows, oprow) = worksheet;
            oprow
                .iter()
                .enumerate()
                .map(|(idx, op)| {
                    numrows
                        .iter()
                        .map(|row| row.get(idx).unwrap())
                        .fold(op.fold_start(), |acc, cur| op.op(acc, *cur))
                })
                .sum()
        };
        dbg!(part1_solution);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Add,
    Mul,
}

impl Op {
    fn fold_start(&self) -> u64 {
        match self {
            Op::Add => 0,
            Op::Mul => 1,
        }
    }

    fn op(&self, a: u64, b: u64) -> u64 {
        match self {
            Op::Add => a + b,
            Op::Mul => a * b,
        }
    }
}

mod parse {
    use super::Op;
    use tap::Pipe as _;
    use winnow::{
        ModalResult, Parser,
        ascii::{dec_uint, line_ending, space0, space1},
        combinator::{alt, delimited, opt, repeat, separated, terminated},
        error::ParserError,
    };

    fn op(input: &mut &str) -> ModalResult<Op> {
        alt(('+'.value(Op::Add), '*'.value(Op::Mul))).parse_next(input)
    }

    fn num(input: &mut &str) -> ModalResult<u64> {
        dec_uint.parse_next(input)
    }

    fn row<'a, O, E, P>(item: P) -> impl Parser<&'a str, Vec<O>, E>
    where
        E: ParserError<&'a str>,
        P: Parser<&'a str, O, E>,
    {
        delimited(space0, separated(1.., item, space1), space0)
    }

    pub fn worksheet(input: &mut &str) -> ModalResult<(Vec<Vec<u64>>, Vec<Op>)> {
        (repeat(2.., terminated(row(num), line_ending)), row(op))
            .pipe(|parser| terminated(parser, opt(line_ending)))
            .parse_next(input)
    }
}
