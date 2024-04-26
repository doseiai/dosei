DO $$
    BEGIN
        IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'account_type') THEN
            CREATE TYPE account_type AS ENUM ('individual', 'organization');
        END IF;
    END
$$;

CREATE TABLE IF NOT EXISTS account (
    id UUID NOT NULL,
    name TEXT NOT NULL,
    type account_type NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    PRIMARY KEY (id),
    UNIQUE (name)
);

CREATE TABLE IF NOT EXISTS "user" (
    id UUID NOT NULL,
    password TEXT,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    FOREIGN KEY (id) REFERENCES account(id),
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS organization (
    id UUID NOT NULL,
    display_name TEXT NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    FOREIGN KEY (id) REFERENCES account(id),
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS organization_member (
    user_id UUID NOT NULL,
    organization_id UUID NOT NULL,
    role_id UUID NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    FOREIGN KEY (user_id) REFERENCES "user"(id),
    FOREIGN KEY (organization_id) REFERENCES organization(id)
);

CREATE TABLE IF NOT EXISTS session (
    id UUID NOT NULL,
    token TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    user_id UUID NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    FOREIGN KEY (user_id) REFERENCES "user"(id),
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS service (
    id UUID NOT NULL,
    name TEXT NOT NULL,
    owner_id UUID NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (owner_id) REFERENCES account(id),
    UNIQUE (name, owner_id)
);

DO $$
    BEGIN
        IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'deployment_status') THEN
            CREATE TYPE deployment_status AS ENUM ('pending', 'error', 'ready', 'running', 'stopped');
        END IF;
    END
$$;

CREATE TABLE IF NOT EXISTS deployment (
    id UUID NOT NULL,
    service_id UUID NOT NULL,
    owner_id UUID NOT NULL,
    host_port smallint,
    container_port smallint,
    status deployment_status NOT NULL DEFAULT 'pending',
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (owner_id) REFERENCES account(id),
    FOREIGN KEY (service_id) REFERENCES service(id)
);

CREATE TABLE IF NOT EXISTS domain (
    id UUID NOT NULL,
    name TEXT NOT NULL,
    service_id UUID NOT NULL,
    owner_id UUID NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (owner_id) REFERENCES account(id),
    FOREIGN KEY (service_id) REFERENCES service(id),
    UNIQUE (name, owner_id)
);

CREATE TABLE IF NOT EXISTS env (
    id UUID NOT NULL,
    name TEXT NOT NULL,
    value TEXT NOT NULL,
    key TEXT,
    nonce TEXT,
    service_id UUID,
    deployment_id UUID,
    owner_id UUID NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (owner_id) REFERENCES account(id),
    FOREIGN KEY (service_id) REFERENCES service(id),
    FOREIGN KEY (deployment_id) REFERENCES deployment(id),
    UNIQUE (name, service_id, owner_id)
);
