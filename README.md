# Design notes

I'm a beginner at Rust and this project took me about 8 hours to complete.

## Build and run


```sh
make
./parser fixtures/example.txt
```

## Approach

I used a `BufReader` to read one line at a time and a
`BTreeMap<String, Vec<usize>>` to store each word and its positions. The map inherently
keeps words in lexicographic order, so printing the result doesn't require a
separate sorting step. Positions are added as the file is read and are already
in ascending order.

`read_line()` handles UTF-8 and
keeps the newline and carriage return, which need to count toward the offsets.
After each read, the line string is cleared. The map owns its word
strings, so reusing the input string doesn't affect the index and completed words
are cloned before the working word is cleared.

Output is buffered with `BufWriter` and explicitly flushed to catch write
errors. It builds the index before printing, so a file read error won't leave
a partial word listing. An output error can still leave partial output.

## Word and position rules

Words end at the six separators listed in the assignment: space, tab, newline,
vertical tab, form feed and carriage return. Case and punctuation stay as they
appear in the file. A nonbreaking space is part of a word. Empty words are
ignored, and the last word is included even without a final newline.

I interpreted a character index as the number of Rust `char` values before a
word. This handles non-ASCII characters without counting their extra UTF-8
bytes. In `café dog`, `dog` starts at 5.

Sorting uses Rust's normal string order, which is case-sensitive and isn't
locale-specific. The input must be valid UTF-8. Positions use `usize`, with the
assumption that the file contains fewer than `usize::MAX` characters.

## Memory and performance

The main tradeoff is that this program keeps the entire index in RAM, and the longest line also
needs to fit in memory. A file without newlines is treated as one line, so in
that case the line string holds the entire input. Repeating a word still uses
more memory because every occurrence adds a position.

A limitation of this approach is that the longest line and the accumulated index 
must fit in RAM. Given more time, my next step would be to process input in 
bounded chunks and use external sorting to keep the index from growing 
indefinitely in memory. That would add implementation complexity, particularly 
around temporary files and merging results.

For C input bytes, N word occurrences and U unique words, the work is roughly
O(C + N log(U + 1)), plus output, assuming bounded word lengths. String
comparisons depend on word length; with maximum word length L, a conservative
bound is O(C + N L log(U + 1)), plus output. Memory is O(R + S + N), where R is
the longest line in bytes and S is the total size of unique word strings.

## Errors and tests

The program requires one filename. Missing or extra arguments, file-open and
read failures, invalid UTF-8, and output write or flush failures are reported
on stderr with a nonzero exit status.

Tests cover the example, empty input, all six separators, CRLF, a missing final
newline, case, punctuation, non-ASCII positions, nonbreaking space and invalid
UTF-8. Command-line tests check the example output and invalid arguments.

Additional tests check blank lines and leading whitespace, prefix and symbol
ordering, emoji offsets, a line longer than the reader buffer, and 10,000
occurrences of one word. A separate 17 GiB fixture tests reading a file larger
than this Mac's RAM with a small index. It is excluded from the automated test
suite; instructions for running it are below.

## Running the 17 GiB test

The test file is larger than this Mac's 16 GiB RAM and is excluded from the repo. It contains two words separated by short lines of
whitespace, so its index stays small (See memory and performance above).

If the file is missing, generate it first. This writes a 17 GiB file and
also creates the expected-output file:

```sh
python3 scripts/generate_large_file.py --size-gib 17 --output fixtures/large/over-ram.txt
```

Build and run the full test on macOS:

```sh
make
/usr/bin/time -l ./parser fixtures/large/over-ram.txt > target/over-ram.actual.txt
diff -u fixtures/large/over-ram.expected.txt target/over-ram.actual.txt
```

The expected output is:

```text
alpha 0
omega 18253611003
```

```sh
rm fixtures/large/over-ram.txt fixtures/large/over-ram.expected.txt
```

## AI assistance

Before starting, I asked ChatGPT to review my plan. It pointed out that non-ASCII
characters need to be counted as characters rather than bytes, which I accounted 
for in the implementation. After implementation, I asked it to write test cases. 
In particular, I had it generate the script for creating the 17 GiB file.
