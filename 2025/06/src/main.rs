#![allow(unused)]

use itertools::Itertools;
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

        // part 2
        let (digit_rows, op_row) = parse::row_slices.parse(txt.as_str()).unwrap();
        let mut digit_rows: Vec<_> = digit_rows.into_iter().map(|row| row.chars()).collect();
        let op_row = op_row.chars();

        let mut total = 0u64;
        // let mut cur = None;
        let mut cur_op = Op::Add;
        loop {
            let n = digit_rows
                .iter_mut()
                .map(|row| row.next())
                .map(|c| {
                    c.and_then(|c| match c {
                        ' ' => None,
                        '0'..='9' => Some(c as u64 - '0' as u64),
                        _ => panic!(),
                    })
                })
                .fold(None, |acc, cur| match (acc, cur) {
                    (None, None) => None,
                    (Some(a), None) | (None, Some(a)) => Some(a),
                    (Some(a), Some(b)) => Some(a * 10 + b),
                });
            dbg!(n);
        }
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
        combinator::{
            alt, delimited, dispatch, empty, eof, fail, not, opt, repeat, separated, terminated,
        },
        error::{ContextError, ParserError},
        stream::Stream,
        token::any,
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

    pub fn row_slices<'i>(input: &mut &'i str) -> ModalResult<(Vec<&'i str>, &'i str)> {
        repeat(
            1..,
            terminated(
                repeat(1.., any.verify(|&c| c != '\r' && c != '\n'))
                    .map(|()| ())
                    .take(),
                line_ending,
            ),
        )
        .fold(
            || (Vec::new(), None),
            |(mut v, last), cur| {
                if let Some(prev_last) = last {
                    v.push(prev_last);
                }
                (v, Some(cur))
            },
        )
        .map(|(v, last)| (v, last.unwrap()))
        .parse_next(input)
    }

    /*
    fn digit_or_ws(input: &mut &str) -> ModalResult<Option<u64>> {
        dispatch! {any;
            ' ' => empty.value(None),
            c @ '0'..='9' => empty.value(Some(c as u64 - '0' as u64)),
            _ => fail,
        }
        .parse_next(input)
    }

    pub fn worksheet_pt2(input: &mut &str) -> ModalResult<Vec<(Op, Vec<u64>)>> {
        let mut row_starts = vec![input.checkpoint()];
        'outer: loop {
            'inner: loop {
                #[derive(Debug, Copy, Clone)]
                enum Foo {
                    Char,
                    Eof,
                    Eol,
                }
                match alt((
                    (opt(line_ending), eof).value(Foo::Eof),
                    line_ending.value(Foo::Eol),
                    any.value(Foo::Char),
                ))
                .parse_next(input)?
                {
                    Foo::Char => {}
                    Foo::Eof => break 'outer,
                    Foo::Eol => break 'inner,
                }
            }
            row_starts.push(input.checkpoint());
        }

        let mut ops_ckpt = row_starts.last();
        row_starts.pop();

        let mut problems = Vec::new();

        loop {
            let mut num: Option<u64> = None;
            for ckpt in row_starts.iter_mut() {
                input.reset(ckpt);
                match digit_or_ws.parse_next(input)? {
                    None => {}
                    Some(n) => {
                        let num = num.get_or_insert_default();
                        *num *= 10;
                        *num += n;
                    }
                }
                *ckpt = input.checkpoint();
            }
        }

        Ok(problems)
    }
    */
}
