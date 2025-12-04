use crate::error::ApiError;
use crate::models::DailyRate;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Optimized O(1) currency conversion without full rebase
/// Directly calculates cross-rate: (Base->To) / (Base->From)
///
/// Example:
/// - ECB provides: EUR->USD (1.05), EUR->JPY (158.2)
/// - To get USD->JPY: 158.2 / 1.05 = 150.67
/// - Result: 1 USD = 150.67 JPY
pub fn convert_currency(
    daily_rate: &DailyRate,
    from: &str,
    to: &str,
    amount: Decimal,
) -> Result<(Decimal, Decimal), ApiError> {
    let from = from.to_uppercase();
    let to = to.to_uppercase();
    let base = &daily_rate.base;

    // Special case: same currency
    if from == to {
        return Ok((amount, Decimal::ONE));
    }

    // 1. Get Base -> From rate (e.g., EUR -> USD)
    let from_rate = if from == *base {
        Decimal::ONE
    } else {
        *daily_rate
            .rates
            .get(&from)
            .ok_or_else(|| ApiError::CurrencyNotFound(from.clone()))?
    };

    // 2. Get Base -> To rate (e.g., EUR -> JPY)
    let to_rate = if to == *base {
        Decimal::ONE
    } else {
        *daily_rate
            .rates
            .get(&to)
            .ok_or_else(|| ApiError::CurrencyNotFound(to.clone()))?
    };

    // 3. Calculate cross-rate: to_rate / from_rate
    // Example: JPY/USD = (EUR->JPY) / (EUR->USD) = 158.2 / 1.05
    let conversion_rate = to_rate.checked_div(from_rate).ok_or_else(|| {
        ApiError::CalculationError("Division by zero or overflow in conversion".to_string())
    })?;

    // 4. Calculate final amount
    let result = amount
        .checked_mul(conversion_rate)
        .ok_or_else(|| ApiError::CalculationError("Overflow in amount calculation".to_string()))?;

    Ok((result, conversion_rate))
}

