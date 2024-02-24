CREATE TABLE IF NOT EXISTS proxy_metrics (
  id SERIAL PRIMARY KEY,
  proxy_id integer NOT NULL,
  status integer NOT NULL,
  response_time integer NOT NULL,
  metric_date timestamp DEFAULT now(),
  CONSTRAINT proxy_id_fkey FOREIGN KEY (proxy_id) REFERENCES proxies(id)
);

CREATE INDEX idx_proxy_metrics_proxy_id ON proxy_metrics(proxy_id);
CREATE INDEX idx_proxy_metrics_status ON proxy_metrics(status);
