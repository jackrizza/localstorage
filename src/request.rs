use std::future::Future;

use http_body_util::BodyExt;
use hyper::body::Bytes;
use hyper::Request;
use hyper_util::rt::TokioIo;
use serde_json::Value;
use tokio::net::TcpStream;

use crate::datastore::DataStore;

// A simple type alias so as to DRY.
type HyperResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub struct Requester {
    base_url: String,
    target_db: String,
}

pub fn blocking_req<T>(func: impl Future<Output = T>) -> T {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { func.await })
}

impl Requester {
    pub fn new(base_url: &str, target_db: &str) -> Requester {
        Requester {
            base_url: base_url.to_string(),
            target_db: target_db.to_string(),
        }
    }

    pub async fn search(&self, body: String) -> HyperResult<Value> {
        let url = format!("{}/api/{}", self.base_url, self.target_db);
        Requester::fetch_url(url, body).await
    }

    pub async fn validate_user(&self, body: String) -> HyperResult<Value> {
        let url = format!("{}/{}/validate_user", self.base_url, self.target_db);
        Requester::fetch_url(url, body.clone()).await
    }

    pub async fn validate_session(&self, body: String) -> HyperResult<Value> {
        let url = format!("{}/{}/validate_session", self.base_url, self.target_db);
        Requester::fetch_url(url, body).await
    }

    pub async fn get(&self, body: String) -> HyperResult<Value> {
        let url = format!("{}/api/get/{}", self.base_url, self.target_db);
        Requester::fetch_url(url, body).await
    }

    pub async fn update(&self, body: String) -> HyperResult<Value> {
        let url = format!("{}/api/update/{}", self.base_url, self.target_db);
        Requester::fetch_url(url, body).await
    }

    pub async fn delete(&self, body: String) -> HyperResult<Value> {
        let url = format!("{}/api/delete/{}", self.base_url, self.target_db);
        Requester::fetch_url(url, body).await
    }

    pub async fn create(&self, body: String) -> HyperResult<Value> {
        let url = format!("{}/api/create/{}", self.base_url, self.target_db);
        Requester::fetch_url(url, body).await
    }

    async fn fetch_url(url: String, body: String) -> HyperResult<Value> {
        // Parse our URL...
        let url = url.parse::<hyper::Uri>()?;
        let host = url.host().expect("uri has no host");
        let port = url.port_u16().unwrap_or(80);
        let address = format!("{}:{}", host, port);

        // Open a TCP connection to the remote host
        let stream = TcpStream::connect(address).await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        // Create the Hyper client
        let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;

        // Spawn a task to poll the connection, driving the HTTP state
        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                println!("Connection failed: {:?}", err);
            }
        });
        // The authority of our URL will be the hostname of the httpbin remote
        let authority = url.authority().unwrap().clone();

        // Create an HTTP request with an empty body and a HOST header
        let req = Request::builder()
            .method("POST")
            .uri(url)
            .header(hyper::header::HOST, authority.as_str())
            .body(body)?;

        let mut chunks: Vec<Bytes> = Vec::new();
        // Await the response...
        let mut res = sender.send_request(req).await?;
        println!("Response status: {}", res.status());
        // Stream the body, writing each frame to stdout as it arrives
        while let Some(next) = res.frame().await {
            let frame = next?;
            if let Some(chunk) = frame.data_ref() {
                chunks.push(chunk.clone());
            }
        }

        let out = serde_json::from_slice::<Value>(&chunks.concat())?;

        Ok(out)
    }
}

use crate::models::{ApiBody, Data, GetApi};

use bidbook_rest::logic::auth::{Auth, Session as AuthSession};

pub fn quick_request<T>(url: &str, db_target: &str, ds: &DataStore<Value>) -> Result<Vec<T>, String>
where
    T: serde::de::DeserializeOwned,
{
    let body_data: ApiBody<GetApi<Data<String>>> = ApiBody::new(
        match ds.get("session".into()) {
            Some(session) => {
                let session: AuthSession = serde_json::from_value(session).unwrap();
                session.session_key
            }
            None => Some("".into()),
        },
        GetApi {
            ..GetApi::default()
        },
    );
    let get_api_json = serde_json::to_string(&body_data).unwrap();

    let req = Requester::new(url, db_target);
    let data = blocking_req(req.get(get_api_json));
    let data = data.unwrap();
    let get = match serde_json::from_value::<GetApi<Vec<T>>>(data.clone()) {
        Ok(db) => Ok(db.data),
        Err(e) => Err(e),
    };

    match get {
        Ok(data) => Ok(data),
        Err(e) => Err(e.to_string()),
    }
}
