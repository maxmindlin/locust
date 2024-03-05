CREATE TABLE IF NOT EXISTS locust_sessions (
  id SERIAL PRIMARY KEY,
  proxy_id integer NOT NULL,
  CONSTRAINT proxy_id_fkey FOREIGN KEY (proxy_id) REFERENCES locust_proxies(id)
);
