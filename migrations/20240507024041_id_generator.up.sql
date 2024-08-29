-- Add up migration script here
CREATE SEQUENCE id_seq;

CREATE OR REPLACE FUNCTION "next_id"(OUT "result" int8)
  RETURNS "pg_catalog"."int8" AS $BODY$
DECLARE
    our_epoch bigint := 978034262000;
    seq_id bigint;
    now_millis bigint;
    shard_id int := 5;
BEGIN
    SELECT nextval('id_seq') % 1024 INTO seq_id;
    SELECT FLOOR(EXTRACT(EPOCH FROM clock_timestamp()) * 1000) INTO now_millis;
    result := (now_millis - our_epoch) << 23;
    result := result | (shard_id <<10);
    result := result | (seq_id);
END;
    $BODY$
  LANGUAGE plpgsql VOLATILE
  COST 100
