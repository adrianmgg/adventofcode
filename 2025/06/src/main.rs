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

mod parse2 {
    use std::ops::RangeInclusive;

    use winnow::stream::{Offset, Stream};

    #[derive(Debug)]
    struct TransposeStream<'s> {
        data: &'s str,
        row_width: usize,
    }

    #[derive(Debug, Clone, Copy)]
    struct Pos {
        row_width: usize,
        n_rows: usize,
        x: usize,
        y: usize,
    }

    impl Pos {
        fn to_idx(&self) -> usize {
            (self.x * self.n_rows) + self.y
        }
    }

    impl Offset for Pos {
        fn offset_from(&self, start: &Self) -> usize {
            assert_eq!(self.row_width, start.row_width);
            assert_eq!(self.n_rows, start.n_rows);
            self.to_idx() - start.to_idx()
        }
    }

    impl<'s> Offset<Pos> for TransposeStream<'s> {
        fn offset_from(&self, start: &Pos) -> usize {
            Pos {
            })
        }
    }

    impl<'s> Stream for TransposeStream<'s> {
        type Token = char;

        type Slice = RangeInclusive<Pos>;

        type IterOffsets;

        type Checkpoint = Pos;

        fn iter_offsets(&self) -> Self::IterOffsets {
            todo!()
        }

        fn eof_offset(&self) -> usize {
            todo!()
        }

        fn next_token(&mut self) -> Option<Self::Token> {
            todo!()
        }

        fn peek_token(&self) -> Option<Self::Token> {
            todo!()
        }

        fn offset_for<P>(&self, predicate: P) -> Option<usize>
        where
            P: Fn(Self::Token) -> bool,
        {
            todo!()
        }

        fn offset_at(&self, tokens: usize) -> Result<usize, winnow::error::Needed> {
            todo!()
        }

        fn next_slice(&mut self, offset: usize) -> Self::Slice {
            todo!()
        }

        fn peek_slice(&self, offset: usize) -> Self::Slice {
            todo!()
        }

        fn checkpoint(&self) -> Self::Checkpoint {
            todo!()
        }

        fn reset(&mut self, checkpoint: &Self::Checkpoint) {
            todo!()
        }

        fn raw(&self) -> &dyn core::fmt::Debug {
            todo!()
        }
    }
}
