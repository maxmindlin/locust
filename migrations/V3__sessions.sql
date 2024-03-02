CREATE TABLE IF NOT EXISTS sessions (
  id SERIAL PRIMARY KEY,
  proxy_id integer NOT NULL,
  CONSTRAINT proxy_id_fkey FOREIGN KEY (proxy_id) REFERENCES proxies(id)
);
