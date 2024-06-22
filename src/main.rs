use std::{
    collections::{hash_map::Entry, HashMap},
    env,
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
    str::from_utf8,
    sync::mpsc,
    thread,
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

struct StationData {
    min: num,
    max: num,
    mean: Mean,
}

impl StationData {
    fn new(value: num) -> Self {
        Self {
            min: value,
            max: value,
            mean: Mean::new(value),
        }
    }
    fn update(&mut self, value: num) {
        self.min = num::min(self.min, value);
        self.max = num::max(self.max, value);
        self.mean.update(value);
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

    let (tx, rx): (mpsc::Sender<(Vec<u8>, Vec<u8>)>, _) = mpsc::channel();

    let acumulator = thread::spawn(move || {
        let mut all_stations: HashMap<_, StationData> = HashMap::new();

        while let Ok((station_buf, temp_buf)) = rx.recv() {
            let temp_s = from_utf8(&temp_buf).unwrap().trim();
            let temp = temp_s.parse::<num>().unwrap();
            let mut station = String::from(from_utf8(&station_buf).unwrap());
            station.pop();

            match all_stations.entry(station) {
                Entry::Occupied(ref mut s) => {
                    s.get_mut().update(temp);
                }
                Entry::Vacant(v) => {
                    v.insert(StationData::new(temp));
                }
            }
        }

        all_stations
    });

    loop {
        let mut station_buf = Vec::new();
        let mut temp_buf = Vec::new();
        if reader.read_until(b';', &mut station_buf)? == 0 {
            break;
        }
        if reader.read_until(b'\n', &mut temp_buf)? == 0 {
            break;
        }
        tx.send((station_buf, temp_buf))?;
    }
    eprintln!("DONE");
    drop(tx);

    print!("{}", "{");
    for (k, v) in acumulator.join().unwrap() {
        print!("{k}={:.1}/{:.1}/{:.1}, ", v.min, v.max, v.mean.value);
    }
    print!("{}", "}");

    Ok(())
}
