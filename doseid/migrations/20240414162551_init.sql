CREATE TABLE IF NOT EXISTS "user" (
    id UUID NOT NULL,
    username TEXT NOT NULL,
    password TEXT,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    PRIMARY KEY (id),
    UNIQUE (username)
);

CREATE TABLE IF NOT EXISTS session (
    id UUID NOT NULL,
    token TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    owner_id UUID NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    FOREIGN KEY (owner_id) REFERENCES "user"(id),
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS service (
    id UUID NOT NULL,
    name TEXT NOT NULL,
    owner_id UUID NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (owner_id) REFERENCES "user"(id),
    UNIQUE (name, owner_id)
);
--
CREATE TABLE IF NOT EXISTS deployment (
    id UUID NOT NULL,
    service_id UUID NOT NULL,
    owner_id UUID NOT NULL,
    exposed_port smallint,
    internal_port smallint,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (owner_id) REFERENCES "user"(id),
    FOREIGN KEY (service_id) REFERENCES service(id)
);

CREATE TABLE IF NOT EXISTS env (
    id UUID NOT NULL,
    name TEXT NOT NULL,
    value TEXT NOT NULL,
    service_id UUID NOT NULL,
    deployment_id UUID,
    owner_id UUID NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (owner_id) REFERENCES "user"(id),
    FOREIGN KEY (service_id) REFERENCES service(id),
    FOREIGN KEY (deployment_id) REFERENCES deployment(id)
);