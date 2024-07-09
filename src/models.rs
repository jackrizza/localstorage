use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_key: Option<String>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Search {
    pub search_term: String,
    pub scope: Vec<String>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ApiBody<V> {
    session: Session,
    data: V,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct GetApi<V> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub db: Option<String>,
    pub data: V,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Data<V> {
    pub _id: V,
}

impl<V> ApiBody<V> {
    pub fn new(session_key: Option<String>, data: V) -> Self {
        ApiBody {
            session: Session { session_key },
            data,
        }
    }
}
