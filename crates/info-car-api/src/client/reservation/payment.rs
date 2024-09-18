use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlikPaymentRequest {
    pub balance_usage_requested: bool,
    pub blik_code: String,
}

impl BlikPaymentRequest {
    pub fn new(blik_code: String, balance_usage_requested: bool) -> Self {
        Self {
            blik_code,
            balance_usage_requested,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlikPaymentResponse {
    pub reservation_id: String,
    pub payment_id: String,
    pub bill_id: String,
    /// Known values: ACCEPTED
    pub payment_status: String,
    /// Value in grosz (0,01 PLN)
    pub paid_amount: i32,
}
