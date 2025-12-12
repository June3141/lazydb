-- Bulk data generation for stress testing lazydb
-- This script generates a large amount of test data

-- Generate additional users (1000 users)
INSERT INTO users (username, email, password_hash, full_name, is_active)
SELECT
    'user_' || i,
    'user_' || i || '@example.com',
    '$2b$12$bulkhash' || i,
    'User Number ' || i,
    (random() > 0.1)  -- 90% active
FROM generate_series(1, 1000) AS i;

-- Generate additional categories (50 categories)
INSERT INTO categories (name, description, parent_id)
SELECT
    'Category ' || i,
    'Description for category ' || i || '. This is a test category with some additional text to make it more realistic.',
    CASE WHEN random() > 0.7 THEN (SELECT id FROM categories ORDER BY random() LIMIT 1) ELSE NULL END
FROM generate_series(10, 59) AS i;

-- Generate products (10000 products)
INSERT INTO products (name, description, price, stock_quantity, category_id, is_available, metadata)
SELECT
    'Product ' || i || ' - ' ||
    CASE (i % 5)
        WHEN 0 THEN 'Standard'
        WHEN 1 THEN 'Premium'
        WHEN 2 THEN 'Basic'
        WHEN 3 THEN 'Pro'
        ELSE 'Lite'
    END,
    'This is a detailed description for product ' || i || '. ' ||
    'It includes various features and specifications. ' ||
    'Lorem ipsum dolor sit amet, consectetur adipiscing elit. ' ||
    'Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.',
    (random() * 1000 + 1)::DECIMAL(10,2),
    (random() * 500)::INTEGER,
    (SELECT id FROM categories ORDER BY random() LIMIT 1),
    (random() > 0.15),  -- 85% available
    jsonb_build_object(
        'sku', 'SKU-' || lpad(i::TEXT, 6, '0'),
        'weight', (random() * 10)::DECIMAL(5,2),
        'dimensions', jsonb_build_object(
            'width', (random() * 50)::INTEGER,
            'height', (random() * 50)::INTEGER,
            'depth', (random() * 50)::INTEGER
        ),
        'tags', ARRAY['tag' || (i % 10), 'tag' || (i % 20), 'tag' || (i % 50)],
        'rating', (random() * 5)::DECIMAL(2,1),
        'review_count', (random() * 1000)::INTEGER
    )
FROM generate_series(13, 10012) AS i;

-- Generate orders (5000 orders)
INSERT INTO orders (user_id, status, total_amount, shipping_address, notes, created_at)
SELECT
    (SELECT id FROM users ORDER BY random() LIMIT 1),
    (ARRAY['pending', 'processing', 'shipped', 'delivered', 'cancelled'])[floor(random() * 5 + 1)],
    (random() * 5000 + 10)::DECIMAL(12,2),
    (floor(random() * 9999) + 1)::TEXT || ' ' ||
    (ARRAY['Main St', 'Oak Ave', 'Pine Rd', 'Elm St', 'Maple Dr', 'Cedar Ln', 'Birch Way', 'Walnut Blvd'])[floor(random() * 8 + 1)] || ', ' ||
    (ARRAY['New York', 'Los Angeles', 'Chicago', 'Houston', 'Phoenix', 'Seattle', 'Denver', 'Boston', 'Miami', 'Portland'])[floor(random() * 10 + 1)] || ', ' ||
    (ARRAY['NY', 'CA', 'IL', 'TX', 'AZ', 'WA', 'CO', 'MA', 'FL', 'OR'])[floor(random() * 10 + 1)] || ' ' ||
    lpad((floor(random() * 99999) + 10000)::TEXT, 5, '0'),
    CASE WHEN random() > 0.7 THEN 'Special instructions: ' || (ARRAY['Handle with care', 'Leave at door', 'Ring doorbell', 'Call before delivery', 'Gift wrap requested'])[floor(random() * 5 + 1)] ELSE NULL END,
    CURRENT_TIMESTAMP - (random() * 365 || ' days')::INTERVAL
FROM generate_series(1, 5000) AS i;

-- Generate order items (average 3 items per order = ~15000 items)
INSERT INTO order_items (order_id, product_id, quantity, unit_price, created_at)
SELECT
    o.id,
    (SELECT id FROM products ORDER BY random() LIMIT 1),
    (floor(random() * 5) + 1)::INTEGER,
    (random() * 500 + 5)::DECIMAL(10,2),
    o.created_at + (random() * 60 || ' minutes')::INTERVAL
FROM orders o
CROSS JOIN generate_series(1, 3) AS item_num
WHERE random() > 0.2;  -- ~80% chance for each item slot

-- Update order totals based on actual items
UPDATE orders o
SET total_amount = (
    SELECT COALESCE(SUM(quantity * unit_price), 0)
    FROM order_items oi
    WHERE oi.order_id = o.id
)
WHERE EXISTS (SELECT 1 FROM order_items oi WHERE oi.order_id = o.id);

-- Create additional indexes for bulk data performance
CREATE INDEX IF NOT EXISTS idx_products_name ON products(name);
CREATE INDEX IF NOT EXISTS idx_products_created ON products(created_at);
CREATE INDEX IF NOT EXISTS idx_orders_created ON orders(created_at);
CREATE INDEX IF NOT EXISTS idx_orders_total ON orders(total_amount);

-- Analyze tables for query optimization
ANALYZE users;
ANALYZE categories;
ANALYZE products;
ANALYZE orders;
ANALYZE order_items;

-- Show data counts
DO $$
DECLARE
    user_count INTEGER;
    category_count INTEGER;
    product_count INTEGER;
    order_count INTEGER;
    order_item_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO user_count FROM users;
    SELECT COUNT(*) INTO category_count FROM categories;
    SELECT COUNT(*) INTO product_count FROM products;
    SELECT COUNT(*) INTO order_count FROM orders;
    SELECT COUNT(*) INTO order_item_count FROM order_items;

    RAISE NOTICE '=== Data Generation Complete ===';
    RAISE NOTICE 'Users: %', user_count;
    RAISE NOTICE 'Categories: %', category_count;
    RAISE NOTICE 'Products: %', product_count;
    RAISE NOTICE 'Orders: %', order_count;
    RAISE NOTICE 'Order Items: %', order_item_count;
END $$;
