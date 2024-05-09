use bytes::Buf;
use reqwest::{redirect::Policy, Client, Url};
use url::ParseError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ParseError(#[from] ParseError),
    #[error(transparent)]
    NetworkError(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Thermostat {
    client: Client,
    base_url: Url,
}

impl Thermostat {
    pub fn create(endpoint: &str) -> Result<Self> {
        let base_url = Url::parse(endpoint).or_else(|e| match e {
            ParseError::RelativeUrlWithoutBase => Url::parse(&format!("http://{endpoint}")),
            err => Err(err),
        })?;
        let thermostat = Self {
            client: Client::builder()
                .redirect(Policy::limited(1))
                .build()
                .expect("unable to build reqwest client from builder"),
            base_url,
        };
        Ok(thermostat)
    }

    pub async fn get_status(&self) -> Result<RawThermostatData> {
        let body = self
            .client
            .get(self.base_url.join("thermostat.xml").unwrap())
            .send()
            .await?
            .bytes()
            .await?;
        let data = serde_xml_rs::from_reader(body.reader()).unwrap();
        Ok(data)
    }
}

#[allow(unused)]
#[derive(serde::Deserialize, Debug)]
pub struct RawThermostatData {
    pub setpoint: f64,
    pub temperature: f64,
    #[serde(deserialize_with = "de::opt_f64")]
    pub outside: Option<f64>,
    pub pause: bool,
    pub heating: bool,
}

mod de {
    use serde::{de::Error, Deserialize, Deserializer};

    pub fn opt_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if value == "---" {
            Ok(None)
        } else if let Ok(value) = value.parse() {
            Ok(Some(value))
        } else {
            Err(D::Error::custom(format!(
                "Unable to parse '{value}' as optional float"
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_message() {
        let xml_message = r#"<?xml version="1.0" encoding="utf-8"?>
        <thermostat>
                <setpoint>14.0</setpoint>
                <temperature>20.3</temperature>
                <outside>---</outside>
                <pause>1</pause>
                <heating>0</heating>
        </thermostat>"#;

        let message: RawThermostatData = serde_xml_rs::from_str(xml_message).unwrap();
        println!("{message:?}");
    }
}