/// Rebase exchange rates from current base to any other currency
/// Only use this when you need to display a complete rate table with a different base
/// For single conversions, use convert_currency() instead (much faster)
///
/// This function:
/// 1. Adds the old base currency to the rates map
/// 2. Excludes the new base currency from the rates map (maintains consistency)
/// 3. Recalculates all other rates relative to the new base
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

    let mut new_rates = HashMap::new();

    // 1. Add the original base currency (e.g., EUR when switching from EUR to USD)
    // EUR in terms of USD = 1 / (EUR->USD rate)
    let old_base_rate = Decimal::ONE
        .checked_div(*base_rate)
        .ok_or_else(|| ApiError::CalculationError("Division by zero in rebase".to_string()))?;
    new_rates.insert(daily_rate.base.clone(), old_base_rate);

    // 2. Recalculate all other rates relative to new base
    // Formula: new_rate = old_rate / base_rate
    for (currency, rate) in &daily_rate.rates {
        // Skip the new base currency itself to maintain "base not in map" consistency
        if *currency == new_base {
            continue;
        }

        let new_rate = rate.checked_div(*base_rate).ok_or_else(|| {
            ApiError::CalculationError(format!("Division error for {}", currency))
        })?;
        new_rates.insert(currency.clone(), new_rate);
    }

    Ok(DailyRate {
        date: daily_rate.date.clone(),
        base: new_base,
        rates: new_rates,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn create_test_rates() -> DailyRate {
        let mut rates = HashMap::new();
        rates.insert("USD".to_string(), dec!(1.05));
        rates.insert("GBP".to_string(), dec!(0.85));
        rates.insert("JPY".to_string(), dec!(158.2));
        rates.insert("EUR".to_string(), dec!(1.0));

        DailyRate {
            date: "2024-12-04".to_string(),
            base: "EUR".to_string(),
            rates,
        }
    }

    #[test]
    fn test_convert_same_currency() {
        let rates = create_test_rates();
        let (result, rate) = convert_currency(&rates, "USD", "USD", dec!(100.0)).unwrap();

        assert_eq!(result, dec!(100.0));
        assert_eq!(rate, Decimal::ONE);
    }

    #[test]
    fn test_convert_usd_to_eur_optimized() {
        let rates = create_test_rates();
        let (result, rate) = convert_currency(&rates, "USD", "EUR", dec!(100.0)).unwrap();

        // 100 USD in EUR: 100 * (1.0 / 1.05) ≈ 95.238095...
        let expected_rate = dec!(1.0) / dec!(1.05);
        let expected_result = dec!(100.0) * expected_rate;

        assert_eq!(rate, expected_rate);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_convert_usd_to_jpy_direct() {
        let rates = create_test_rates();
        let (result, rate) = convert_currency(&rates, "USD", "JPY", dec!(100.0)).unwrap();

        // USD->JPY = (EUR->JPY) / (EUR->USD) = 158.2 / 1.05 = 150.666666...
        let expected_rate = dec!(158.2) / dec!(1.05);
        let expected_result = dec!(100.0) * expected_rate;

        assert_eq!(rate, expected_rate);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_convert_eur_to_usd() {
        let rates = create_test_rates();
        let (result, rate) = convert_currency(&rates, "EUR", "USD", dec!(100.0)).unwrap();

        // EUR->USD = 1.05
        assert_eq!(rate, dec!(1.05));
        assert_eq!(result, dec!(105.0));
    }

    #[test]
    fn test_convert_unknown_currency() {
        let rates = create_test_rates();
        let result = convert_currency(&rates, "USD", "XXX", dec!(100.0));

        assert!(result.is_err());
        match result {
            Err(ApiError::CurrencyNotFound(currency)) => {
                assert_eq!(currency, "XXX");
            }
            _ => panic!("Expected CurrencyNotFound error"),
        }
    }

    #[test]
    fn test_rebase_same_currency() {
        let rates = create_test_rates();
        let result = rebase_rates(&rates, "EUR").unwrap();
        assert_eq!(result.base, "EUR");
        assert_eq!(result.rates["USD"], dec!(1.05));
    }

    #[test]
    fn test_rebase_eur_to_usd() {
        let rates = create_test_rates();
        let result = rebase_rates(&rates, "USD").unwrap();

        assert_eq!(result.base, "USD");

        // EUR in terms of USD: 1.0 / 1.05
        let expected_eur_rate = dec!(1.0) / dec!(1.05);
        assert_eq!(result.rates["EUR"], expected_eur_rate);

        // USD should NOT be in the rates map (base currency)
        assert!(!result.rates.contains_key("USD"));

        // JPY in terms of USD: 158.2 / 1.05
        let expected_jpy_rate = dec!(158.2) / dec!(1.05);
        assert_eq!(result.rates["JPY"], expected_jpy_rate);
    }

    #[test]
    fn test_rebase_does_not_include_new_base() {
        let rates = create_test_rates();
        let result = rebase_rates(&rates, "GBP").unwrap();

        assert_eq!(result.base, "GBP");
        // GBP should NOT appear in the rates map
        assert!(!result.rates.contains_key("GBP"));
        // But EUR (old base) should be there
        assert!(result.rates.contains_key("EUR"));
    }

    #[test]
    fn test_decimal_precision() {
        let rates = create_test_rates();

        // Test that Decimal maintains precision across multiple operations
        let (result1, _rate1) = convert_currency(&rates, "USD", "JPY", dec!(1000.0)).unwrap();
        let (result2, _rate2) = convert_currency(&rates, "JPY", "USD", result1).unwrap();

        // Converting back should give us close to original (within decimal precision)
        // 1000 USD -> JPY -> USD should be ~1000
        let diff = (result2 - dec!(1000.0)).abs();
        assert!(
            diff < dec!(0.0001),
            "Round-trip conversion lost precision: {}",
            diff
        );
    }

    #[test]
    fn test_cross_rate_consistency() {
        let rates = create_test_rates();

        // Test: USD->GBP should equal (EUR->GBP)/(EUR->USD)
        let (_, usd_to_gbp) = convert_currency(&rates, "USD", "GBP", dec!(1.0)).unwrap();
        let expected = dec!(0.85) / dec!(1.05);

        assert_eq!(usd_to_gbp, expected);
    }
}
