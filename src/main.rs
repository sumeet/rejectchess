use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, Write};

use rejectchess::board::{PieceKind, Square};
use rejectchess::engine::Engine;
use rejectchess::moves::{Move, MoveKind};

fn main() {
    let mut log = open_log();
    let stdin = io::stdin();
    let mut engine = Engine::new();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        log_line(&mut log, "<<", line);
        if line == "uci" {
            send(&mut log, "id name rejectchess");
            send(&mut log, "id author unknown");
            send(&mut log, "uciok");
        } else if line == "isready" {
            send(&mut log, "readyok");
        } else if line == "ucinewgame" {
            engine.reset();
        } else if line.starts_with("position") {
            handle_position(line, &mut engine);
        } else if line.starts_with("go") {
            let best = engine.go();
            match best {
                Some((mv, score)) => {
                    let depth = engine.search_depth();
                    let mv_str = to_uci(mv);
                    send(&mut log, &format!("info depth {} score cp {} pv {}", depth, score, mv_str));
                    send(&mut log, &format!("bestmove {}", mv_str));
                }
                None => send(&mut log, "bestmove 0000"),
            }
        } else if line == "quit" {
            break;
        }
        let _ = io::stdout().flush();
    }
}

fn open_log() -> Option<File> {
    if std::env::var("REJECTCHESS_DEBUG").is_err() {
        return None;
    }
    let path = std::env::temp_dir().join("rejectchess.log");
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .ok()
}

fn log_line(log: &mut Option<File>, prefix: &str, msg: &str) {
    if let Some(f) = log.as_mut() {
        let _ = writeln!(f, "{} {}", prefix, msg);
        let _ = f.flush();
    }
}

fn send(log: &mut Option<File>, msg: &str) {
    println!("{}", msg);
    log_line(log, ">>", msg);
}

fn handle_position(line: &str, engine: &mut Engine) {
    let mut parts = line.split_whitespace();
    let _ = parts.next();

    let mut moves_iter: Option<std::str::SplitWhitespace> = None;

    match parts.next() {
        Some("startpos") => {
            engine.reset();
            if let Some("moves") = parts.next() {
                moves_iter = Some(parts);
            }
        }
        Some("fen") => {
            let mut fen_parts = Vec::new();
            for part in parts.by_ref() {
                if part == "moves" {
                    moves_iter = Some(parts);
                    break;
                }
                fen_parts.push(part);
            }
            let fen = fen_parts.join(" ");
            if !engine.set_fen(&fen) {
                return;
            }
        }
        _ => return,
    }

    if let Some(moves) = moves_iter {
        for token in moves {
            let legal = engine.legal_moves();
            let Some(mv) = parse_uci_move(token, &legal) else {
                break;
            };
            if engine.apply_moves(&[mv]).is_err() {
                break;
            }
        }
    }
}

fn parse_uci_move(token: &str, legal: &[Move]) -> Option<Move> {
    if token.len() < 4 || token.len() > 5 {
        return None;
    }
    let from = parse_square(&token[0..2])?;
    let to = parse_square(&token[2..4])?;
    let promo = if token.len() == 5 {
        Some(parse_promo(token.as_bytes()[4])?)
    } else {
        None
    };

    for mv in legal {
        if mv.from != from || mv.to != to {
            continue;
        }
        match (promo, mv.kind) {
            (Some(p), MoveKind::Promotion(kind)) if p == kind => return Some(*mv),
            (None, MoveKind::Promotion(_)) => continue,
            (None, _) => return Some(*mv),
            _ => continue,
        }
    }
    None
}

fn parse_square(token: &str) -> Option<Square> {
    let bytes = token.as_bytes();
    if bytes.len() != 2 {
        return None;
    }
    let file = bytes[0];
    let rank = bytes[1];
    if !(b'a'..=b'h').contains(&file) || !(b'1'..=b'8').contains(&rank) {
        return None;
    }
    Some((file - b'a', rank - b'1'))
}

fn parse_promo(b: u8) -> Option<PieceKind> {
    match b.to_ascii_lowercase() {
        b'q' => Some(PieceKind::Queen),
        b'r' => Some(PieceKind::Rook),
        b'b' => Some(PieceKind::Bishop),
        b'n' => Some(PieceKind::Knight),
        _ => None,
    }
}

fn to_uci(mv: Move) -> String {
    let mut out = String::new();
    push_square(&mut out, mv.from);
    push_square(&mut out, mv.to);
    if let MoveKind::Promotion(kind) = mv.kind {
        out.push(match kind {
            PieceKind::Queen => 'q',
            PieceKind::Rook => 'r',
            PieceKind::Bishop => 'b',
            PieceKind::Knight => 'n',
            PieceKind::King => 'k',
            PieceKind::Pawn => 'p',
        });
    }
    out
}

fn push_square(out: &mut String, sq: Square) {
    out.push((b'a' + sq.0) as char);
    out.push((b'1' + sq.1) as char);
}
