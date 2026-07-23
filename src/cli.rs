use crate::action::Action;
use crate::search::{SearchResult, search_best_action};
use crate::state::GameState;
use std::io::{self, BufRead, Write};

const HELP_TEXT: &str = "\
commands:
e2e4       make a move in UCI format
a7a8q      parsed, but promotion is not implemented
moves      list all legal actions
moves e2   list legal actions from one square
go depth <0-8>    calculate the best move without playing it
play depth <0-8>  calculate and play the best move
board      print the current board
help       show this help message
quit       exit the program
exit       exit the program";

pub fn run_cli<R: BufRead, W: Write>(mut reader: R, writer: &mut W) -> io::Result<()> {
    let mut game = GameState::standard();
    let mut line = String::new();

    writeln!(writer, "OmniChess")?;
    writeln!(writer)?;
    writeln!(writer, "{}", game.board())?;

    loop {
        write!(writer, "{}> ", game.side_to_move())?;
        writer.flush()?;

        line.clear();
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 {
            return Ok(());
        }

        let input = line.trim();
        if input.is_empty() {
            continue;
        }

        let lowercase = input.to_ascii_lowercase();

        if lowercase == "quit" || lowercase == "exit" {
            return Ok(());
        }

        if lowercase == "help" {
            writeln!(writer, "{HELP_TEXT}")?;
            continue;
        }

        if lowercase == "board" {
            writeln!(writer)?;
            writeln!(writer, "{}", game.board())?;
            continue;
        }

        if lowercase == "moves" {
            write_actions(writer, &game.legal_actions())?;
            continue;
        }

        if let Some(depth) = parse_depth_command(&lowercase, "go") {
            match depth {
                Some(depth) => {
                    let result = search_best_action(&game, depth);
                    write_search_result(writer, &result)?;
                }
                None => writeln!(writer, "usage: go depth <0-8>")?,
            }
            continue;
        }

        if let Some(depth) = parse_depth_command(&lowercase, "play") {
            match depth {
                Some(depth) => {
                    let result = search_best_action(&game, depth);
                    write_search_result(writer, &result)?;

                    let Some(best_action) = result.best_action else {
                        writeln!(writer, "no legal actions")?;
                        continue;
                    };

                    match game.apply_action(best_action) {
                        Ok(_) => {
                            writeln!(writer)?;
                            writeln!(writer, "{}", game.board())?;

                            if let Some(result) = game.result() {
                                writeln!(writer, "{result}")?;
                                return Ok(());
                            }
                        }
                        Err(error) => {
                            panic!("search returned illegal action {best_action}: {error}");
                        }
                    }
                }
                None => writeln!(writer, "usage: play depth <0-8>")?,
            }
            continue;
        }

        if let Some((command, square_text)) = lowercase.split_once(char::is_whitespace)
            && command == "moves"
        {
            if let Some(square) = crate::square::Square::from_algebraic(square_text.trim()) {
                write_actions(writer, &game.legal_actions_from(square))?;
            } else {
                writeln!(writer, "invalid command or move")?;
            }
            continue;
        }

        let Some(action) = Action::from_uci(input) else {
            writeln!(writer, "invalid command or move")?;
            continue;
        };

        match game.apply_action(action) {
            Ok(_) => {
                writeln!(writer)?;
                writeln!(writer, "{}", game.board())?;

                if let Some(result) = game.result() {
                    writeln!(writer, "{result}")?;
                    return Ok(());
                }
            }
            Err(error) => {
                writeln!(writer, "{error}")?;
            }
        }
    }
}

fn parse_depth_command(input: &str, command: &str) -> Option<Option<u8>> {
    let mut parts = input.split_whitespace();
    let first = parts.next()?;
    if first != command {
        return None;
    }

    let Some(keyword) = parts.next() else {
        return Some(None);
    };
    if keyword != "depth" {
        return Some(None);
    }

    let Some(depth_text) = parts.next() else {
        return Some(None);
    };
    if parts.next().is_some() {
        return Some(None);
    }

    match depth_text.parse::<u8>() {
        Ok(depth) if depth <= 8 => Some(Some(depth)),
        _ => Some(None),
    }
}

fn write_actions<W: Write>(writer: &mut W, actions: &[Action]) -> io::Result<()> {
    if actions.is_empty() {
        writeln!(writer, "no legal actions")
    } else {
        let line = actions
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ");
        writeln!(writer, "{line}")
    }
}

