#[cfg(test)]
#[path = "acceptance/support.rs"]
mod support;

#[cfg(test)]
#[path = "acceptance/cmd_reserve_stock.rs"]
mod cmd_reserve_stock;

#[cfg(test)]
#[path = "acceptance/cmd_release_stock.rs"]
mod cmd_release_stock;

#[cfg(test)]
#[path = "acceptance/qry_sku_stock.rs"]
mod qry_sku_stock;
