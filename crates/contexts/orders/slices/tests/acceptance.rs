#[cfg(test)]
#[path = "acceptance/support.rs"]
mod support;

#[cfg(test)]
#[path = "acceptance/cmd_place_order.rs"]
mod cmd_place_order;

#[cfg(test)]
#[path = "acceptance/cmd_cancel_order.rs"]
mod cmd_cancel_order;

#[cfg(test)]
#[path = "acceptance/qry_order_detail.rs"]
mod qry_order_detail;
