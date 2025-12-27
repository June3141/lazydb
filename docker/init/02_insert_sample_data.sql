-- Sample data for testing lazydb

-- Insert users
INSERT INTO users (id, username, email, password_hash, full_name, is_active) VALUES
    ('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 'alice', 'alice@example.com', '$2b$12$hash1', 'Alice Johnson', true),
    ('b0eebc99-9c0b-4ef8-bb6d-6bb9bd380a12', 'bob', 'bob@example.com', '$2b$12$hash2', 'Bob Smith', true),
    ('c0eebc99-9c0b-4ef8-bb6d-6bb9bd380a13', 'charlie', 'charlie@example.com', '$2b$12$hash3', 'Charlie Brown', true),
    ('d0eebc99-9c0b-4ef8-bb6d-6bb9bd380a14', 'diana', 'diana@example.com', '$2b$12$hash4', 'Diana Prince', false),
    ('e0eebc99-9c0b-4ef8-bb6d-6bb9bd380a15', 'eve', 'eve@example.com', '$2b$12$hash5', 'Eve Wilson', true);

-- Insert categories
INSERT INTO categories (name, description, parent_id) VALUES
    ('Electronics', 'Electronic devices and accessories', NULL),
    ('Computers', 'Desktop and laptop computers', 1),
    ('Smartphones', 'Mobile phones and accessories', 1),
    ('Clothing', 'Apparel and fashion items', NULL),
    ('Men''s Clothing', 'Clothing for men', 4),
    ('Women''s Clothing', 'Clothing for women', 4),
    ('Books', 'Physical and digital books', NULL),
    ('Home & Garden', 'Home improvement and garden supplies', NULL);

-- Insert products
INSERT INTO products (name, description, price, stock_quantity, category_id, is_available, metadata) VALUES
    ('MacBook Pro 14"', 'Apple MacBook Pro with M3 chip', 1999.99, 50, 2, true, '{"brand": "Apple", "color": "Space Gray", "specs": {"ram": "16GB", "storage": "512GB"}}'),
    ('ThinkPad X1 Carbon', 'Lenovo business laptop', 1499.99, 30, 2, true, '{"brand": "Lenovo", "color": "Black", "specs": {"ram": "16GB", "storage": "256GB"}}'),
    ('iPhone 15 Pro', 'Apple smartphone with titanium design', 999.99, 100, 3, true, '{"brand": "Apple", "color": "Natural Titanium", "specs": {"storage": "256GB"}}'),
    ('Galaxy S24 Ultra', 'Samsung flagship smartphone', 1199.99, 75, 3, true, '{"brand": "Samsung", "color": "Titanium Black", "specs": {"storage": "512GB"}}'),
    ('Classic T-Shirt', '100% cotton basic tee', 29.99, 200, 5, true, '{"material": "Cotton", "sizes": ["S", "M", "L", "XL"]}'),
    ('Summer Dress', 'Floral print summer dress', 79.99, 50, 6, true, '{"material": "Polyester", "sizes": ["XS", "S", "M", "L"]}'),
    ('The Rust Programming Language', 'Official Rust book', 39.99, 150, 7, true, '{"author": "Steve Klabnik", "format": "Paperback", "pages": 560}'),
    ('Clean Code', 'A handbook of agile software craftsmanship', 44.99, 80, 7, true, '{"author": "Robert C. Martin", "format": "Paperback", "pages": 464}'),
    ('Garden Tool Set', '5-piece stainless steel garden tools', 49.99, 60, 8, true, '{"pieces": 5, "material": "Stainless Steel"}'),
    ('Wireless Mouse', 'Ergonomic wireless mouse', 39.99, 150, 1, true, '{"brand": "Logitech", "connection": "Bluetooth"}'),
    ('USB-C Hub', '7-in-1 USB-C adapter', 59.99, 100, 1, true, '{"ports": 7, "brand": "Anker"}'),
    ('Vintage Jeans', 'Classic fit vintage wash jeans', 89.99, 40, 5, false, '{"material": "Denim", "sizes": ["30", "32", "34", "36"]}');

-- Insert orders
INSERT INTO orders (user_id, status, total_amount, shipping_address, notes) VALUES
    ('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 'delivered', 2039.98, '123 Main St, New York, NY 10001', 'Leave at door'),
    ('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 'shipped', 79.99, '123 Main St, New York, NY 10001', NULL),
    ('b0eebc99-9c0b-4ef8-bb6d-6bb9bd380a12', 'processing', 1199.99, '456 Oak Ave, Los Angeles, CA 90001', 'Gift wrap please'),
    ('c0eebc99-9c0b-4ef8-bb6d-6bb9bd380a13', 'pending', 84.98, '789 Pine Rd, Chicago, IL 60601', NULL),
    ('e0eebc99-9c0b-4ef8-bb6d-6bb9bd380a15', 'cancelled', 1999.99, '321 Elm St, Seattle, WA 98101', 'Changed my mind');

-- Insert order items
INSERT INTO order_items (order_id, product_id, quantity, unit_price) VALUES
    (1, 1, 1, 1999.99),
    (1, 10, 1, 39.99),
    (2, 6, 1, 79.99),
    (3, 4, 1, 1199.99),
    (4, 7, 1, 39.99),
    (4, 8, 1, 44.99),
    (5, 1, 1, 1999.99);
