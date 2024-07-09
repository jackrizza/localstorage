use localstorage::datastore::{DataStore, Op};
use rand::Rng;
use std::iter;
use std::process::Command;
use std::thread;

const MAX_THREADS: i32 = 256;

fn generate(len: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    let one_char = || CHARSET[rng.gen_range(0..CHARSET.len())] as char;
    iter::repeat_with(one_char).take(len).collect()
}

fn main() {
    let mut ds = Box::new(DataStore::new());
    ds.listen();

    for _ in 0..MAX_THREADS {
        let add_sender = ds.sender.clone();
        thread::spawn(move || loop {
            let mut child = Command::new("sleep").arg("0.05").spawn().unwrap();
            let _result = child.wait().unwrap();
            let lock = add_sender.lock().unwrap();
            lock.send(Op::Upsert(generate(16), generate(128))).unwrap();
        });
    }

    for _ in 0..MAX_THREADS / 2 {
        let remove_sender = ds.sender.clone();
        thread::spawn(move || loop {
            let mut child = Command::new("sleep").arg("0.2").spawn().unwrap();
            let _result = child.wait().unwrap();
            let lock = remove_sender.lock().unwrap();
            lock.send(Op::RemoveRandom).unwrap();
        });
    }

    loop {
        let mut child = Command::new("sleep").arg("1").spawn().unwrap();
        let _result = child.wait().unwrap();
        print!("\x1B[2J\x1B[1;1H");
        println!("Length of hashmap : {}", ds.all().len());
    }
}
