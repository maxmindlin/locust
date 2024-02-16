CREATE TABLE IF NOT EXISTS proxies (
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

CREATE TABLE IF NOT EXISTS tags (
  id SERIAL PRIMARY KEY,
  name varchar NOT NULL,
  UNIQUE(name)
);

CREATE TABLE IF NOT EXISTS proxy_tag_map (
  id SERIAL PRIMARY KEY,
  proxy_id integer NOT NULL,
  tag_id integer NOT NULL,
	CONSTRAINT proxy_id_fkey FOREIGN KEY (proxy_id) REFERENCES proxies(id),
	CONSTRAINT tag_id_fkey FOREIGN KEY (tag_id) REFERENCES tags(id)
);

CREATE TABLE IF NOT EXISTS domains (
  id SERIAL PRIMARY KEY,
  host varchar NOT NULL,
	date_created timestamp DEFAULT now(),
	date_modified timestamp DEFAULT now(),
	date_deleted timestamp NULL,
  UNIQUE(host)
);

CREATE TABLE IF NOT EXISTS domain_tag_map (
  id SERIAL PRIMARY KEY,
  domain_id integer NOT NULL,
  tag_id integer NOT NULL,
	CONSTRAINT domain_id_fkey FOREIGN KEY (domain_id) REFERENCES domains(id),
	CONSTRAINT tag_id_fkey FOREIGN KEY (tag_id) REFERENCES tags(id)
);

CREATE INDEX idx_proxy_map_proxy_id ON proxy_tag_map(proxy_id);
CREATE INDEX idx_proxy_map_tag_id ON proxy_tag_map(tag_id);
CREATE INDEX idx_domain_map_domain_id ON domain_tag_map(domain_id);
CREATE INDEX idx_domain_map_tag_id ON domain_tag_map(tag_id);
CREATE INDEX idx_name ON tags(name);
CREATE INDEX idx_provider ON proxies(provider);
CREATE INDEX idx_date_last_used ON proxies(date_last_used);
CREATE INDEX idx_host ON domains(host);
