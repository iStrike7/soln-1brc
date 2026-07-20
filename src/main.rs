use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::{BufRead, BufReader, Read, Seek, SeekFrom, Write},
};

const INPUT_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../1brc/data/measurements.txt"
);

const SEMI: u8 = b';';
const MINUS: u8 = b'-';
const ZERO: u8 = b'0';
const NEWLINE: u8 = b'\n';

fn main() {
    let input_file = File::open(INPUT_PATH).unwrap();
    let mut reader = BufReader::with_capacity(256, input_file);

    let num_threads = std::thread::available_parallelism().unwrap().get();
    let read_len = reader.get_ref().metadata().unwrap().len();
    let chunk_size = read_len / num_threads as u64;
    let (tx, rx) = std::sync::mpsc::sync_channel(num_threads);
    std::thread::scope(|s| {
        for _ in 0..num_threads {
            let start = reader.stream_position().unwrap();
            let _ = reader.seek(SeekFrom::Current(chunk_size.try_into().unwrap()));
            let _ = reader.skip_until(NEWLINE).unwrap();
            let end = reader.stream_position().unwrap().min(read_len);
            let tx = tx.clone();
            s.spawn(move || {
                let _ = tx.send(process_chunk(start, end));
            });
        }
    });

    drop(tx);
    let mut bmap = BTreeMap::new();
    for map in rx {
        for (loc, (v_n, v_x, v_c, v_a)) in map {
            let loc = std::str::from_utf8(&loc).unwrap();
            let (n, x, c, a) = match bmap.get_mut(loc) {
                Some(v) => v,
                None => bmap
                    .entry(loc.to_string())
                    .or_insert((i16::MAX, i16::MIN, 0, 0)),
            };

            if v_n < *n {
                *n = v_n;
            };
            if v_x > *x {
                *x = v_x;
            };
            *a = *a + v_a as i64;
            *c = *c + v_c;
        }
    }

    print_to_stdout(bmap);
}

fn process_chunk(start: u64, end: u64) -> HashMap<Vec<u8>, (i16, i16, u32, i64)> {
    let input_file = File::open(INPUT_PATH).unwrap();
    let mut reader = BufReader::with_capacity(128 * 1024, input_file);
    let _ = reader.seek(SeekFrom::Start(start));
    let mut reader = reader.take(end - start);
    let mut map = HashMap::<Vec<u8>, (i16, i16, u32, i64)>::with_capacity(10000);
    let mut buf = Vec::with_capacity(128);
    loop {
        buf.clear();
        // Split on byte '\n' since with utf-8 all multibyte chars start with non-00 prefixes
        // We use read_until to avoid a fresh String memory alloc that split() does and the utf8 parsing that read_line() does
        let bytes_read = reader.read_until(NEWLINE, &mut buf).unwrap();
        if bytes_read == 0 {
            break; // EOF
        }
        let line = &buf[..bytes_read - 1];
        let pos = line.iter().position(|&x| x == SEMI).unwrap();
        let loc = &line[..pos];
        let val = parse_temp(&line[pos + 1..]);
        update_hashmap(loc, val, &mut map);
    }
    map
}

fn update_hashmap(loc: &[u8], val: i16, map: &mut HashMap<Vec<u8>, (i16, i16, u32, i64)>) {
    let (n, x, c, a) = match map.get_mut(loc) {
        Some(v) => v,
        None => map
            .entry(loc.to_vec())
            .or_insert((i16::MAX, i16::MIN, 0, 0)),
    };

    // using cold_path here due to https://curiouscoding.nl/posts/1brc/#branchy-min-max-1-dot-37s
    if val < *n {
        std::hint::cold_path();
        *n = val;
    };
    if val > *x {
        std::hint::cold_path();
        *x = val;
    };
    *a = *a + val as i64;
    *c = *c + 1;
}

// Read the last two digits - always there - always in the same position from end - these are always the ones and tens place
// Read the first digit - maybe at 0 or 1 - depending on whether the first byte is a '-' - this maybe the hundreds place or just the tens place again
// Set the first digit to 0 if the slice is not supposed to have a hundreds digit depending on the len of the slice (and whether a sign byte was skipped)
fn parse_temp(slice: &[u8]) -> i16 {
    let len = slice.len();
    assert!(len >= 3);
    let (sign, first_idx) =
        std::hint::select_unpredictable(slice[0] == MINUS, (-1i16, 1), (1i16, 0));
    let v3 = i16::from(slice[len - 1] - ZERO);
    let v2 = i16::from(slice[len - 3] - ZERO);
    let is_dd = len - first_idx == 4;
    let mut first = slice[first_idx];
    first = std::hint::select_unpredictable(is_dd, first, ZERO);
    let v1 = i16::from(first - ZERO);
    (v3 + v2 * 10 + v1 * 100) * sign
}

#[test]
fn pt() {
    assert_eq!(parse_temp(b"0.0"), 0);
    assert_eq!(parse_temp(b"9.2"), 92);
    assert_eq!(parse_temp(b"-9.2"), -92);
    assert_eq!(parse_temp(b"98.2"), 982);
    assert_eq!(parse_temp(b"-98.2"), -982);
}

fn print_to_stdout(stats: BTreeMap<String, (i16, i16, u32, i64)>) {
    let stdout = std::io::stdout();
    let stdout = stdout.lock();
    let mut writer = std::io::BufWriter::new(stdout);
    write!(writer, "{{").unwrap();
    let mut stats = stats.into_iter().peekable();
    while let Some((station, (n, x, c, a))) = stats.next() {
        write!(
            writer,
            "{station}={:.1}/{:.1}/{:.1}",
            (n as f64) / 10.,
            (a as f64) / 10. / (c as f64),
            (x as f64) / 10.
        )
        .unwrap();
        if stats.peek().is_some() {
            write!(writer, ", ").unwrap();
        }
    }
    write!(writer, "}}").unwrap();
}
