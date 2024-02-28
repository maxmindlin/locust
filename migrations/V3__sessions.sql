CREATE TABLE IF NOT EXISTS sessions (
  id SERIAL PRIMARY KEY,
  time TIMESTAMPTZ NOT NULL,
  proxy_id integer NOT NULL,
  CONSTRAINT proxy_id_fkey FOREIGN KEY (proxy_id) REFERENCES proxies(id)
);

SELECT create_hypertable('sessions', by_range('time'));
SELECT add_retention_policy('sessions', INTERVAL '1 week');

CREATE INDEX idx_proxy_id_time ON sessions(proxy_id time DESC);
