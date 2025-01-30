use crate::client::client::Request;
use serde::de::{DeserializeOwned, Error};
pub struct Json<T>(pub T);

impl Request {
    fn json<T: DeserializeOwned>(self) -> Result<Json<T>, serde_json::Error> {
        if let Some(body) = &self.body {
            let parsed = serde_json::from_slice::<T>(&body)?;
            Ok(Json(parsed))
        } else {
            Err(serde_json::Error::custom("ayy"))
        }
    }

    fn string(self) -> Result<String, ()> {
        if let Some(body) = self.body {
            match String::from_utf8(body) {
                Ok(string) => return Ok(string),
                Err(_) => return Err(()),
            };
        }
        Err(())
    }
}
