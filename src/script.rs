use nom::{
    IResult, bytes::complete::take_while1, character::complete::space1, combinator::all_consuming,
    multi::separated_list1,
};

use crate::context::Context;

enum Command {
    Point(String),
    Distinct(String, String),
    Circle(String, String, String),
    Intersect(String, String, String),
    Equal(String, String, String, String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ScriptError {
    pub line: usize,
    pub message: String,
}

impl std::fmt::Display for ScriptError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "line {}: {}", self.line, self.message)
    }
}

pub fn run(source: &str) -> Result<(), ScriptError> {
    let mut context = Context::default();
    for (line_index, source_line) in source.lines().enumerate() {
        let line = source_line.trim();
        if line.is_empty() {
            continue;
        }

        let command = parse_command(line).map_err(|()| ScriptError {
            line: line_index + 1,
            message: "invalid command syntax".to_owned(),
        })?;
        execute(&mut context, command).map_err(|message| ScriptError {
            line: line_index + 1,
            message,
        })?;
    }
    Ok(())
}

fn execute(context: &mut Context, command: Command) -> Result<(), String> {
    match command {
        Command::Point(name) => context
            .given_point(name)
            .map(|_| ())
            .map_err(|error| format!("{error:?}")),
        Command::Distinct(a, b) => context
            .assume_distinct_points(&a, &b)
            .map_err(|error| format!("{error:?}")),
        Command::Circle(name, center, through) => context
            .construct_circle(name, &center, &through)
            .map(|_| ())
            .map_err(|error| format!("{error:?}")),
        Command::Intersect(name, alpha, beta) => context
            .construct_intersection(name, &alpha, &beta)
            .map(|_| ())
            .map_err(|error| format!("{error:?}")),
        Command::Equal(a, b, c, d) => match context.has_equal_lengths(&a, &b, &c, &d) {
            Ok(true) => Ok(()),
            Ok(false) => Err(format!("assertion failed: equal {a} {b} {c} {d}")),
            Err(error) => Err(format!("{error:?}")),
        },
    }
}

fn identifier(input: &str) -> IResult<&str, &str> {
    take_while1(|character: char| character.is_ascii_alphanumeric() || character == '_')(input)
}

fn parse_command(input: &str) -> Result<Command, ()> {
    let (_, tokens) = all_consuming(separated_list1(space1, identifier))(input).map_err(|_| ())?;
    match tokens.as_slice() {
        ["point", name] => Ok(Command::Point((*name).to_owned())),
        ["distinct", a, b] => Ok(Command::Distinct((*a).to_owned(), (*b).to_owned())),
        ["circle", name, center, through] => Ok(Command::Circle(
            (*name).to_owned(),
            (*center).to_owned(),
            (*through).to_owned(),
        )),
        ["intersect", name, alpha, beta] => Ok(Command::Intersect(
            (*name).to_owned(),
            (*alpha).to_owned(),
            (*beta).to_owned(),
        )),
        ["equal", a, b, c, d] => Ok(Command::Equal(
            (*a).to_owned(),
            (*b).to_owned(),
            (*c).to_owned(),
            (*d).to_owned(),
        )),
        _ => Err(()),
    }
}

#[cfg(test)]
mod tests {
    use super::run;

    #[test]
    fn checks_the_i1_example() {
        assert_eq!(run(include_str!("../examples/i1.gnomon")), Ok(()));
    }

    #[test]
    fn false_equal_assertion_reports_its_line() {
        let source =
            include_str!("../examples/i1.gnomon").replace("equal B C C A", "equal A B A A");
        let error = run(&source).unwrap_err();
        assert_eq!(error.line, 9);
        assert!(error.message.starts_with("assertion failed:"));
    }

    #[test]
    fn syntax_error_reports_its_line() {
        let error = run("\npoint A extra\n").unwrap_err();
        assert_eq!(error.line, 2);
        assert_eq!(error.message, "invalid command syntax");
    }
}
