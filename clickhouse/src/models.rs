use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Product {
    pub id: u32,
    pub name: String,
    pub price: f64,
    pub category: String,
    pub stock: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    pub id: u32,
    pub user_id: u32,
    pub product_id: u32,
    pub quantity: u32,
    pub total_amount: f64,
    pub order_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub database_name: String,
    pub table_count: u32,
    pub total_size: String,
    pub version: String,
}
