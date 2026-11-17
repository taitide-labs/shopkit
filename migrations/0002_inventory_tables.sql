CREATE TABLE IF NOT EXISTS sku_stocks (
    sku TEXT PRIMARY KEY,
    available_qty INT NOT NULL CHECK (available_qty >= 0),
    reserved_qty INT NOT NULL CHECK (reserved_qty >= 0),
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS stock_reservations (
    reservation_id UUID PRIMARY KEY,
    order_id UUID NOT NULL UNIQUE,
    status TEXT NOT NULL,
    items JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    released_at TIMESTAMPTZ
);
