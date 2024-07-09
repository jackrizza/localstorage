use rand::Rng;
use std::{
    collections::HashMap,
    sync::{
        mpsc,
        mpsc::{Receiver, Sender},
        {Arc, Mutex},
    },
    thread,
};
#[derive(Debug, Clone)]
pub enum Op<B> {
    Upsert(String, B),
    Remove(String),
    NewTab(String),
    RemoveRandom, // only for testing... should not be used in production
}

pub struct DataStore<B> {
    store: Arc<Mutex<HashMap<String, B>>>,
    pub sender: Arc<Mutex<Sender<Op<B>>>>,
    pub receiver: Arc<Mutex<Receiver<Op<B>>>>,
}

impl<B> AsRef<DataStore<B>> for DataStore<B> {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<B> DataStore<B>
where
    B: Clone + Default + Send + Sync + 'static,
{
    pub fn new() -> DataStore<B> {
        let (sender, receiver) = mpsc::channel();
        DataStore {
            store: Arc::new(Mutex::new(HashMap::new())),
            sender: Arc::new(Mutex::new(sender)),
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    pub fn all(&self) -> HashMap<String, B> {
        let locked_store = self.store.lock().unwrap();
        locked_store.clone()
    }

    pub fn get(&self, key: String) -> Option<B> {
        let locked_store = self.store.lock().unwrap();
        match locked_store.get(&key) {
            Some(value) => Some(value.clone()),
            None => None,
        }
    }

    pub fn listen(&mut self) {
        let receiver = self.receiver.clone();
        let store = self.store.clone();
        thread::spawn(move || loop {
            let locked_receiver = receiver.lock().unwrap();
            let packet = match locked_receiver.recv() {
                Ok(msg) => msg,
                Err(e) => return eprintln!("{}", e),
            };

            let mut locked_store = store.lock().unwrap();
            match packet {
                Op::Upsert(key, value) => {
                    locked_store.insert(key, value);
                }
                Op::Remove(key) => {
                    locked_store.remove(&key);
                }
                Op::RemoveRandom => {
                    let keys = locked_store.keys().cloned().collect::<Vec<String>>();
                    let key = keys[rand::thread_rng().gen_range(0..keys.len())].clone();
                    locked_store.remove(&key);
                }
                Op::NewTab(key) => {
                    locked_store.insert(format!("tab_{}", key), B::default());
                }
            }
        });
    }

    pub fn send(&self, msg: Op<B>) {
        let sender = self.sender.lock().unwrap();
        sender.send(msg).unwrap();
    }
}