fn write_search_result<W: Write>(writer: &mut W, result: &SearchResult) -> io::Result<()> {
    match result.best_action {
        Some(action) => writeln!(
            writer,
            "bestmove {action} score {} depth {} nodes {}",
            result.score, result.depth, result.nodes
        ),
        None => writeln!(
            writer,
            "bestmove none score {} depth {} nodes {}",
            result.score, result.depth, result.nodes
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::run_cli;
    use std::io::Cursor;

    fn run(input: &str) -> String {
        let reader = Cursor::new(input.as_bytes());
        let mut output = Vec::new();
        run_cli(reader, &mut output).unwrap();
        String::from_utf8(output).unwrap()
    }

    #[test]
    fn startup_prints_board_and_white_prompt() {
        let output = run("quit\n");

        assert!(output.contains("OmniChess"));
        assert!(output.contains("8 r n b q k b n r"));
        assert!(output.contains("White>"));
    }

    #[test]
    fn e2e4_changes_prompt_to_black() {
        let output = run("e2e4\nquit\n");

        assert!(output.contains("4 . . . . P . . ."));
        assert!(output.contains("Black>"));
    }

    #[test]
    fn illegal_move_does_not_change_board_or_turn() {
        let output = run("e2e5\nquit\n");

        assert!(output.contains("illegal movement"));
        assert!(!output.contains("Black>"));
        assert_eq!(output.matches("White>").count(), 2);
    }

    #[test]
    fn invalid_text_input_does_not_stop_program() {
        let output = run("hello\nquit\n");

        assert!(output.contains("invalid command or move"));
        assert_eq!(output.matches("White>").count(), 2);
    }

    #[test]
    fn moves_lists_twenty_actions_from_start() {
        let output = run("moves\nquit\n");
        let moves_line = output
            .lines()
            .find(|line| line.contains("b1a3"))
            .unwrap()
            .trim();
        let moves: Vec<_> = moves_line
            .split_whitespace()
            .filter(|item| *item != "White>")
            .collect();

        assert_eq!(moves.len(), 20);
    }

    #[test]
    fn moves_e2_lists_e2e3_and_e2e4() {
        let output = run("moves e2\nquit\n");

        assert!(output.contains("e2e3 e2e4"));
    }

    #[test]
    fn board_command_reprints_current_board() {
        let output = run("board\nquit\n");

        assert!(output.matches("8 r n b q k b n r").count() >= 2);
    }

    #[test]
    fn help_prints_command_list() {
        let output = run("help\nquit\n");

        assert!(output.contains("commands:"));
        assert!(output.contains("moves e2"));
        assert!(output.contains("go depth <0-8>"));
        assert!(output.contains("quit"));
    }

    #[test]
    fn quit_and_exit_end_normally() {
        let quit_output = run("quit\n");
        let exit_output = run("exit\n");

        assert!(quit_output.contains("White>"));
        assert!(exit_output.contains("White>"));
    }

    #[test]
    fn eof_exits_normally() {
        let output = run("");

        assert!(output.contains("White>"));
    }

    #[test]
    fn king_capture_prints_result_and_exits() {
        let output = run("e2e4\nf7f6\nd1h5\na7a6\nh5e8\n");

        assert!(output.contains("White wins"));
    }

    #[test]
    fn go_depth_one_prints_bestmove() {
        let output = run("go depth 1\nquit\n");

        assert!(output.contains("bestmove "));
        assert!(output.contains(" depth 1 "));
    }

    #[test]
    fn go_depth_one_does_not_change_state() {
        let output = run("go depth 1\nboard\nquit\n");

        assert!(output.matches("8 r n b q k b n r").count() >= 2);
        assert_eq!(output.matches("White>").count(), 3);
    }

    #[test]
    fn play_depth_one_applies_move() {
        let output = run("play depth 1\nquit\n");

        assert!(output.contains("bestmove "));
        assert!(output.contains("Black>"));
    }

    #[test]
    fn play_depth_one_changes_turn_and_ply() {
        let output = run("play depth 1\nplay depth 1\nquit\n");

        assert!(output.matches("bestmove ").count() >= 2);
        assert!(output.contains("White>"));
        assert!(output.contains("Black>"));
    }

    #[test]
    fn invalid_go_usage_prints_usage_and_continues() {
        let output = run("go\nquit\n");

        assert!(output.contains("usage: go depth <0-8>"));
        assert_eq!(output.matches("White>").count(), 2);
    }

    #[test]
    fn invalid_play_usage_prints_usage_and_continues() {
        let output = run("play depth -1\nquit\n");

        assert!(output.contains("usage: play depth <0-8>"));
        assert_eq!(output.matches("White>").count(), 2);
    }

    #[test]
    fn go_depth_nine_is_rejected() {
        let output = run("go depth 9\nquit\n");

        assert!(output.contains("usage: go depth <0-8>"));
    }

    #[test]
    fn play_after_king_capture_prints_result_and_exits() {
        let output = run("e2e4\nf7f6\nd1h5\na7a6\nplay depth 1\n");

        assert!(output.contains("bestmove h5e8"));
        assert!(output.contains("White wins"));
    }
}
