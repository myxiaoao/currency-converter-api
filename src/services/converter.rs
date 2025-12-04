use crate::error::ApiError;
use crate::models::DailyRate;
use std::collections::HashMap;

/// Rebase exchange rates from EUR to any other currency
/// This is a direct Rust translation of the provided TypeScript logic
pub fn rebase_rates(daily_rate: &DailyRate, new_base: &str) -> Result<DailyRate, ApiError> {
    let new_base = new_base.to_uppercase();

    // If already the requested base, return clone
    if new_base == daily_rate.base {
        return Ok(daily_rate.clone());
    }

    // Get the rate for the new base currency relative to current base
    let base_rate = daily_rate
        .rates
        .get(&new_base)
        .ok_or_else(|| ApiError::CurrencyNotFound(new_base.clone()))?;

    // Recalculate all rates relative to new base
    // Formula: new_rate = old_rate / base_rate
    let mut new_rates = HashMap::new();
    for (currency, rate) in &daily_rate.rates {
        new_rates.insert(currency.clone(), rate / base_rate);
    }

    // Add the original base currency back (e.g., EUR when switching from EUR to USD)
    // Since original data doesn't contain EUR->EUR
    new_rates.insert(daily_rate.base.clone(), 1.0 / base_rate);

    Ok(DailyRate {
        date: daily_rate.date.clone(),
        base: new_base,
        rates: new_rates,
    })
}

/// Calculate conversion between two currencies
pub fn convert_currency(
    daily_rate: &DailyRate,
    from: &str,
    to: &str,
    amount: f64,
) -> Result<(f64, f64), ApiError> {
    let from = from.to_uppercase();
    let to = to.to_uppercase();

    // Special case: same currency
    if from == to {
        return Ok((amount, 1.0));
    }

    // Rebase to the source currency
    let rebased = rebase_rates(daily_rate, &from)?;

    // Get the rate for target currency
    let rate = rebased
        .rates
        .get(&to)
        .ok_or_else(|| ApiError::CurrencyNotFound(to.clone()))?;

    Ok((amount * rate, *rate))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_rates() -> DailyRate {
        let mut rates = HashMap::new();
        rates.insert("USD".to_string(), 1.05);
        rates.insert("GBP".to_string(), 0.85);
        rates.insert("JPY".to_string(), 158.2);
        rates.insert("EUR".to_string(), 1.0);

        DailyRate {
            date: "2024-12-04".to_string(),
            base: "EUR".to_string(),
            rates,
        }
    }

    #[test]
    fn test_rebase_same_currency() {
        let rates = create_test_rates();
        let result = rebase_rates(&rates, "EUR").unwrap();
        assert_eq!(result.base, "EUR");
        assert_eq!(result.rates["USD"], 1.05);
    }

    #[test]
    fn test_rebase_eur_to_usd() {
        let rates = create_test_rates();
        let result = rebase_rates(&rates, "USD").unwrap();

        assert_eq!(result.base, "USD");
        // EUR in terms of USD: 1.0 / 1.05 ≈ 0.9524
        assert!((result.rates["EUR"] - 0.9524).abs() < 0.01);
        // USD in terms of USD should be 1.0
        assert!((result.rates["USD"] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cross_rate_usd_to_jpy() {
        let rates = create_test_rates();
        let result = rebase_rates(&rates, "USD").unwrap();

        // JPY in terms of USD: 158.2 / 1.05 ≈ 150.67
        let expected = 158.2 / 1.05;
        assert!((result.rates["JPY"] - expected).abs() < 0.01);
    }

    #[test]
    fn test_convert_same_currency() {
        let rates = create_test_rates();
        let (result, rate) = convert_currency(&rates, "USD", "USD", 100.0).unwrap();

        assert_eq!(result, 100.0);
        assert_eq!(rate, 1.0);
    }

    #[test]
    fn test_convert_usd_to_eur() {
        let rates = create_test_rates();
        let (result, rate) = convert_currency(&rates, "USD", "EUR", 100.0).unwrap();

        // 100 USD in EUR: 100 * (1.0 / 1.05) ≈ 95.24
        assert!((result - 95.24).abs() < 0.1);
        assert!((rate - 0.9524).abs() < 0.01);
    }

    #[test]
    fn test_convert_unknown_currency() {
        let rates = create_test_rates();
        let result = convert_currency(&rates, "USD", "XXX", 100.0);

        assert!(result.is_err());
    }
}
