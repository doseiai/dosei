CREATE TYPE git_source AS ENUM ('github', 'gitlab', 'bitbucket');

CREATE TABLE IF NOT EXISTS projects (
    id UUID NOT NULL,
    name TEXT NOT NULL,
    owner_id UUID NOT NULL,
    git_source git_source NOT NULL,
    git_source_metadata jsonb NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    PRIMARY KEY (id),
    UNIQUE (name, owner_id)
);

CREATE TYPE deployment_status AS ENUM ('queued', 'building', 'error', 'canceled', 'ready');

CREATE TABLE IF NOT EXISTS deployments (
    id UUID NOT NULL,
    value TEXT NOT NULL,
    project_id UUID NOT NULL,
    owner_id UUID NOT NULL,
    status deployment_status NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    PRIMARY KEY (id)
);
