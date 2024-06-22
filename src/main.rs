use std::{
    collections::{hash_map::Entry, HashMap},
    env,
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
    str::from_utf8,
    sync::mpsc,
};

type num = f64;

struct Mean {
    value: num,
    count: u64,
}

impl Mean {
    fn new(value: num) -> Self {
        Self { value, count: 1 }
    }

    fn update(&mut self, value: num) {
        self.count = self.count + 1;
        self.value = self.value + (value - self.value) / (self.count as num);
    }
}

#[test]
fn test_mean() {
    let mut mean = Mean::new(0.0);
    mean.update(2.0);
    assert_eq!(mean.value, 1.0);
    mean.update(1.0);
    assert_eq!(mean.value, 1.0);
    mean.update(80.0);
    assert_eq!(mean.value, 20.75);
}

fn main() -> Result<(), Box<dyn Error>> {

    let mut args = env::args();

    args.next();
    let path = PathBuf::from(args.next().expect("Usage: 1brc <path/to/measurements.txt>"));
    let file = File::options().read(true).open(&path).unwrap();
    let mut reader = BufReader::new(file);

    let mut all_stations: HashMap<_, Mean> = HashMap::new();

    let mut station_buf = Vec::new();
    let mut temp_buf = Vec::new();
    loop {
        temp_buf.clear();
        station_buf.clear();
        if reader.read_until(b';', &mut station_buf)? == 0 {
            break;
        }
        if reader.read_until(b'\n', &mut temp_buf)? == 0 {
            break;
        }

        let s = from_utf8(&temp_buf)?.trim();
        #[cfg(debug_assertions)]
        eprintln!("s={:?}", s);
        let temp = s.parse::<num>()?;
        #[cfg(debug_assertions)]
        eprintln!("{:x?} -> {temp:?}", station_buf);

        match all_stations.entry(station_buf.clone()) {
            Entry::Occupied(ref mut s) => {
                s.get_mut().update(temp);
            }
            Entry::Vacant(v) => {
                v.insert(Mean::new(temp));
            }
        }
    }
    eprintln!("DONE");

    print!("{}", "{");
    for (k, v) in all_stations {
        let name = from_utf8(&k)?.replace(";", "");
        print!("{name}=0.0/0.0/{}, ", v.value);
    }
    print!("{}", "}");

    Ok(())
}
