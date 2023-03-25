use std::env;
use std::fmt;
use std::process;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Shape {
    rows: Vec<Vec<bool>>,
}

#[derive(Clone, Debug)]
struct Step {
    x: usize,
    y: usize,
    piece: Shape,
    r: usize,
}

#[derive(Debug)]
struct Settings {
    width: usize,
    height: usize,
    pieces: Vec<Shape>,
}

impl Shape {
    fn new(rows: Vec<Vec<bool>>) -> Self {
        return Self { rows };
    }

    fn from_str(string: &str) -> Self {
        let mut rows = Vec::new();
        for line in string.trim().lines() {
            let mut row = Vec::new();
            for c in line.chars().filter(|c| !c.is_whitespace()) {
                row.push(c != '_');
            }
            row.shrink_to_fit();
            rows.push(row)
        }
        rows.shrink_to_fit();
        Self::new(rows)
    }

    fn with_size(width: usize, height: usize) -> Self {
        Self::new(vec![vec![false; width]; height])
    }

    fn width(&self) -> usize {
        self.rows[0].len()
    }

    fn height(&self) -> usize {
        self.rows.len()
    }

    fn rot(&self) -> Self {
        let mut new_rows = vec![vec![false; self.height()]; self.width()];
        for i in 0..self.height() {
            for j in 0..self.width() {
                new_rows[j][self.height() - i - 1] = self.rows[i][j];
            }
        }
        Self::new(new_rows)
    }

    fn flip(&self) -> Self {
        Self::new(
            self.rows
                .iter()
                .map(|row| row.iter().copied().rev().collect())
                .collect(),
        )
    }

    fn can_put(&self, x: usize, y: usize, other: &Self) -> bool {
        if x + other.width() > self.width() || y + other.height() > self.height() {
            return false;
        }

        for i in 0..other.height() {
            for j in 0..other.width() {
                if other.rows[i][j] && self.rows[y + i][x + j] {
                    return false;
                }
            }
        }

        true
    }

    fn put(&self, x: usize, y: usize, other: &Self) -> Self {
        let mut new = self.rows.clone();
        for i in 0..other.height() {
            for j in 0..other.width() {
                new[y + i][x + j] |= other.rows[i][j]
            }
        }

        Shape::new(new)
    }

    fn len(&self) -> usize {
        self.rows
            .iter()
            .map(|row| row.iter().map(|c| *c as usize).sum::<usize>())
            .sum()
    }

    fn is_open(&self, x: isize, y: isize) -> bool {
        0 <= x
            && (x as usize) < self.width()
            && 0 <= y
            && (y as usize) < self.height()
            && !self.rows[y as usize][x as usize]
    }

    fn is_dead(&self, x: usize, y: usize) -> bool {
        let mut explored = [Some((x as isize, y as isize)), None, None];

        for _ in 0..4 {
            for ei in 0..explored.len() {
                if let Some((x, y)) = explored[ei] {
                    for (dx, dy) in [(0, 1), (1, 0), (0, -1), (-1, 0)] {
                        if self.is_open(x + dx, y + dy)
                            && !explored.contains(&Some((x + dx, y + dy)))
                        {
                            let mut stored = false;
                            for iei in 0..explored.len() {
                                if explored[iei].is_none() {
                                    explored[iei] = Some((x + dx, y + dy));
                                    stored = true;
                                    break;
                                }
                            }
                            if !stored {
                                // explore list full, means there may be space for a shape
                                return false;
                            }
                        }
                    }
                }
            }
        }

        // could not fill list, probably dead
        true
    }

    fn has_dead_zones(&self) -> bool {
        self.rows.iter().enumerate().any(|(i, row)| {
            row.iter()
                .enumerate()
                .any(|(j, c)| !*c && self.is_dead(j, i))
        })
    }
}

