use std::fmt::{Display, Error, Formatter};

#[derive(Debug)]
struct Gameboard<const X: usize, const Y: usize> {
    state: [[u8; Y]; X],
}

impl<const X: usize, const Y: usize> Gameboard<X, Y> {
    fn set_cell(&mut self, x: usize, y: usize, value: u8) {
        self.state[x][y] = value;
    }
}

impl<const X: usize, const Y: usize> Display for Gameboard<X, Y> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        for x in 0..X {
            for y in 0..Y {
                write!(
                    f,
                    "{} ",
                    match self.state[x][y] {
                        0 => ".".to_string(),
                        a => format!("{}", a),
                    },
                )?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

impl<const X: usize, const Y: usize> From<[[u8; Y]; X]> for Gameboard<X, Y> {
    fn from(value: [[u8; Y]; X]) -> Self {
        Self {
            state: value.clone(),
        }
    }
}

trait ToCellMask {
    fn to_cell_mask(self) -> u16;
}

#[derive(Debug)]
struct Candidates<const X: usize, const Y: usize> {
    cells: [[u16; Y]; X],
}

impl<const X: usize, const Y: usize> Default for Candidates<X, Y> {
    fn default() -> Self {
        Self {
            cells: [[511_u16; Y]; X], // 0b111_111_111
        }
    }
}

impl ToCellMask for u8 {
    fn to_cell_mask(self) -> u16 {
        1 << self - 1
    }
}

impl<const X: usize, const Y: usize> Candidates<X, Y> {
    fn apply_uniques(&self, gameboard: &mut Gameboard<X, Y>) -> bool {
        let mut changes_made = false;

        for x in 0..X {
            for y in 0..Y {
                if self.remaining_candidates(x, y) == 1 {
                    gameboard.set_cell(x, y, (self.cells[x][y] as f32).log2() as u8 + 1);
                    changes_made = true;
                }
            }
        }

        changes_made
    }

    fn exclude_candidate(&mut self, x: usize, y: usize, candidate: u8) {
        self.cells[x][y] &= !candidate.to_cell_mask();
    }
    fn set_exclusive_candidate(&mut self, x: usize, y: usize, candidate: u8) {
        self.cells[x][y] = candidate.to_cell_mask();
    }
    fn remaining_candidates(&self, x: usize, y: usize) -> u32 {
        self.cells[x][y].count_ones()
    }
    fn mark_as_solved(&mut self, x: usize, y: usize) {
        self.cells[x][y] = 0;
    }
}

trait Rule<const X: usize, const Y: usize> {
    fn visit(gameboard: &Gameboard<X, Y>, candidates: &mut Candidates<X, Y>);
}

struct ExcludeWhenSolved;

impl<const X: usize, const Y: usize> Rule<X, Y> for ExcludeWhenSolved {
    fn visit(gameboard: &Gameboard<X, Y>, candidates: &mut Candidates<X, Y>) {
        for x in 0..X {
            for y in 0..Y {
                if gameboard.state[x][y] == 0 {
                    continue;
                };

                candidates.mark_as_solved(x, y);
            }
        }
    }
}

struct UniqueByRow;

impl<const X: usize, const Y: usize> Rule<X, Y> for UniqueByRow {
    fn visit(gameboard: &Gameboard<X, Y>, candidates: &mut Candidates<X, Y>) {
        for x in 0..X {
            for y in 0..Y {
                if gameboard.state[x][y] == 0 {
                    continue;
                };

                for y2 in 0..Y {
                    candidates.exclude_candidate(x, y2, gameboard.state[x][y]);
                }
            }
        }
    }
}

struct UniqueByColumn;

impl<const X: usize, const Y: usize> Rule<X, Y> for UniqueByColumn {
    fn visit(gameboard: &Gameboard<X, Y>, candidates: &mut Candidates<X, Y>) {
        for x in 0..X {
            for y in 0..Y {
                if gameboard.state[x][y] == 0 {
                    continue;
                };

                for x2 in 0..X {
                    candidates.exclude_candidate(x2, y, gameboard.state[x][y]);
                }
            }
        }
    }
}

struct FillColumnUniquely;

impl<const X: usize, const Y: usize> Rule<X, Y> for FillColumnUniquely {
    fn visit(_: &Gameboard<X, Y>, candidates: &mut Candidates<X, Y>) {
        for x in 0..X {
            'next_n: for n in 1..=9 {
                let mut solo_position = None;

                for y in 0..Y {
                    if candidates.cells[x][y] & n.to_cell_mask() > 0 {
                        if solo_position != None {
                            continue 'next_n;
                        } else {
                            solo_position = Some(y);
                        }
                    }
                }

                if let Some(y) = solo_position {
                    for y2 in 0..Y {
                        if y2 == y {
                            candidates.set_exclusive_candidate(x, y2, n);
                        } else {
                            candidates.exclude_candidate(x, y2, n);
                        }
                    }
                }
            }
        }
    }
}

struct FillRowUniquely;

impl<const X: usize, const Y: usize> Rule<X, Y> for FillRowUniquely {
    fn visit(_: &Gameboard<X, Y>, candidates: &mut Candidates<X, Y>) {
        for y in 0..Y {
            'next_n: for n in 1..=9 {
                let mut solo_position = None;

                for x in 0..X {
                    if candidates.cells[x][y] & n.to_cell_mask() > 0 {
                        if solo_position != None {
                            continue 'next_n;
                        } else {
                            solo_position = Some(x);
                        }
                    }
                }

                if let Some(x) = solo_position {
                    for x2 in 0..X {
                        if x2 == x {
                            candidates.set_exclusive_candidate(x2, y, n);
                        } else {
                            candidates.exclude_candidate(x2, y, n);
                        }
                    }
                }
            }
        }
    }
}

