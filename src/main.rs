use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::process::ExitCode;

type WordIndex = BTreeMap<String, Vec<usize>>;

fn finish_word(word: &mut String, start: usize, index: &mut WordIndex) {
    if !word.is_empty() {
        let completed_word = word.clone();
        word.clear();
        if let Some(positions) = index.get_mut(&completed_word) {
            positions.push(start);
        } else {
            index.insert(completed_word, vec![start]);
        }
    }
}

fn build_index<R: BufRead>(mut reader: R) -> io::Result<WordIndex> {
    let mut index = BTreeMap::new();
    let mut line = String::new();
    let mut word = String::new();
    let mut word_start = 0;
    let mut offset = 0;

    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            break;
        }

        // read line keeps the newline so it still counts toward the offset
        for character in line.chars() {
            if matches!(character, ' ' | '\t' | '\n' | '\u{b}' | '\u{c}' | '\r') {
                finish_word(&mut word, word_start, &mut index);
            } else {
                if word.is_empty() {
                    word_start = offset;
                }
                word.push(character);
            }
            offset += 1;
        }
    }

    // last word in the file might not have a separator after it
    finish_word(&mut word, word_start, &mut index);
    Ok(index)
}

fn write_index<W: Write>(writer: &mut W, index: &WordIndex) -> io::Result<()> {
    // BTreeMap already sorts words and scanning in order already sorts positions
    for (word, positions) in index {
        write!(writer, "{word}")?;
        for position in positions {
            write!(writer, " {position}")?;
        }
        writeln!(writer)?;
    }
    writer.flush()
}

fn run() -> io::Result<()> {
    let mut args = std::env::args_os().skip(1);
    let filename = args
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Usage: parser <filename>"))?;
    if args.next().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "expected exactly one filename",
        ));
    }
    let reader = BufReader::new(File::open(PathBuf::from(filename))?);
    let index = build_index(reader)?;
    let stdout = io::stdout();
    write_index(&mut BufWriter::new(stdout.lock()), &index)
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("parser: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn output(input: &str) -> String {
        let index = build_index(Cursor::new(input)).unwrap();
        let mut bytes = Vec::new();
        write_index(&mut bytes, &index).unwrap();
        String::from_utf8(bytes).unwrap()
    }

    #[test]
    fn example() {
        assert_eq!(output("zzz aaa bbb aaa"), "aaa 4 12\nbbb 8\nzzz 0\n");
    }

    #[test]
    fn whitespace_and_final_word() {
        assert_eq!(output(""), "");
        assert_eq!(output(" \t\n\r\u{b}\u{c}"), "");
        assert_eq!(
            output("a b\tc\nd\u{b}e\u{c}f\rg"),
            "a 0\nb 2\nc 4\nd 6\ne 8\nf 10\ng 12\n"
        );
        assert_eq!(output("a\r\na"), "a 0 3\n");
        assert_eq!(output("hello"), "hello 0\n");
    }

    #[test]
    fn words_keep_their_content() {
        assert_eq!(output("Apple apple hi!"), "Apple 0\napple 6\nhi! 12\n");
        assert_eq!(output("café dog\ncafé"), "café 0 9\ndog 5\n");
        assert_eq!(output("a\u{a0}b c"), "a\u{a0}b 0\nc 4\n");
    }

    #[test]
    fn blank_lines_and_leading_whitespace() {
        assert_eq!(output("\n \tcat\r\n\ncat "), "cat 3 9\n");
    }

    #[test]
    fn prefixes_and_symbols_sort_correctly() {
        assert_eq!(
            output("ab a abc a:b : Z"),
            ": 13\nZ 15\na 3\na:b 9\nab 0\nabc 5\n"
        );
    }

    #[test]
    fn emoji_positions_are_character_indices() {
        assert_eq!(output("🚀 x\n🚀"), "x 2\n🚀 0 4\n");
    }

    #[test]
    fn line_longer_than_reader_buffer() {
        let word = "é".repeat(20000);
        let input = format!("{word} x\n{word}");
        let reader = BufReader::new(Cursor::new(input));
        let index = build_index(reader).unwrap();
        assert_eq!(index.get(&word).unwrap(), &vec![0, 20003]);
        assert_eq!(index.get("x").unwrap(), &vec![20001]);
    }

    #[test]
    fn many_occurrences_are_kept_in_order() {
        let input = "word\n".repeat(10000);
        let index = build_index(Cursor::new(input)).unwrap();
        let positions = index.get("word").unwrap();
        assert_eq!(index.len(), 1);
        assert_eq!(positions.len(), 10000);
        for (i, position) in positions.iter().enumerate() {
            assert_eq!(*position, i * 5);
        }
    }

    #[test]
    fn invalid_utf8() {
        assert!(build_index(Cursor::new(b"hello \xff")).is_err());
    }
}
