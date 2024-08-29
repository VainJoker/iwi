-- Add up migration script here
CREATE OR REPLACE FUNCTION update_at() 
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW(); 
  RETURN NEW; 
END
$$ language 'plpgsql';