struct UniqueBy3x3Box;

impl Rule<9, 9> for UniqueBy3x3Box {
    fn visit(gameboard: &Gameboard<9, 9>, candidates: &mut Candidates<9, 9>) {
        for x_outer in 0..3 {
            for y_outer in 0..3 {
                for x_inner in 0..3 {
                    for y_inner in 0..3 {
                        let x = x_outer * 3 + x_inner;
                        let y = y_outer * 3 + y_inner;

                        if gameboard.state[x][y] == 0 {
                            continue;
                        };

                        for x_inner2 in 0..3 {
                            for y_inner2 in 0..3 {
                                let x2 = x_outer * 3 + x_inner2;
                                let y2 = y_outer * 3 + y_inner2;
                                candidates.exclude_candidate(x2, y2, gameboard.state[x][y]);
                            }
                        }
                    }
                }
            }
        }
    }
}

struct Fill3x3BoxUniquely;

impl Rule<9, 9> for Fill3x3BoxUniquely {
    fn visit(_: &Gameboard<9, 9>, candidates: &mut Candidates<9, 9>) {
        for x_outer in 0..3 {
            for y_outer in 0..3 {
                'next_n: for n in 1..=9 {
                    let mut solo_position = None;

                    for x_inner in 0..3 {
                        for y_inner in 0..3 {
                            let x = x_outer * 3 + x_inner;
                            let y = y_outer * 3 + y_inner;

                            if candidates.cells[x][y] & n.to_cell_mask() > 0 {
                                if solo_position != None {
                                    continue 'next_n;
                                } else {
                                    solo_position = Some((x, y));
                                }
                            }
                        }
                    }

                    if let Some((x, y)) = solo_position {
                        for x_inner2 in 0..3 {
                            for y_inner2 in 0..3 {
                                let x2 = x_outer * 3 + x_inner2;
                                let y2 = y_outer * 3 + y_inner2;

                                if (x2, y2) == (x, y) {
                                    candidates.set_exclusive_candidate(x2, y2, n);
                                } else {
                                    candidates.exclude_candidate(x2, y2, n);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    let mut gameboard: Gameboard<9, 9> = [
        [0, 0, 0, 0, 8, 0, 0, 0, 0],
        [0, 0, 5, 6, 0, 3, 9, 0, 0],
        [0, 8, 4, 0, 0, 0, 2, 7, 0],
        [0, 3, 0, 1, 0, 0, 0, 5, 0],
        [5, 0, 0, 0, 3, 0, 0, 0, 2],
        [0, 6, 0, 0, 0, 5, 0, 1, 0],
        [0, 1, 9, 0, 0, 0, 5, 6, 0],
        [0, 0, 8, 4, 0, 2, 7, 0, 0],
        [0, 0, 0, 0, 6, 0, 0, 0, 0],
    ]
    .into();

    let mut candidates = Candidates::<9, 9>::default();

    loop {
        ExcludeWhenSolved::visit(&gameboard, &mut candidates);
        UniqueByColumn::visit(&gameboard, &mut candidates);
        UniqueByRow::visit(&gameboard, &mut candidates);
        UniqueBy3x3Box::visit(&gameboard, &mut candidates);
        FillColumnUniquely::visit(&gameboard, &mut candidates);
        FillRowUniquely::visit(&gameboard, &mut candidates);
        Fill3x3BoxUniquely::visit(&gameboard, &mut candidates);

        if !candidates.apply_uniques(&mut gameboard) {
            break;
        }
    }

    println!("{}", gameboard);
    println!("{:?}", candidates);
}
