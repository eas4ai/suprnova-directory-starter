-- Synthetic records inserted only after the actual 3.2.1 migrations run.
INSERT INTO users (id, name, email, password, email_verified_at) VALUES
 (9000000001, 'Legacy moderator', 'moderator@example.test', 'fixture-hash', '2026-10-01 00:00:00'),
 (9000000002, 'Direct editor', 'editor@example.test', 'fixture-hash', '2026-10-01 00:00:00'),
 (9000000003, 'Community member', 'member@example.test', 'fixture-hash', '2026-10-01 00:00:00');
INSERT INTO roles (id, name, display_name) VALUES (31, 'moderator', 'Legacy moderator'), (32, 'community', 'Community');
INSERT INTO permissions (id, name, display_name) VALUES
 (41, 'admin.access', 'Enter administration'), (42, 'listings.moderate', 'Review listings'),
 (43, 'articles.manage', 'Edit articles'), (44, 'community.read', 'Read community');
INSERT INTO role_permissions (role_id, permission_id) VALUES (31, 41), (31, 42), (32, 44);
INSERT INTO model_roles (model_type, model_id, role_id) VALUES ('directory.user', '9000000001', 31), ('directory.user', '9000000003', 32);
INSERT INTO model_permissions (model_type, model_id, permission_id) VALUES ('directory.user', '9000000002', 43);
INSERT INTO sessions (id, user_id, payload, csrf_token, last_activity)
 VALUES ('legacy-session', '9000000001', '{"draft":"Keep this draft"}', 'legacy-csrf', '2026-10-01 00:00:00');
INSERT INTO listings (id, owner_id, slug, created_at) VALUES (51, 9000000001, 'legacy-listing', 1790812800);
INSERT INTO listing_revisions (id, listing_id, title, summary, search_text, description, url, media_alt, status, created_at)
 VALUES (61, 51, 'Legacy listing', 'Preserved summary', 'legacy listing', 'Preserved content', 'https://example.test', '', 'approved', 1790812800);
UPDATE listings SET current_revision_id = 61, approved_revision_id = 61 WHERE id = 51;
INSERT INTO publishing_plans (key, name, description, enabled, billing_type, amount, currency, version, created_at, updated_at)
 VALUES ('legacy-plan', 'Legacy plan', 'Paid publication', 1, 'one_time', 2500, 'USD', 1, 1790812800, 1790812800);
INSERT INTO publishing_purchases (id, listing_id, owner_id, approved_revision_id, plan_key, plan_name, billing_type, amount, currency, provider, mode, return_origin, state, version, cancel_requested, cancel_event_at, created_at, updated_at)
 VALUES ('purchase-1', 51, 9000000001, 61, 'legacy-plan', 'Legacy plan', 'one_time', 2500, 'USD', 'stripe', 'test', 'https://example.test', 'paid', 1, 0, 0, 1790812800, 1790812800);
INSERT INTO publishing_payments (id, purchase_id, provider_payment_id, amount_total, period_start, status, paid_at, refund_event_at, dispute_event_at, adverse, created_at)
 VALUES ('payment-1', 'purchase-1', 'pi_fixture', 2500, 1790812800, 'paid', 1790812800, 0, 0, '', 1790812800);
INSERT INTO publishing_events (id, provider, mode, provider_event_id, event_type, raw_body, signature, occurred_at, received_at, purchase_id, credential_source, status, attempts, next_attempt_at)
 VALUES ('event-1', 'stripe', 'test', 'evt_fixture', 'payment.completed', X'7B7D', 'fixture-signature', 1790812800, 1790812800, 'purchase-1', 'fixture', 'applied', 1, 0);
INSERT INTO publishing_receipts (event_record_id, purchase_id, outcome, applied_at) VALUES ('event-1', 'purchase-1', 'paid', 1790812800);
INSERT INTO publication_entitlements (id, listing_id, mode, status, valid_from, updated_at)
 VALUES ('entitlement-1', 51, 'paid', 'active', 1790812800, 1790812800);
