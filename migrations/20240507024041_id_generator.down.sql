-- Add down migration script here
DROP FUNCTION IF EXISTS next_id();
DROP SEQUENCE IF EXISTS id_seq;
