use bigdecimal::{BigDecimal, RoundingMode, Zero};

/// Weighted average price — Java `PriceUtils.getAveragePrice` (scale 16, HALF_DOWN).
pub fn get_average_price(
    amount: &BigDecimal,
    price: &BigDecimal,
    now_amount: &BigDecimal,
    now_price: &BigDecimal,
) -> BigDecimal {
    let total_amount = price * amount + now_price * now_amount;
    let total_quantity = amount + now_amount;
    if total_quantity.is_zero() {
        return BigDecimal::zero();
    }
    (total_amount / total_quantity)
        .with_scale_round(16, RoundingMode::HalfDown)
        .normalized()
}
