CREATE TABLE google (
  id VARCHAR(255) PRIMARY KEY NOT NULL,
  user_id UUID REFERENCES users(id),
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE OR REPLACE FUNCTION insert_user_with_google(
    param_google_id VARCHAR(255),
    param_name VARCHAR(20),
    param_email VARCHAR(32)
  )
  RETURNS users
  LANGUAGE plpgsql
  SECURITY DEFINER
  AS $$
  DECLARE
    user_entry users;
BEGIN
  INSERT INTO users (name, email)
    VALUES (param_name, param_email)
    RETURNING * 
    INTO user_entry;
  INSERT INTO google (id, user_id)
    VALUES (param_google_id, user_entry.id);
  RETURN user_entry;
END;
$$;
CREATE OR REPLACE FUNCTION lookup_user_with_google(
    param_google_id VARCHAR(255)
  ) 
  RETURNS users 
  LANGUAGE plpgsql 
  SECURITY DEFINER 
  AS $$ 
BEGIN
  RETURN (SELECT u FROM google g 
    INNER JOIN users u 
      ON g.user_id = u.id
      AND g.id = param_google_id);
END;
$$;