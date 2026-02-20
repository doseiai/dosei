ALTER TABLE deployment ADD COLUMN node_id UUID REFERENCES node(id);
