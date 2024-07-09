use localstorage::datastore::{DataStore, Op};
use localstorage::models::User;
use localstorage::request::Requester;
use serde_json::Value;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

fn main() {
    let mut ds: DataStore<Value> = DataStore::new();
    ds.listen();
    thread(&ds.sender);
    loop {
        print!("\x1B[2J\x1B[1;1H");
        println!("auth : {:#?}", ds.all());
        if ds.all().len() > 0 {
            break;
        }
    }
}

fn thread(ds_sender: &Arc<Mutex<Sender<Op<Value>>>>) {
    let ds_sender = ds_sender.clone();
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let lock = ds_sender.lock().unwrap();
            let req = Requester::new("http://localhost:8080", "auth");
            // log in
            let login = User {
                username: Some("jack".into()),
                password: Some("jack".into()),
            };
            let login_body = serde_json::to_string(&login).unwrap();
            let login = req.validate_user(login_body).await;
            match login {
                Ok(session) => {
                    lock.send(Op::Upsert("session".into(), session)).unwrap();
                }
                Err(e) => eprintln!("{}", e),
            }
        });
}
