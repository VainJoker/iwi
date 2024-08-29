-- Add down migration script here
DROP TRIGGER update_bw_account_updated_at ON bw_account;
DROP INDEX IF EXISTS idx_bw_account_name;
DROP INDEX IF EXISTS idx_bw_account_email;
DROP TABLE IF EXISTS bw_account;
