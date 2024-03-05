CREATE TABLE IF NOT EXISTS locust_domain_coefficients (
  id SERIAL PRIMARY KEY,
  proxy_id integer NOT NULL,
  domain_id integer NOT NULL,
  coefficient integer NOT NULL,
	CONSTRAINT proxy_id_fkey FOREIGN KEY (proxy_id) REFERENCES locust_proxies(id),
	CONSTRAINT domain_id_fkey FOREIGN KEY (domain_id) REFERENCES locust_domains(id),
  UNIQUE(proxy_id, domain_id)
);

CREATE INDEX idx_domain_coefficients_proxy_id ON locust_domain_coefficients(proxy_id);
CREATE INDEX idx_domain_coefficients_domain_id ON locust_domain_coefficients(domain_id);