fn solve(board: Shape, pieces: Vec<Shape>, used: Vec<Step>) -> bool {
    if pieces.is_empty() {
        for step in used {
            println!(
                "rotate {} times, then place at ({}, {}):",
                step.r, step.x, step.y
            );
            println!("{}", step.piece);
        }
        return true;
    }

    let mut tried_pieces = Vec::new();

    for y in 0..board.height() {
        for x in 0..board.width() {
            tried_pieces.clear();
            for (i, mut piece) in pieces.iter().cloned().enumerate() {
                for r in 0..3 {
                    if tried_pieces.contains(&piece) {
                        piece = piece.rot();
                        continue;
                    }
                    tried_pieces.push(piece.clone());
                    if board.can_put(x, y, &piece) {
                        let new = board.put(x, y, &piece);
                        if !new.has_dead_zones() {
                            let mut remaining = pieces.clone();
                            remaining.swap_remove(i);

                            let mut used_now = used.clone();
                            used_now.push(Step {
                                x,
                                y,
                                piece: piece.clone(),
                                r,
                            });

                            if solve(new, remaining, used_now) {
                                return true;
                            }
                        }
                    }
                    piece = piece.rot();
                }
            }
        }
    }

    false
}

impl fmt::Display for Shape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in self.rows.iter() {
            for c in row {
                f.write_str(if *c { "# " } else { "_ " })?;
            }
            f.write_str("\n")?;
        }
        Ok(())
    }
}

fn parse_args() -> Option<Settings> {
    let shape_t = Shape::from_str(
        "
        # # #
        _ # _
        ",
    );
    let shape_i = Shape::from_str(
        "
        # # # #
        ",
    );
    let shape_l = Shape::from_str(
        "
        # # #
        # _ _
        ",
    );
    let shape_o = Shape::from_str(
        "
        # #
        # #
        ",
    );
    let shape_s = Shape::from_str(
        "
        # # _
        _ # #
        ",
    );

    let mut args = env::args();
    let _prog = args.next()?;

    let width = match args.next()?.parse() {
        Ok(n) => n,
        Err(e) => {
            eprintln!("could not parse board width: {e}");
            process::exit(1);
        }
    };

    let height = match args.next()?.parse() {
        Ok(n) => n,
        Err(e) => {
            eprintln!("could not parse board height: {e}");
            process::exit(2);
        }
    };

    let mut pieces = Vec::new();
    for c in args.next()?.chars() {
        pieces.push(match c {
            't' => shape_t.clone(),
            'i' => shape_i.clone(),
            'l' => shape_l.clone(),
            'o' => shape_o.clone(),
            's' => shape_s.clone(),
            'T' => shape_t.clone(),
            'I' => shape_i.clone(),
            'L' => shape_l.flip().rot().rot(),
            'O' => shape_o.clone(),
            'S' => shape_s.flip(),
            _ => {
                eprintln!("unknown shape: {c}");
                process::exit(3);
            }
        });
    }

    Some(Settings {
        width,
        height,
        pieces,
    })
}

fn main() {
    let settings = match parse_args() {
        Some(s) => s,
        None => {
            eprintln!("usage: {} WIDTH HEIGHT PIECES", env::args().next().unwrap());
            eprintln!("available pieces (uppercase to flip it): tilos");
            process::exit(4);
        }
    };

    if settings
        .pieces
        .iter()
        .map(|piece| piece.len())
        .sum::<usize>()
        != settings.width * settings.height
    {
        eprintln!("available pieces (given pieces are not enough or are too much)");
        process::exit(5);
    }

    solve(
        Shape::with_size(settings.width, settings.height),
        settings.pieces,
        vec![],
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_deadzones() {
        assert!(Shape::from_str(
            "
            # # # _ _ _ _ _
            _ # _ _ _ _ _ _
            # # # _ _ _ _ _
            _ # _ _ _ _ _ _
            _ _ _ _ _ _ _ _
            _ _ _ _ _ _ _ _
            ",
        )
        .has_dead_zones());
        assert!(Shape::from_str(
            "
            # # # _ _ _ _ _
            _ _ # _ _ _ _ _
            # # # _ _ _ _ _
            _ # _ _ _ _ _ _
            _ _ _ _ _ _ _ _
            _ _ _ _ _ _ _ _
            ",
        )
        .has_dead_zones());
        assert!(Shape::from_str(
            "
            # # # _ _ _ _ _
            _ _ # _ _ _ _ _
            # _ # # # _ _ _
            # # # _ _ _ _ _
            # _ _ _ _ _ _ _
            _ _ _ _ _ _ _ _
            ",
        )
        .has_dead_zones());
        assert!(!Shape::from_str(
            "
            # # # # # _ _ _
            _ _ _ _ # _ _ _
            # # # # # # _ _
            _ _ _ _ _ _ _ _
            _ _ _ _ _ _ _ _
            _ _ _ _ _ _ _ _
            ",
        )
        .has_dead_zones());
    }
}
