use owo_colors::OwoColorize as _;
use std::env;
use std::fmt::{self, Write as _};
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

fn solve(board: Shape, pieces: Vec<Shape>) -> Option<Vec<Step>> {
    if pieces.is_empty() {
        return Some(Vec::new());
    }

    let mut tried_pieces = Vec::new();
    let mut put_any = false;

    'out:
    for y in 0..board.height() {
        for x in 0..board.width() {
            tried_pieces.clear();
            for (i, mut piece) in pieces.iter().cloned().enumerate() {
                for r in 0..4 {
                    if tried_pieces.contains(&piece) {
                        piece = piece.rot();
                        continue;
                    }
                    tried_pieces.push(piece.clone());
                    if board.can_put(x, y, &piece) {
                        put_any = true;
                        let new = board.put(x, y, &piece);
                        if !new.has_dead_zones() {
                            let mut remaining = pieces.clone();
                            remaining.swap_remove(i);

                            if let Some(mut steps) = solve(new, remaining) {
                                steps.insert(
                                    0,
                                    Step {
                                        x,
                                        y,
                                        piece: piece.clone(),
                                        r,
                                    },
                                );
                                return Some(steps);
                            }
                        }
                    }
                    piece = piece.rot();
                }
            }
            if put_any {
                break 'out;
            }
        }
    }

    None
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

fn parse_args(mut args: impl Iterator<Item = String>) -> Option<Settings> {
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

fn render_step(board: &Shape, step: &Step) -> String {
    let mut result = String::new();

    result.push_str("╲x│");
    for j in 0..board.width() {
        if j % 2 == 1 {
            if j == step.x {
                write!(result, "{:>2}", j.green()).unwrap();
            } else {
                write!(result, "{:>2}", j).unwrap();
            }
        } else {
            result.push_str("  ");
        }
    }
    write!(result, "│ place at x={}, y={}\n", step.x, step.y).unwrap();

    result.push_str("y╲│");
    for j in 0..board.width() {
        if j % 2 == 0 {
            if j == step.x {
                write!(result, "{:>2}", j.green()).unwrap();
            } else {
                write!(result, "{:>2}", j).unwrap();
            }
        } else {
            result.push_str("  ");
        }
    }
    write!(
        result,
        "│ rotate {} time{}\n",
        step.r,
        if step.r == 1 { "" } else { "s" }
    )
    .unwrap();

    write!(result, "──┼{:─>w$}┤\n", "", w = board.width() * 2).unwrap();

    let orig_piece = {
        let mut piece = step.piece.clone();
        for _ in 0..(4 - step.r) {
            piece = piece.rot();
        }
        piece
    };

    let board_after = board.put(step.x, step.y, &step.piece);

    for i in 0..board.height() {
        if i == step.y {
            write!(result, "{:>2}│", i.green()).unwrap();
        } else {
            write!(result, "{:>2}│", i).unwrap();
        }

        for j in 0..board.width() {
            if board.rows[i][j] {
                result.push_str("██");
            } else if board_after.rows[i][j] {
                write!(result, "{}", "██".green()).unwrap();
            } else {
                result.push_str("  ");
            }
        }

        result.push_str("│");

        if i < orig_piece.height() {
            result.push_str(" ");
            orig_piece.rows[i]
                .iter()
                .for_each(|c| result.push_str(if *c { "██" } else { "  " }));
        }

        result.push_str("\n");
    }
    write!(result, "  └{:─>w$}┘\n", "", w = board.width() * 2).unwrap();

    result
}

fn main() {
    let settings = match parse_args(env::args()) {
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

    if let Some(solution) = solve(
        Shape::with_size(settings.width, settings.height),
        settings.pieces,
    ) {
        let mut board = Shape::with_size(settings.width, settings.height);
        for step in solution {
            println!("{}", render_step(&board, &step));
            board = board.put(step.x, step.y, &step.piece);
        }
    } else {
        println!("no solution found!");
    }
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

    #[test]
    fn check_all_rotations() {
        let settings = parse_args(vec!["", "4", "3", "LLs"].into_iter().map(String::from)).unwrap();
        assert!(solve(
            Shape::with_size(settings.width, settings.height),
            settings.pieces,
        )
        .is_some());
    }
}
