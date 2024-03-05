CREATE TABLE IF NOT EXISTS locust_proxies (
	id SERIAL PRIMARY KEY,
	protocol varchar NOT NULL,
	host varchar NOT NULL,
	port integer,
  username varchar,
  password varchar,
  provider varchar,
	date_created timestamp DEFAULT now(),
	date_modified timestamp DEFAULT now(),
	date_deleted timestamp NULL,
  date_last_used timestamp NULL
);

CREATE TABLE IF NOT EXISTS locust_tags (
  id SERIAL PRIMARY KEY,
  name varchar NOT NULL,
  UNIQUE(name)
);

CREATE TABLE IF NOT EXISTS locust_proxy_tag_map (
  id SERIAL PRIMARY KEY,
  proxy_id integer NOT NULL,
  tag_id integer NOT NULL,
	CONSTRAINT proxy_id_fkey FOREIGN KEY (proxy_id) REFERENCES locust_proxies(id),
	CONSTRAINT tag_id_fkey FOREIGN KEY (tag_id) REFERENCES locust_tags(id)
);

CREATE TABLE IF NOT EXISTS locust_domains (
  id SERIAL PRIMARY KEY,
  host varchar NOT NULL,
	date_created timestamp DEFAULT now(),
	date_modified timestamp DEFAULT now(),
	date_deleted timestamp NULL,
  UNIQUE(host)
);

CREATE TABLE IF NOT EXISTS locust_domain_tag_map (
  id SERIAL PRIMARY KEY,
  domain_id integer NOT NULL,
  tag_id integer NOT NULL,
	CONSTRAINT domain_id_fkey FOREIGN KEY (domain_id) REFERENCES locust_domains(id),
	CONSTRAINT tag_id_fkey FOREIGN KEY (tag_id) REFERENCES locust_tags(id)
);

CREATE INDEX idx_proxy_map_proxy_id ON locust_proxy_tag_map(proxy_id);
CREATE INDEX idx_proxy_map_tag_id ON locust_proxy_tag_map(tag_id);
CREATE INDEX idx_domain_map_domain_id ON locust_domain_tag_map(domain_id);
CREATE INDEX idx_domain_map_tag_id ON locust_domain_tag_map(tag_id);
CREATE INDEX idx_name ON locust_tags(name);
CREATE INDEX idx_provider ON locust_proxies(provider);
CREATE INDEX idx_date_last_used ON locust_proxies(date_last_used);
CREATE INDEX idx_host ON locust_domains(host);
