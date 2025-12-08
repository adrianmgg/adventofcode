use rayon::iter::plumbing::Producer;

pub struct SpiralIndexIterator {
    size: u32,
    // (inclusive)
    lo: usize,
    // (inclusive)
    hi: usize,
    done: bool,
}

impl SpiralIndexIterator {
    pub fn new(size: u32) -> Self {
        Self {
            size,
            lo: 0,
            hi: ((size * size) - 1) as usize,
            done: false,
        }
    }

    // via https://stackoverflow.com/a/19287714/8762161
    fn map_spiral(size: u32, i: usize) -> (usize, usize) {
        // handle odd radius.
        // via https://stackoverflow.com/questions/398299/looping-in-a-spiral#comment75984469_19287714
        let (spaceoffset, skipzero) = match size.is_multiple_of(2) {
            true => (size / 2, false),
            false => (size / 2, true),
        };
        let spaceoffset = spaceoffset as i64;

        // inside out -> outside in
        let nu: u64 = ((size * size) as u64) - (i as u64);
        let mut n: i64 = nu.try_into().unwrap();
        if skipzero {
            n -= 1;
        }

        let (x, y) = if skipzero && n == 0 {
            (0, 0)
        } else {
            let r = (n.isqrt() - 1) / 2 + 1;
            let p = (8 * r * (r - 1)) / 2;
            let en = r * 2;
            let a = (1 + n - p) % (r * 8);
            match a / (r * 2) {
                0 => (a - r, -r),
                1 => (r, (a % en) - r),
                2 => (r - (a % en), r),
                3 => (-r, r - (a % en)),
                _ => panic!(),
            }
        };

        (
            (x + spaceoffset).try_into().unwrap(),
            (y + spaceoffset).try_into().unwrap(),
        )
    }
}

impl Producer for SpiralIndexIterator {
    type Item = (usize, usize);
    type IntoIter = SpiralIndexIterator;

    fn into_iter(self) -> Self::IntoIter {
        self
    }

    fn split_at(self, index: usize) -> (Self, Self) {
        (
            Self {
                size: self.size,
                lo: self.lo,
                hi: self.lo + index - 1,
                done: false,
            },
            Self {
                size: self.size,
                lo: self.lo + index,
                hi: self.hi,
                done: false,
            },
        )
    }

    fn min_len(&self) -> usize {
        self.len()
    }

    fn max_len(&self) -> usize {
        self.len()
    }
}

impl ExactSizeIterator for SpiralIndexIterator {
    fn len(&self) -> usize {
        (self.hi + 1).saturating_sub(self.lo)
    }
}

impl DoubleEndedIterator for SpiralIndexIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let cur = self.hi;
        self.hi -= 1;
        if self.hi < self.lo {
            self.done = true;
        }
        Some(Self::map_spiral(self.size, cur))
    }
}

impl Iterator for SpiralIndexIterator {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let cur = self.lo;
        self.lo += 1;
        if self.lo > self.hi {
            self.done = true;
        }
        Some(Self::map_spiral(self.size, cur))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

#[cfg(test)]
mod tests {
    use super::SpiralIndexIterator;

    #[test]
    fn test_spiral_iterator() {
        for n in 1..=32 {
            let it = SpiralIndexIterator::new(n);
            for _ in it {}
        }
    }
}
