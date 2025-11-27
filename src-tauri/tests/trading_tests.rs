#[cfg(test)]
mod tests {
    use app_lib::trading::types::{CreateOrderRequest, OrderSide, OrderType};

    #[test]
    fn test_order_creation() {
        let request = CreateOrderRequest {
            order_type: OrderType::Limit,
            side: OrderSide::Buy,
            input_mint: "So11111111111111111111111111111111111111112".to_string(),
            output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            input_symbol: "SOL".to_string(),
            output_symbol: "USDC".to_string(),
            amount: 1.0,
            limit_price: Some(50.0),
            stop_price: None,
            trailing_percent: None,
            linked_order_id: None,
            slippage_bps: 50,
            priority_fee_micro_lamports: 5000,
            wallet_address: "test_wallet".to_string(),
        };

        assert_eq!(request.order_type, OrderType::Limit);
        assert_eq!(request.side, OrderSide::Buy);
        assert_eq!(request.amount, 1.0);
    }

    #[test]
    fn test_order_type_display() {
        assert_eq!(OrderType::Limit.to_string(), "limit");
        assert_eq!(OrderType::StopLoss.to_string(), "stop_loss");
        assert_eq!(OrderType::TakeProfit.to_string(), "take_profit");
        assert_eq!(OrderType::TrailingStop.to_string(), "trailing_stop");
    }

    #[test]
    fn test_order_side_display() {
        assert_eq!(OrderSide::Buy.to_string(), "buy");
        assert_eq!(OrderSide::Sell.to_string(), "sell");
    }
}
