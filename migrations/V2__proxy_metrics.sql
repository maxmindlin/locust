CREATE TABLE IF NOT EXISTS proxy_metrics (
  time TIMESTAMPTZ NOT NULL,
  proxy_id integer NOT NULL,
  status integer NOT NULL,
  success boolean NOT NULL,
  response_time integer NOT NULL,
  domain varchar NOT NULL,
  CONSTRAINT proxy_id_fkey FOREIGN KEY (proxy_id) REFERENCES proxies(id)
);

SELECT create_hypertable('proxy_metrics', by_range('time'));
SELECT add_retention_policy('proxy_metrics', INTERVAL '1 month');

CREATE INDEX idx_status_time ON proxy_metrics(status, time DESC);
CREATE INDEX idx_proxy_id_time ON proxy_metrics(proxy_id, time DESC);
CREATE INDEX idx_response_time_time ON proxy_metrics(response_time, time DESC);
CREATE INDEX idx_domain_time ON proxy_metrics(domain, time DESC);
CREATE INDEX idx_success_time ON proxy_metrics(success, time DESC);
CREATE INDEX idx_proxy_metrics_proxy_id ON proxy_metrics(proxy_id);
CREATE INDEX idx_proxy_metrics_status ON proxy_metrics(status);
CREATE INDEX idx_proxy_metrics_domain ON proxy_metrics(domain);

CREATE MATERIALIZED VIEW proxy_metrics_daily
WITH (timescaledb.continuous) AS
SELECT
  domain,
  proxy_id,
  success,
  time_bucket(INTERVAL '1 day', time) as bucket,
  AVG(response_time),
  MAX(response_time),
  MIN(response_time)
FROM proxy_metrics
GROUP BY domain, proxy_id, success, bucket
WITH NO DATA;

SELECT add_continuous_aggregate_policy('proxy_metrics_daily',
  start_offset => INTERVAL '1 month',
  end_offset => INTERVAL '1 day',
  schedule_interval => INTERVAL '1 hour');

