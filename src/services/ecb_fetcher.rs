use crate::error::ApiError;
use crate::models::{DailyRate, EcbEnvelope};
use std::time::Duration;

const USER_AGENT: &str = "Currency-API/0.1.0";
const TIMEOUT_SECONDS: u64 = 30;

pub struct EcbFetcher {
    pub(crate) client: reqwest::Client,
    pub(crate) ecb_url: String,
}

impl EcbFetcher {
    pub fn new(ecb_url: String) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(TIMEOUT_SECONDS))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, ecb_url }
    }

    /// Fetch and parse ECB XML data into DailyRate
    pub async fn fetch_rates(&self) -> Result<DailyRate, ApiError> {
        tracing::info!("Fetching exchange rates from ECB: {}", self.ecb_url);

        // Fetch XML
        let response = self
            .client
            .get(&self.ecb_url)
            .send()
            .await
            .map_err(|e| ApiError::EcbFetchError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ApiError::EcbFetchError(format!(
                "ECB returned status: {}",
                response.status()
            )));
        }

        let xml_content = response
            .text()
            .await
            .map_err(|e| ApiError::EcbFetchError(format!("Failed to read response: {}", e)))?;

        // Parse XML
        self.parse_ecb_xml(&xml_content)
    }

    /// Parse ECB XML format into DailyRate
    fn parse_ecb_xml(&self, xml: &str) -> Result<DailyRate, ApiError> {
        // Parse with quick-xml
        let envelope: EcbEnvelope = quick_xml::de::from_str(xml)
            .map_err(|e| ApiError::XmlParseError(format!("Failed to parse XML: {}", e)))?;

        let time_cube = envelope.cube.time_cube;
        let daily_rate = DailyRate::from_ecb_data(time_cube.time, time_cube.rates)
            .map_err(|e| ApiError::XmlParseError(e))?;

        // Validate date format
        daily_rate
            .validate_date()
            .map_err(|e| ApiError::XmlParseError(e))?;

        tracing::info!(
            "Successfully parsed {} exchange rates for {}",
            daily_rate.rates.len(),
            daily_rate.date
        );

        Ok(daily_rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ecb_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<gesmes:Envelope xmlns:gesmes="http://www.gesmes.org/xml/2002-08-01" xmlns="http://www.ecb.int/vocabulary/2002-08-01/eurofxref">
    <Cube>
        <Cube time="2024-12-04">
            <Cube currency="USD" rate="1.0534"/>
            <Cube currency="JPY" rate="158.23"/>
            <Cube currency="GBP" rate="0.8345"/>
        </Cube>
    </Cube>
</gesmes:Envelope>"#;

        let fetcher = EcbFetcher::new("http://example.com".to_string());
        let result = fetcher.parse_ecb_xml(xml).unwrap();

        use rust_decimal_macros::dec;

        assert_eq!(result.date, "2024-12-04");
        assert_eq!(result.base, "EUR");
        assert_eq!(result.rates.len(), 4); // USD, JPY, GBP + EUR
        assert_eq!(result.rates["USD"], dec!(1.0534));
        assert_eq!(result.rates["JPY"], dec!(158.23));
        assert_eq!(result.rates["EUR"], dec!(1.0)); // EUR added automatically
    }
}
